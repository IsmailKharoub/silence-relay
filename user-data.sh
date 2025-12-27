#!/bin/bash
set -e

# Update system
yum update -y

# Install Docker
yum install -y docker git
systemctl start docker
systemctl enable docker
usermod -aG docker ec2-user

# Install Docker Compose
curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# Clone the repo and start the server
cd /home/ec2-user
git clone https://github.com/placeholder/silence.git || true

# Create docker-compose.yml directly
mkdir -p /home/ec2-user/relay-server
cat > /home/ec2-user/relay-server/docker-compose.yml << 'EOF'
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    restart: always
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 3

  relay:
    image: rust:1.75-slim
    restart: always
    ports:
      - "8080:8080"
    environment:
      - BIND_ADDR=0.0.0.0:8080
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=relay_server=info
    depends_on:
      redis:
        condition: service_healthy
    working_dir: /app
    volumes:
      - ./src:/app/src
      - ./Cargo.toml:/app/Cargo.toml
      - ./Cargo.lock:/app/Cargo.lock
      - cargo_cache:/usr/local/cargo/registry
    command: bash -c "cargo build --release && ./target/release/relay-server"

volumes:
  redis_data:
  cargo_cache:
EOF

# Create the relay server source files
mkdir -p /home/ec2-user/relay-server/src

# Cargo.toml
cat > /home/ec2-user/relay-server/Cargo.toml << 'EOF'
[package]
name = "relay-server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
tower = { version = "0.4", features = ["limit", "timeout"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio-tungstenite = "0.21"
futures = "0.3"
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
base64 = "0.21"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
thiserror = "1"
anyhow = "1"
EOF

chown -R ec2-user:ec2-user /home/ec2-user/relay-server

echo "Setup complete - relay server will be started manually"

