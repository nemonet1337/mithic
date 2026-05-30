FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# OpenSSLのビルド等に必要なシステムパッケージのインストール
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends pkg-config libssl-dev

# Rust ターゲットの追加 (フロントエンド WebAssembly 用)
RUN rustup target add wasm32-unknown-unknown

# Trunk などのツールのインストールを高速化するため cargo-binstall を導入
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Trunkのインストール
RUN cargo binstall --no-confirm trunk

# Node.js LTS のインストール (nodesourceを使用)
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash - \
    && apt-get install -y nodejs

