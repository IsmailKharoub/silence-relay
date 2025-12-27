//! Core relay logic and state management

use crate::config::Config;
use crate::storage::RedisStorage;
use crate::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Message envelope - we only see encrypted blobs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageEnvelope {
    pub message_id: String,
    pub from: String,
    pub to: String,
    pub payload: String, // Base64 encoded encrypted blob
    pub timestamp: i64,
}

/// Delivery receipt
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeliveryReceipt {
    pub message_id: String,
    pub status: DeliveryStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Read,
    Failed,
}

/// Active connection handle
pub type ConnectionTx = mpsc::Sender<String>;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub storage: RedisStorage,
    pub connections: Arc<RwLock<HashMap<String, ConnectionTx>>>,
    pub config: Config,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self> {
        let storage = RedisStorage::new(&config.redis_url).await?;
        
        Ok(Self {
            storage,
            connections: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
        })
    }

    /// Register a new connection
    pub async fn register_connection(&self, user_id: &str, tx: ConnectionTx) {
        let mut connections = self.connections.write().await;
        connections.insert(user_id.to_string(), tx);
        tracing::info!(user_id = %user_id, "Connection registered");
    }

    /// Remove a connection
    pub async fn remove_connection(&self, user_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(user_id);
        tracing::info!(user_id = %user_id, "Connection removed");
    }

    /// Route a message to recipient (online) or queue (offline)
    pub async fn route_message(&self, envelope: MessageEnvelope) -> Result<()> {
        let connections = self.connections.read().await;
        
        if let Some(tx) = connections.get(&envelope.to) {
            // Recipient is online - send directly
            let msg = serde_json::to_string(&envelope)?;
            if tx.send(msg).await.is_err() {
                // Connection dead, queue instead
                drop(connections);
                self.storage.queue_message(&envelope).await?;
            }
            tracing::debug!(
                message_id = %envelope.message_id,
                to = %envelope.to,
                "Message delivered directly"
            );
        } else {
            // Recipient offline - queue for later
            drop(connections);
            self.storage.queue_message(&envelope).await?;
            tracing::debug!(
                message_id = %envelope.message_id,
                to = %envelope.to,
                "Message queued for offline delivery"
            );
        }

        Ok(())
    }

    /// Fetch pending messages for a user
    pub async fn fetch_pending(&self, user_id: &str) -> Result<Vec<MessageEnvelope>> {
        self.storage.fetch_pending(user_id).await
    }
}

