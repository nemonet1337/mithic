#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f .env && -f .env.example ]]; then
  cp .env.example .env
  echo "Created .env from .env.example — set JWT_SECRET before running the server."
fi

# Warm the dependency graph (no full release build)
cargo fetch
rustup show
echo "mithic devcontainer ready."
echo "  Infra:  docker compose up -d surrealdb dragonfly"
echo "  API:    cargo run -p mithic-server"
echo "  UI:     cd frontend && trunk serve"
