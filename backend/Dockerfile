# =============================================================================
# Stage 1: cargo-chef base (Alpine/musl)
# rust:*-alpine は host target が x86_64-unknown-linux-musl（--target 不要）。
# 出力は target/release/（musl が host なので glibc 版と同じパス）。
# =============================================================================
FROM rust:1.99-alpine AS chef
# cmake は aws-lc-sys 0.45.0 の C ビルド、perl は ring 0.17 の asm 生成に必要
RUN apk add --no-cache build-base pkgconf cmake perl ca-certificates curl
# ソースから cargo install すると数分かかるので公式バイナリを使う
# （インストーラーは host target に合う musl バイナリを自動選択する）
# パイプに流すと curl の失敗が消えて「cargo chef がない」状態で
# 後続ステージが落ちるので、ファイルに落としてから実行する。
# GitHub への取得は一時的にタイムアウトすることがあるため
# connect-timeout + リトライで粘る（初回接続だけ数十秒ハングする）
RUN curl --proto '=https' --tlsv1.2 -LsSf \
        --connect-timeout 30 --retry 8 --retry-all-errors --retry-delay 3 \
        -o /tmp/cargo-chef-installer.sh \
        https://github.com/LukeMathWalker/cargo-chef/releases/download/v0.1.78/cargo-chef-installer.sh \
    && sh /tmp/cargo-chef-installer.sh \
    && rm -f /tmp/cargo-chef-installer.sh
ENV CARGO_INCREMENTAL=0
ENV CARGO_TERM_COLOR=never
# crates.io への取得も同様に不安定なことがあるため多めにリトライする
ENV CARGO_NET_RETRY=10
ENV CARGO_TARGET_DIR=/app/target
# コンテナ再ビルドでは thin LTO のコストが大きい。opt-level=3 + strip は残す
ENV CARGO_PROFILE_RELEASE_LTO=false
WORKDIR /app
# mold は外す: Alpine にパッケージがなく、musl との組合わせも未検証。
# .cargo/config.toml の clang+mold は gnu 向けで musl ビルドでは無効
# （ローカル/CI の glibc ビルドは据え置き）。

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
COPY --from=planner /app/.cargo .cargo
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=shared \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=shared \
    --mount=type=cache,target=/app/target,sharing=shared \
    cargo chef cook --release \
    --package mithic-server \
    --recipe-path recipe.json

# =============================================================================
# Stage 4: Build backend binary (HTTP + federation delivery worker)
# =============================================================================
FROM backend-deps AS backend-builder
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=shared \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=shared \
    --mount=type=cache,target=/app/target,sharing=shared \
    cargo build --release --package mithic-server && \
    cp /app/target/release/mithic-server /app/mithic-server

# =============================================================================
# Stage 5: Backend runtime image (API + delivery worker in one process)
# =============================================================================
FROM alpine:3.24 AS backend
# curl は healthcheck 用、ca-certificates は TLS（連合・SurrealDB）用、
# libgcc は musl バイナリの実行に必要
RUN apk add --no-cache ca-certificates curl libgcc

COPY --from=backend-builder /app/mithic-server /usr/local/bin/

EXPOSE 3000
ENTRYPOINT ["mithic-server"]
