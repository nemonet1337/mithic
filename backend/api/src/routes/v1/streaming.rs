//! `/api/v1/streaming` WebSocket
//!
//! ワイヤフォーマットは `shared::StreamEvent`:
//! `{ "type": "note"|"notification", "body": ... }`

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use serde::Deserialize;
use shared::StreamEvent;
use tracing::{debug, warn};

use mithic_core::AppError;
use mithic_core::models::actor::ActorId;

use crate::events::StreamBroadcast;
use crate::middleware::resolve_bearer;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    token: Option<String>,
}

pub async fn streaming_handler(
    State(state): State<AppState>,
    Query(query): Query<StreamQuery>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let token = query
        .token
        .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?;
    let auth = resolve_bearer(&state, &token).await?;
    let user_id = auth.user_id;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, user_id: ActorId) {
    let user_id_str = user_id.to_string();
    debug!("Streaming connection opened for {user_id_str}");

    // ponytail: 接続 1000 超では per-user チャネルへ切替を検討
    let mut bus_rx = state.subscribe_stream();

    loop {
        tokio::select! {
            bus_msg = bus_rx.recv() => {
                match bus_msg {
                    Ok(StreamBroadcast::Note(note)) => {
                        let Ok(payload) = serde_json::to_string(&StreamEvent::Note(note)) else {
                            continue;
                        };
                        if socket.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Ok(StreamBroadcast::Notification { user_id: target, notification }) => {
                        if target != user_id_str {
                            continue;
                        }
                        let Ok(payload) =
                            serde_json::to_string(&StreamEvent::Notification(notification))
                        else {
                            continue;
                        };
                        if socket.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Ok(StreamBroadcast::NoteDeleted { id }) => {
                        let Ok(payload) =
                            serde_json::to_string(&StreamEvent::NoteDeleted { id })
                        else {
                            continue;
                        };
                        if socket.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Stream bus lagged by {n} messages for {user_id_str}");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {} // client text frames ignored; server is push-only
                    Some(Err(_)) => break,
                }
            }
        }
    }

    debug!("Streaming connection closed for {user_id_str}");
}
