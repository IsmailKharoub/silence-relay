# Build stage
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/relay-server /app/relay-server

ENV BIND_ADDR=0.0.0.0:8080
ENV RUST_LOG=relay_server=info

EXPOSE 8080

CMD ["/app/relay-server"]

