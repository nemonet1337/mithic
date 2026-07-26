# =============================================================================
# Stage 1: cargo-chef installer (shared base for dep caching)
# =============================================================================
FROM rust:1.94-bookworm AS chef
RUN apt-get update && apt-get install -y clang mold && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
ENV CARGO_BUILD_JOBS=4
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
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo chef cook --release \
    --package mithic-server \
    --package mithic-worker \
    --recipe-path recipe.json

# =============================================================================
# Stage 4: Build backend binaries (server + worker)
# =============================================================================
FROM backend-deps AS backend-builder
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release \
    --package mithic-server \
    --package mithic-worker && \
    cp /app/target/release/mithic-server /app/mithic-server && \
    cp /app/target/release/mithic-worker /app/mithic-worker

# =============================================================================
# Stage 5: Build frontend WASM with Trunk
# Tailwind CSS v4 is built by Trunk's standalone CLI (no Node.js / package.json)
# =============================================================================
FROM rust:1.94-bookworm AS frontend-builder
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    clang \
    mold \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown

# Install Trunk binary (much faster than cargo install)
RUN curl -sSLf \
    https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xzf - -C /usr/local/bin

WORKDIR /app
COPY . .
WORKDIR /app/frontend-web

# trunk build downloads wasm-bindgen, wasm-opt, and Tailwind CLI at build time
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    trunk build --release

# =============================================================================
# Stage 6: Server runtime image
# =============================================================================
FROM debian:bookworm-slim AS server
# curl はコンテナの healthcheck に必要
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /app/mithic-server /usr/local/bin/

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

COPY --from=backend-builder /app/mithic-worker /usr/local/bin/

ENTRYPOINT ["mithic-worker"]

# =============================================================================
# Stage 8: Frontend — caddy serving WASM dist
# =============================================================================
FROM caddy:alpine AS frontend
COPY --from=frontend-builder /app/frontend-web/dist /usr/share/caddy
# Flatten public/ into dist root (Trunk nests public/ inside dist/)
RUN find /usr/share/caddy -mindepth 2 -exec mv -t /usr/share/caddy {} + 2>/dev/null || true
# Clean up empty public/ directory if it exists
RUN rm -rf /usr/share/caddy/public
COPY Caddyfile /etc/caddy/Caddyfile
EXPOSE 80
EXPOSE 443
EXPOSE 3000
