#!/bin/bash
set -e

echo "Initializing Mithic development environment..."

# Fetch cargo dependencies
echo "Fetching Rust dependencies..."
cargo fetch

# Install wasm-bindgen-cli if not present
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli --locked
fi

# Install cargo-watch for development
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch --locked
fi

# Install cargo-edit for easier dependency management
if ! command -v cargo-edit &> /dev/null; then
    echo "Installing cargo-edit..."
    cargo install cargo-edit --locked
fi

echo "DevContainer initialization complete!"
echo ""
echo "Available commands:"
echo "  - cargo run -p mithic-server    # Start the backend server"
echo "  - cargo run -p mithic-worker    # Start the background worker"
echo "  - cd frontend-web && trunk serve # Start the frontend dev server"
echo "  - playwright test               # Run E2E tests"
echo "  - bacon                         # Run tests with bacon TUI"