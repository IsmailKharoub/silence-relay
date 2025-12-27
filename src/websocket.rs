//! WebSocket connection handling

use crate::relay::{AppState, DeliveryReceipt, MessageEnvelope};
use crate::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Response {
    ws.on_upgrade(move |socket| handle_connection(socket, user_id, state))
}

/// Handle a WebSocket connection
async fn handle_connection(socket: WebSocket, user_id: String, state: AppState) {
    tracing::info!(user_id = %user_id, "New WebSocket connection");

    let (mut ws_tx, mut ws_rx) = socket.split();
    
    // Channel for sending messages to this connection
    let (tx, mut rx) = mpsc::channel::<String>(100);
    
    // Register connection
    state.register_connection(&user_id, tx).await;

    // Fetch and send pending messages
    if let Ok(pending) = state.fetch_pending(&user_id).await {
        for envelope in pending {
            if let Ok(msg) = serde_json::to_string(&envelope) {
                if ws_tx.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        }
    }

    // Spawn task to forward messages from channel to WebSocket
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let user_id_clone = user_id.clone();
    let state_clone = state.clone();
    
    while let Some(msg) = ws_rx.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_message(&text, &user_id_clone, &state_clone).await {
                    tracing::error!(error = %e, "Failed to handle message");
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!(user_id = %user_id_clone, "Client closed connection");
                break;
            }
            Ok(Message::Ping(_)) => {
                // Pong is handled automatically by axum
                tracing::trace!("Received ping");
            }
            Err(e) => {
                tracing::error!(error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    state.remove_connection(&user_id).await;
    forward_task.abort();
    
    tracing::info!(user_id = %user_id, "Connection closed");
}

/// Handle an incoming message
async fn handle_message(text: &str, from_user: &str, state: &AppState) -> Result<()> {
    // Try to parse as message envelope
    if let Ok(mut envelope) = serde_json::from_str::<MessageEnvelope>(text) {
        // Ensure the 'from' field matches the authenticated user
        envelope.from = from_user.to_string();
        envelope.timestamp = chrono::Utc::now().timestamp_millis();
        
        if envelope.message_id.is_empty() {
            envelope.message_id = uuid::Uuid::new_v4().to_string();
        }
        
        // Route the message
        state.route_message(envelope).await?;
        return Ok(());
    }

    // Try to parse as delivery receipt
    if let Ok(receipt) = serde_json::from_str::<DeliveryReceipt>(text) {
        tracing::debug!(
            message_id = %receipt.message_id,
            status = ?receipt.status,
            "Received delivery receipt"
        );
        // Future: Forward receipt to sender
        return Ok(());
    }

    tracing::warn!("Unknown message format: {}", text);
    Ok(())
}

