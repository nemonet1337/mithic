# =============================================================================
# Stage 1: cargo-chef installer (shared base for dep caching)
# =============================================================================
FROM rust:1.85-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# =============================================================================
# Stage 2: Analyze workspace and generate dependency recipe
# =============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3: Pre-compile backend dependencies (cached Docker layer)
# =============================================================================
FROM chef AS backend-deps
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release \
    --package mithic-server \
    --package mithic-worker \
    --recipe-path recipe.json

# =============================================================================
# Stage 4: Build backend binaries (server + worker)
# =============================================================================
FROM backend-deps AS backend-builder
COPY . .
RUN cargo build --release \
    --package mithic-server \
    --package mithic-worker

# =============================================================================
# Stage 5: Build frontend WASM with Trunk
# =============================================================================
FROM rust:1.85-bookworm AS frontend-builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown

# Install Trunk binary (much faster than cargo install)
RUN curl -sSLf \
    https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xzf - -C /usr/local/bin

WORKDIR /app
COPY . .
WORKDIR /app/crates/frontend-web

# trunk build downloads wasm-bindgen, wasm-opt, and Tailwind CLI at build time
RUN trunk build --release

# =============================================================================
# Stage 6: Server runtime image
# =============================================================================
FROM debian:bookworm-slim AS server
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/target/release/mithic-server /usr/local/bin/

EXPOSE 3000
ENTRYPOINT ["mithic-server"]

# =============================================================================
# Stage 7: Worker runtime image
# =============================================================================
FROM debian:bookworm-slim AS worker
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/target/release/mithic-worker /usr/local/bin/

ENTRYPOINT ["mithic-worker"]

# =============================================================================
# Stage 8: Frontend — nginx serving WASM dist
# =============================================================================
FROM nginx:alpine AS frontend
COPY --from=frontend-builder /app/crates/frontend-web/dist /usr/share/nginx/html
COPY infra/nginx/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
