# mithic

Rust製 Misskey互換 APub対応SNS

- **バックエンド**: Axum + SurrealDB + Dragonfly (Redis互換)
- **フロントエンド**: Leptos (CSR/WASM) + Tailwind CSS
- **連合**: ActivityPub (WebFinger / NodeInfo / HTTP Signatures)

## Docker で動かす

```bash
# 1. 環境変数を用意 (そのままでも起動可。本番では JWT_SECRET 等を必ず変更)
cp .env.example .env

# 2. ビルドして起動 (server / worker / frontend / surrealdb / dragonfly)
docker compose up -d --build

# 3. 動作確認
curl http://localhost/api/v1/health      # => {"status":"ok"}
# ブラウザで http://localhost を開き、サインアップ → 投稿 → タイムライン表示
```

| サービス | ポート | 役割 |
|---|---|---|
| frontend (nginx) | 80 | WASM 配信 + `/api` リバースプロキシ + WebSocket |
| server | 3000 | REST API + `/api/streaming` WebSocket + ActivityPub |
| worker | - | 連合配送キュー (並列4ワーカー + リトライスケジューラ) |
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
cd frontend-web && trunk serve
```

SurrealDB を立てずに API だけ試す場合はメモリエンジンが使える:

```bash
SURREALDB_ENDPOINT=mem:// SURREALDB_POOL_SIZE=1 cargo run -p mithic-server
```

## 品質ゲート

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude frontend-web -- -D warnings
cargo test --workspace --exclude frontend-web
cargo check -p frontend-web --target wasm32-unknown-unknown
```

## ドキュメント

- 開発ガイドライン: `CLAUDE.md`
- ロードマップ / TODO: `TODO.md`
- 機能ギャップ台帳: `docs/feature-gap-analysis.md`

## ライセンス

AGPL-3.0
