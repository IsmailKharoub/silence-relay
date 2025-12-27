//! Configuration for the relay server

use crate::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub redis_url: String,
    pub message_ttl_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind_addr: std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            message_ttl_secs: std::env::var("MESSAGE_TTL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(86400), // 24 hours default
        })
    }
}

