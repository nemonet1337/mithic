FROM ubuntu:24.04

# Avoid interactive tzdata prompt
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    nodejs \
    npm \
    python3 \
    python3-pip \
    libnss3 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libxkbcommon0 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libasound2 \
    libpango-1.0-0 \
    libcairo2 \
    libdbus-1-3 \
    libatspi2.0-0 \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
ENV CARGO_HOME=/root/.cargo
ENV RUSTUP_HOME=/root/.rustup

# Install Rust components
RUN rustup default stable && \
    rustup target add wasm32-unknown-unknown && \
    rustup component add rust-src

# Install Trunk for WASM frontend builds
RUN curl -L https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf - -C /usr/local/bin

# Install Playwright
WORKDIR /tmp/playwright
RUN npm install -g playwright && \
    playwright install-deps && \
    playwright install chromium firefox webkit

# Install Rust Token Killer
RUN cargo install --locked tokio-console && \
    cargo install --locked bacon

# Clean up
RUN rm -rf /tmp/playwright

# Set working directory
WORKDIR /workspace

# Copy initialization script
COPY .devcontainer/init.sh /usr/local/bin/init.sh
RUN chmod +x /usr/local/bin/init.sh

# Set environment variables
ENV CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=wasm-ld
ENV RUST_LOG=info

# Expose port for development server
EXPOSE 3000

# Default command
CMD ["/bin/bash"]