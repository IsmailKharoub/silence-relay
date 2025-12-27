//! SecureChat Relay Server
//! 
//! A minimal WebSocket relay for storing and forwarding encrypted messages
//! when P2P connections aren't possible. Zero-knowledge: we never see plaintext.

mod config;
mod error;
mod relay;
mod storage;
mod websocket;

use axum::{routing::get, Router};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use error::{Error, Result};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "relay_server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;
    
    tracing::info!("Starting relay server on {}", config.bind_addr);

    // Initialize state
    let state = relay::AppState::new(&config).await?;

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws/:user_id", get(websocket::ws_handler))
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!("Relay server listening on {}", config.bind_addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
