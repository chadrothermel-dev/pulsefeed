# Stage 1: Build React + TypeScript UI
FROM node:22-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build optimized Rust binary (with bundled SQLite FTS5)
FROM rust:latest AS backend-builder
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release

# Stage 3: Minimal production runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=backend-builder /app/backend/target/release/pulsefeed /app/pulsefeed
COPY --from=frontend-builder /app/frontend/dist /app/dist

RUN mkdir -p /data
VOLUME ["/data"]

ENV PULSEFEED_HOST=0.0.0.0
ENV PULSEFEED_PORT=8080
ENV PULSEFEED_DB=/data/pulsefeed.db
ENV PULSEFEED_STATIC_DIR=/app/dist

EXPOSE 8080
CMD ["/app/pulsefeed"]
