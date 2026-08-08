# mithic: Axum backend + Leptos/Trunk frontend (wasm)
# Matches project rust-version >= 1.85; use recent bookworm rust image.
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

USER root
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        clang \
        mold \
        binaryen \
        python3 \
    && rm -rf /var/lib/apt/lists/*

USER vscode
RUN rustup target add wasm32-unknown-unknown \
    && rustup component add rustfmt clippy \
    && cargo install trunk --locked \
    && cargo install cargo-watch --locked || true
