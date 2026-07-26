//! `/api/v1/streaming` WebSocket
//!
//! ワイヤフォーマットは `shared::StreamEvent` に統一:
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

use mithic_core::models::actor::ActorId;
use mithic_core::services::auth::verify_jwt;
use mithic_core::AppError;
use mithic_db::queries::get_actor_by_id;

use crate::events::StreamBroadcast;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    token: Option<String>,
}

async fn authenticate(state: &AppState, token: &str) -> Option<ActorId> {
    let claims = verify_jwt(token, &state.config().jwt_secret).ok()?;
    let user_id = claims.sub.parse::<ActorId>().ok()?;
    let actor = get_actor_by_id(state.surreal(), &user_id).await.ok()??;
    if actor.is_suspended {
        return None;
    }
    match actor.token.as_deref() {
        Some(stored) if stored == token => Some(user_id),
        _ => None,
    }
}

pub async fn streaming_handler(
    State(state): State<AppState>,
    Query(query): Query<StreamQuery>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let token = query
        .token
        .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?;
    let user_id = authenticate(&state, &token)
        .await
        .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

fn encode_event(ev: &StreamEvent) -> Option<String> {
    serde_json::to_string(ev).ok()
}

async fn handle_socket(mut socket: WebSocket, state: AppState, user_id: ActorId) {
    let user_id_str = user_id.to_string();
    debug!("Streaming connection opened for {user_id_str}");

    // ponytail: 接続 1000 超では per-user チャネルへ切替を検討
    let mut bus_rx = state.subscribe_stream();

    let (conn, mut rx) = mithic_stream::StreamConnection::new(
        Some(user_id_str.clone()),
        state.surreal().clone(),
        state.dragonfly().clone(),
    );
    let conn = std::sync::Arc::new(conn);

    loop {
        tokio::select! {
            msg = rx.recv() => {
                let Some(msg) = msg else { break };
                // チャンネル系メッセージはそのまま (connected / channel 等)
                let payload = match serde_json::to_string(&msg) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to serialize stream message: {e}");
                        continue;
                    }
                };
                if socket.send(Message::Text(payload.into())).await.is_err() {
                    break;
                }
            }
            bus_msg = bus_rx.recv() => {
                match bus_msg {
                    Ok(StreamBroadcast::Note(note)) => {
                        let Some(payload) = encode_event(&StreamEvent::Note(note)) else {
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
                        let Some(payload) = encode_event(&StreamEvent::Notification(notification)) else {
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
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<mithic_stream::ClientMessage>(&text) {
                            let conn_clone = conn.clone();
                            tokio::spawn(async move {
                                conn_clone.handle_message(client_msg).await;
                            });
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }

    debug!("Streaming connection closed for {user_id_str}");
}
