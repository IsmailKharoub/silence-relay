//! Redis storage for message queuing

use crate::relay::MessageEnvelope;
use crate::Result;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RedisStorage {
    client: redis::Client,
}

impl RedisStorage {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        
        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        
        tracing::info!("Connected to Redis");
        
        Ok(Self { client })
    }

    /// Queue a message for offline delivery
    pub async fn queue_message(&self, envelope: &MessageEnvelope) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("pending:{}", envelope.to);
        let value = serde_json::to_string(envelope)?;
        
        // Push to list and set TTL (24 hours)
        conn.rpush::<_, _, ()>(&key, &value).await?;
        conn.expire::<_, ()>(&key, 86400).await?;
        
        tracing::debug!(
            message_id = %envelope.message_id,
            to = %envelope.to,
            "Message queued in Redis"
        );
        
        Ok(())
    }

    /// Fetch and clear pending messages for a user
    pub async fn fetch_pending(&self, user_id: &str) -> Result<Vec<MessageEnvelope>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("pending:{}", user_id);
        
        // Get all pending messages
        let messages: Vec<String> = conn.lrange(&key, 0, -1).await?;
        
        // Clear the queue
        if !messages.is_empty() {
            conn.del::<_, ()>(&key).await?;
        }
        
        // Parse messages
        let envelopes: Vec<MessageEnvelope> = messages
            .into_iter()
            .filter_map(|m| serde_json::from_str(&m).ok())
            .collect();
        
        tracing::debug!(
            user_id = %user_id,
            count = envelopes.len(),
            "Fetched pending messages"
        );
        
        Ok(envelopes)
    }

    /// Acknowledge message delivery (remove from any backup queues)
    pub async fn ack_message(&self, _message_id: &str) -> Result<()> {
        // For MVP, messages are removed on fetch
        // Future: implement proper ack with backup queue
        Ok(())
    }
}

