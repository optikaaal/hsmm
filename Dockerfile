# Multi-stage build for Hytale Server Mod Manager
# Stage 1: Build the frontend
FROM node:20-slim AS frontend-builder

WORKDIR /build/frontend

# Copy frontend package files
COPY frontend/package.json frontend/package-lock.json ./

# Install frontend dependencies
RUN npm ci

# Copy frontend source
COPY frontend/ ./

# Build frontend
RUN npm run build

# Stage 2: Build the Rust binaries
FROM rust:1.83-slim AS rust-builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests and source code
COPY Cargo.toml ./
COPY src ./src

# Build both binaries (CLI and web)
RUN cargo build --release --bin hsmm && \
    cargo build --release --bin hsmm-web

# Strip the binaries to reduce size
RUN strip /build/target/release/hsmm && \
    strip /build/target/release/hsmm-web

# Stage 3: Final image - Extend the IndifferentBroccoli Hytale server image
FROM indifferentbroccoli/hytale-server-docker:latest

# Switch to root to install components
USER root

# Copy the hsmm and hsmm-web binaries from rust-builder
COPY --from=rust-builder /build/target/release/hsmm /usr/local/bin/hsmm
COPY --from=rust-builder /build/target/release/hsmm-web /usr/local/bin/hsmm-web
RUN chmod +x /usr/local/bin/hsmm /usr/local/bin/hsmm-web

# Copy the frontend build from frontend-builder
COPY --from=frontend-builder /build/dist/web /home/hytale/web-ui

# Backup the original entrypoint
RUN cp /home/hytale/server/init.sh /home/hytale/server/init.sh.original

# Copy our mod manager wrapper
COPY scripts/mod-manager-init.sh /home/hytale/server/init.sh
RUN chmod +x /home/hytale/server/init.sh

# Set ownership
RUN chown -R hytale:hytale /usr/local/bin/hsmm /usr/local/bin/hsmm-web /home/hytale/server/init.sh /home/hytale/web-ui

# Additional environment variables for mod manager
ENV CURSEFORGE_API_KEY="" \
    CONFIG_PATH=/home/hytale/server-files/mods.toml \
    MODS_DIR=/home/hytale/server-files/mods \
    MOD_MANAGER_LOG=/home/hytale/server-files/logs/mod-manager.log \
    WEB_UI_PORT=8080 \
    STATIC_DIR=/home/hytale/web-ui

# Expose the web UI port in addition to Hytale server port
EXPOSE 8080

# Inherits all other settings from base image:
# - USER hytale
# - WORKDIR /home/hytale/server
# - VOLUME /home/hytale/server-files
# - ENV variables (SERVER_NAME, MAX_PLAYERS, etc.)
# - HEALTHCHECK
# - ENTRYPOINT /home/hytale/server/init.sh
