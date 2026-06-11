//! `/api/streaming` WebSocket ルート (TODO Phase 5)
//!
//! `?token=` クエリで JWT 認証し、プロセス内イベントバス
//! (`crate::events`) のイベントをクライアントへ配信する。

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use serde::Deserialize;
use tracing::{debug, warn};

use mithic_core::AppError;
use mithic_core::models::actor::ActorId;

use crate::events::StreamBroadcast;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[allow(dead_code)]
    exp: i64,
    #[serde(default)]
    typ: String,
}

fn authenticate(token: &str, secret: &str) -> Option<ActorId> {
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .ok()?;
    if data.claims.typ != "access" {
        return None;
    }
    data.claims.sub.parse::<ActorId>().ok()
}

pub async fn streaming_handler(
    State(state): State<AppState>,
    Query(query): Query<StreamQuery>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let token = query
        .token
        .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?;
    let user_id = authenticate(&token, state.config().jwt_secret())
        .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, user_id: ActorId) {
    let mut rx = state.subscribe_stream();
    let user_id_str = user_id.to_string();
    debug!("Streaming connection opened for {}", user_id_str);

    loop {
        tokio::select! {
            event = rx.recv() => {
                let event = match event {
                    Ok(event) => event,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Streaming client lagged, skipped {} events", skipped);
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                };

                let payload = match event {
                    StreamBroadcast::Note(note) => {
                        serde_json::to_string(&shared::StreamEvent::Note { note }).ok()
                    }
                    StreamBroadcast::Notification { user_id: target, notification } => {
                        if target == user_id_str {
                            serde_json::to_string(&shared::StreamEvent::Notification { notification })
                                .ok()
                        } else {
                            None
                        }
                    }
                };

                if let Some(payload) = payload
                    && socket.send(Message::Text(payload.into())).await.is_err()
                {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    // チャンネル購読等のクライアントメッセージは現状すべて受理する
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

    debug!("Streaming connection closed for {}", user_id_str);
}
