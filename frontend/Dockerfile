# =============================================================================
# Stage 1: cargo-chef + mold (shared base)
# =============================================================================
FROM rust:1.99-bookworm AS chef
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    mold \
    pkg-config \
    ca-certificates \
    curl \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*
# ソースから cargo install すると数分かかるので公式バイナリを使う
RUN curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/LukeMathWalker/cargo-chef/releases/download/v0.1.78/cargo-chef-installer.sh | sh
ENV CARGO_INCREMENTAL=0
ENV CARGO_TERM_COLOR=never
ENV CARGO_TARGET_DIR=/app/target
# コンテナ再ビルドでは thin LTO のコストが大きい。opt-level=3 + strip は残す
ENV CARGO_PROFILE_RELEASE_LTO=false
WORKDIR /app

# =============================================================================
# Stage 2: Analyze workspace and generate dependency recipe
# =============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3: Pre-compile frontend WASM dependencies
# =============================================================================
FROM chef AS frontend-deps
RUN rustup target add wasm32-unknown-unknown
RUN curl -sSLf \
    https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xzf - -C /usr/local/bin
COPY --from=planner /app/.cargo .cargo
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=shared \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=shared \
    --mount=type=cache,id=frontend-target,target=/app/target,sharing=shared \
    cargo chef cook --release \
    --package frontend \
    --target wasm32-unknown-unknown \
    --recipe-path recipe.json

# =============================================================================
# Stage 4: Build frontend WASM with Trunk
# Tailwind CSS v4 is built by Trunk's standalone CLI (no Node.js / package.json)
# =============================================================================
FROM frontend-deps AS frontend-builder
COPY . .
WORKDIR /app/frontend
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=shared \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=shared \
    --mount=type=cache,id=frontend-target,target=/app/target,sharing=shared \
    --mount=type=cache,id=trunk-cache,target=/root/.cache/trunk,sharing=shared \
    trunk build --release

# =============================================================================
# Stage 5: dist — Web サーバーは持たない（compose の frontend サービスが
# /dist を frontend_dist ボリュームへ同期し、nginx が配信する）
# NOTE: ビルダーを musl/Alpine にはしない。Trunk の Tailwind CLI は
#       tailwindcss-linux-x64（glibc 固定）を取得するため musl 上では動かない。
# =============================================================================
FROM alpine:3.24 AS dist
# Flatten public/ into dist root (Trunk nests public/ inside dist/)
COPY --from=frontend-builder /app/frontend/dist /dist
RUN find /dist -mindepth 2 -exec mv -t /dist {} + 2>/dev/null || true
RUN rm -rf /dist/public
