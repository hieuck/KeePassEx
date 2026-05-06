//! WebSocket handler for real-time vault sync notifications
//!
//! Protocol:
//! - Client connects: GET /ws?token=<jwt>
//! - Server sends: {"type":"connected","user_id":"...","version":3}
//! - On vault upload: server broadcasts {"type":"vault_updated","version":4}
//! - Client responds: {"type":"pull"} → client fetches new vault via REST
//! - Ping/pong: server sends {"type":"ping"} every 30s, client replies {"type":"pong"}

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use crate::auth::validate_token;
use crate::AppState;

/// Query parameters for WebSocket connection
#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

/// Messages sent from server to client
#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Connected { user_id: String, version: u32 },
    VaultUpdated { version: u32, uploaded_at: String },
    Ping,
    Error { message: String },
}

/// Messages sent from client to server
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Pong,
    Pull,
    Subscribe { user_id: String },
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, query.token, state))
}

async fn handle_socket(mut socket: WebSocket, token: String, state: AppState) {
    // Validate JWT
    let claims = match validate_token(&token, &state.config) {
        Ok(c) if c.token_type == "access" => c,
        _ => {
            let _ = socket
                .send(Message::Text(
                    serde_json::to_string(&ServerMessage::Error {
                        message: "Unauthorized".to_string(),
                    })
                    .unwrap_or_default(),
                ))
                .await;
            return;
        }
    };

    let user_id = claims.sub.clone();
    tracing::info!("WebSocket connected: user={}", user_id);

    // Get current vault version
    let current_version = state
        .db
        .get_vault_meta(&user_id)
        .await
        .ok()
        .flatten()
        .map(|m| m.version)
        .unwrap_or(0);

    // Send connected message
    let connected_msg = serde_json::to_string(&ServerMessage::Connected {
        user_id: user_id.clone(),
        version: current_version,
    })
    .unwrap_or_default();

    if socket.send(Message::Text(connected_msg)).await.is_err() {
        return;
    }

    // Main message loop
    let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            // Incoming message from client
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            match client_msg {
                                ClientMessage::Pong => {
                                    // Client is alive
                                }
                                ClientMessage::Pull => {
                                    // Client wants to know current version
                                    if let Ok(Some(meta)) = state.db.get_vault_meta(&user_id).await {
                                        let msg = serde_json::to_string(&ServerMessage::VaultUpdated {
                                            version: meta.version,
                                            uploaded_at: meta.uploaded_at.to_rfc3339(),
                                        }).unwrap_or_default();
                                        let _ = socket.send(Message::Text(msg)).await;
                                    }
                                }
                                ClientMessage::Subscribe { .. } => {
                                    // Already subscribed on connect
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("WebSocket disconnected: user={}", user_id);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }

            // Periodic ping
            _ = ping_interval.tick() => {
                let ping = serde_json::to_string(&ServerMessage::Ping).unwrap_or_default();
                if socket.send(Message::Text(ping)).await.is_err() {
                    break;
                }
            }
        }
    }
}
