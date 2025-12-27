//! Error types for the relay server

use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid user ID")]
    InvalidUserId,

    #[error("Connection closed")]
    ConnectionClosed,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match &self {
            Error::InvalidUserId => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

