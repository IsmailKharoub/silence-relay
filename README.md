# SecureChat Relay Server

WebSocket relay server for storing and forwarding encrypted messages when P2P connections aren't possible.

## Overview

This is a **dumb relay** - it only sees encrypted blobs and never decrypts messages. Zero-knowledge architecture.

## Requirements

- Rust 1.75+
- Redis 7+
- Docker (optional)

## Development

### Local Setup

```bash
# Start Redis
docker run -d -p 6379:6379 redis:7-alpine

# Run server
cargo run

# Or with environment variables
RUST_LOG=debug REDIS_URL=redis://localhost:6379 cargo run
```

### Docker

```bash
# Start everything
docker-compose up

# Just Redis
docker-compose up redis
```

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `BIND_ADDR` | `0.0.0.0:8080` | Server bind address |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection URL |
| `MESSAGE_TTL_SECS` | `86400` | Message TTL (24h default) |
| `RUST_LOG` | `info` | Log level |

## API

### WebSocket

Connect to `/ws/{user_id}` to establish a relay connection.

**Send Message:**
```json
{
  "to": "recipient_user_id",
  "payload": "base64_encrypted_blob"
}
```

**Receive Message:**
```json
{
  "messageId": "uuid",
  "from": "sender_user_id",
  "to": "your_user_id",
  "payload": "base64_encrypted_blob",
  "timestamp": 1234567890
}
```

**Delivery Receipt:**
```json
{
  "messageId": "uuid",
  "status": "delivered",
  "timestamp": 1234567890
}
```

### HTTP

- `GET /health` - Health check

## Testing

```bash
cargo test
```

## Building

```bash
cargo build --release
```

