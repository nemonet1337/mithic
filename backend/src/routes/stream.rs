//! WebSocket Streaming Endpoint
//!
//! Provides WebSocket endpoint for real-time streaming.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::{
    state::{AppState, AuthUser},
    stream::{ClientMessage, StreamConnection, StreamMessage},
};

/// WebSocket handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    auth_user: Option<AuthUser>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, auth_user, state))
}

/// Handle WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    auth_user: Option<AuthUser>,
    state: AppState,
) {
    let user_id = auth_user.map(|u| u.user_id.to_string());
    
    info!("WebSocket connection established for user {:?}", user_id);

    let (mut sender, mut receiver) = socket.split();

    // Create channel for sending messages to client
    let (tx, mut rx) = mpsc::unbounded_channel::<StreamMessage>();

    // Create stream connection
    let connection = Arc::new(StreamConnection::new(
        user_id.clone(),
        tx.clone(),
        state.dragonfly().clone(),
    ));

    // Task to send messages to client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap_or_default();
            if let Err(e) = sender.send(Message::Text(json)).await {
                error!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Task to receive messages from client
    let conn_clone = connection.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
                        conn_clone.handle_message(msg).await;
                    } else {
                        warn!("Failed to parse client message: {}", text);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Client closed connection");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    // Pong is handled automatically by axum
                    info!("Received ping");
                }
                Ok(Message::Pong(data)) => {
                    info!("Received pong");
                }
                Ok(Message::Binary(_)) => {
                    warn!("Binary messages not supported");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }

    // Cleanup
    connection.dispose().await;
    info!("WebSocket connection closed for user {:?}", user_id);
}

/// Streaming channels endpoint (for compatibility with Misskey API)
pub async fn streaming_channels() -> &'static str {
    r#"{
        "channels": [
            "homeTimeline",
            "globalTimeline",
            "hashtag",
            "main",
            "admin",
            "queueStats",
            "serverStats"
        ]
    }"#
}
