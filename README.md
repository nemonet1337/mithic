# mithic

Rust製 ActivityPub対応SNS

- **バックエンド**: Axum + SurrealDB + Dragonfly (Redis互換)
- **フロントエンド**: Leptos (CSR/WASM) + **DaisyUI v5** + Tailwind CSS v4
- **PWA (Progressive Web App)**: Workbox 7 による Service Worker + Web App Manifest（オフラインサポート、ホーム画面追加対応）
- **連合**: ActivityPub (WebFinger / NodeInfo / HTTP Signatures)

## Docker で動かす

```bash
# 1. 環境変数を用意 — JWT_SECRET は必須 (未設定・placeholder ではサーバーが起動拒否)
cp .env.example .env
# .env の JWT_SECRET を十分な長さのランダム文字列に書き換える

# 2. ビルドして起動 (server / worker / frontend / surrealdb / dragonfly)
docker compose up -d --build

# 3. 動作確認
curl http://localhost/api/v1/health      # => {"status":"ok"}
# ブラウザで http://localhost を開き、サインアップ → 投稿 → タイムライン表示
```

| サービス | ポート | 役割 |
|---|---|---|
| frontend (caddy) | 80 | WASM 配信 + `/api` リバースプロキシ + WebSocket + PWA静的ファイル |
| server | 3000 | REST API (`/api/v1/*`) + WebSocket (`/api/v1/streaming`) + ActivityPub |
| worker | - | 連合配送キュー (並列4 + 指数バックオフ + DLQ) |
| surrealdb | 8000 | データベース |
| dragonfly | 6379 | キャッシュ / 配送キュー |

## ローカル開発

```bash
# インフラのみ Docker で起動
docker compose up -d surrealdb dragonfly

# バックエンド (http://localhost:3000)
cargo run -p mithic-server
cargo run -p mithic-worker

# フロントエンド (http://localhost:1420, /api は :3000 へプロキシ)
# Tailwind CSS は Trunk の standalone CLI がビルド時に処理する (Node.js 不要)
cd frontend
trunk serve
```

SurrealDB を立てずに API だけ試す場合はメモリエンジンが使える:

```bash
SURREALDB_ENDPOINT=mem:// SURREALDB_POOL_SIZE=1 cargo run -p mithic-server
```

## 品質ゲート

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude frontend -- -D warnings
cargo test --workspace --exclude frontend
cargo check -p frontend --target wasm32-unknown-unknown
```

## ドキュメント

- 開発ガイドライン: `CLAUDE.md`
- ロードマップ / TODO: `TODO.md`

## ライセンス

AGPL-3.0

## DevContainer

VS Code DevContainerを使用して開発環境を構築できます。

### Podmanを使用する場合

```bash
# podman-composeのインストール
pip install podman-compose

# コンテナの起動
podman-compose -f .devcontainer/docker-compose.yml up -d --build
```

Podman固有の注意事項は `.devcontainer/PODMAN.md` を参照してください。
