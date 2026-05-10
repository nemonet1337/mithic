FROM rust:1.75-slim AS builder

WORKDIR /app

# 依存関係のキャッシュ
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# ソースコードコピーとビルド
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# 実行用イメージ
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/mithic-backend /app/mithic-backend

ENV RUST_LOG=info

EXPOSE 3000

CMD ["./mithic-backend"]
