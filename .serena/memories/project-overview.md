# Mithic プロジェクト概要

ActivityPub 連合 SNS。Rust edition 2024。

## クレート
- `backend/api` — Axum 0.8 `/api/v1/*` ネイティブ REST + AP/OGP
- `backend/federation` — ActivityPub 配送 + `http_sig` (RSA-SHA256)
- `backend/core` — モデル / JWT+Argon2+TOTP
- `backend/server` — `mithic-server` (HTTP + 連合配送)
- `db` — SurrealDB 3 + Dragonfly
- `shared` — front↔back DTO
- `frontend` — Leptos 0.7 CSR/WASM + Tailwind 4

## しないこと
- Misskey / Mastodon クライアント互換 API
- 推測モデル先置き
- 依存の無暴追加

## TODO 推奨順 (ソース: TODO.md)
1. ノート作成時の poll 永続化
2. フロント: ピン / ブロック / ミュートを API 接続
3. Web Push / サムネの E2E
4. DM: やるか捨てるか

AP inbox Delete/Update/Accept/Reject/Block は 2026-08-25 実装 (`mem:ap-inbox`)。
