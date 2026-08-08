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

# 2. ビルドして起動 (backend / frontend / surrealdb / dragonfly)
docker compose up -d --build

# 3. 動作確認
curl http://localhost:3000/api/v1/health      # => {"status":"ok"}
# ブラウザで http://localhost:3000 を開き、サインアップ → 投稿 → タイムライン表示
# (port 80 でも可: http://localhost )
```

| サービス | ポート | 役割 |
|---|---|---|
| frontend (caddy) | 80, 3000 | WASM 配信 + `/api` リバースプロキシ + WebSocket + PWA静的ファイル |
| backend | 3000 (内部) | REST API + WebSocket + ActivityPub + 連合配送ワーカー（同一プロセス）。ホストからは Caddy 経由 |
| surrealdb | 8000 | データベース |
| dragonfly | 6379 | キャッシュ / 配送キュー |

メディアはデフォルトでローカル FS（`media_files` ボリューム）。外部 S3 互換を使う場合は `.env` に `STORAGE_TYPE=s3` と `STORAGE_S3_*` を設定する。

### Windows + Podman で `ERR_CONNECTION_REFUSED` になる場合

Podman Machine (WSL) がコンテナの公開ポートを **WSL 側 IP には出すが Windows の `127.0.0.1` には転送しない**ことがあります。
コンテナ内ヘルスチェックは通っていても、ブラウザだけ `localhost:3000` に繋がらない状態です。

```powershell
# 一時回避: Windows localhost を WSL IP へ中継 (管理者権限不要)
python scripts/localhost_proxy.py
# 別ターミナルで確認
curl http://127.0.0.1:3000/api/v1/health
```

恒久対策の候補:
1. Podman Desktop で **User mode networking** を有効化してマシンを再作成 / 再起動する
2. `.wslconfig` で `networkingMode=mirrored` を試す（要 `wsl --shutdown`）
3. 上記プロキシを開発時に併用する

## ローカル開発

```bash
# インフラのみ Docker で起動
docker compose up -d surrealdb dragonfly

# バックエンド (http://localhost:3000) — HTTP + 連合配送ワーカー
cargo run -p mithic-server

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
