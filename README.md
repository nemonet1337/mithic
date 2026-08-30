# Mithic

**Rust 製の ActivityPub 対応 SNS。** WebUI は Leptos の WASM クライアントで、ブラウザからホーム画面に追加できる PWA でもある。

連合の外向けは ActivityPub だけ。クライアント向け API は Mithic 専用の `/api/v1/*` で、Misskey / Mastodon 互換は持たない。

> タグライン（マニフェストより）: *signal not noise*

---

## これは何か

Mithic は、小さなインスタンスを自分で立てて、他の Fediverse とつながるための SNS である。タイムライン、ノート、リアクション、ドライブ、通知といった「普通の分散 SNS」に必要な一式を、バックエンドからフロントまで Rust で組んでいる。

方針は次のとおり。

- **自前の WebUI が第一級クライアント。** サードパーティの Misskey / Mastodon アプリは対象外。
- **連合は ActivityPub のみ。** 受信 inbox は HTTP Signature 必須。Misskey 拡張（絵文字リアクション `_misskey_reaction`、引用 `quoteUrl`）は受け取る。
- **本文は Markdown。** パーサは comrak。ActivityPub の `content` は HTML、`source` に原文を載せる。
- **作らないものを先に決める。** アンテナ、クリップ、OAuth、クライアント互換 API などは意図的に持たない。ロードマップは [`TODO.md`](TODO.md)。

UI は paper / ink / accent のハードシャドウ基調。デスクトップは浮遊クローム、狭い画面では下部ドック。ライト / ダーク切替あり。

---

## できること

| 領域 | 内容 |
|---|---|
| アカウント | サインアップ / ログイン、プロフィール編集、パスワード変更、JWT 失効 |
| ノート | 投稿・返信・引用・リノート・削除。公開範囲（public / home / followers / specified）。CW。ファイル添付（最大 4、32MB） |
| 反応 | 絵文字リアクション、お気に入り、ピン留め API |
| タイムライン | ホーム / ローカル / グローバル / ハッシュタグ。トレンドタグ |
| 関係 | フォロー（承認待ち含む）/ ブロック / ミュート。ユーザー検索・おすすめ |
| ドライブ | アップロード、URL 取込、WebP サムネイル、`/uploads/{hash}` 配信 |
| 通知 | アプリ内通知、WebSocket による即時反映。任意で Web Push（VAPID 設定時） |
| インスタンス | メタ情報、公開カスタム絵文字、管理 API（停止 / 解除 / 削除、リレー CRUD） |
| PWA | Service Worker（Workbox 7）、オフラインページ、ホーム画面追加 |

投稿は Markdown。ハッシュタグ・メンション・絵文字は本文から抽出する。

画面としてはホーム / ローカル / グローバル、ノート詳細、通知、検索、プロフィール、設定（プロフィール / 通知 / テーマ）、ドライブ、ログイン / サインアップがある。DM と管理コンソールはルートだけあり、中身はまだプレースホルダ。

---

## 連合

他インスタンスとのやりとりは ActivityPub。Mithic の WebUI 以外からは、Fediverse 上の Person アクターとして見える。

**公開するエンドポイント**

- WebFinger (`/.well-known/webfinger`)
- NodeInfo
- Actor (`/users/{username}`)、outbox / followers / following / featured
- inbox（ユーザー inbox と shared inbox）。HTTP Signature (RSA-SHA256) 必須

**扱う Activity**

Create, Delete, Follow, Undo, Accept, Reject, Announce, Like, Update, Block。Question / 投票系も受信する。配送は Dragonfly 上のキュー（apalis）に載せ、署名付き POST を指数バックオフで再試行する。ホスト単位の並列制限と鍵キャッシュあり。

**やらないこと**

- Misskey / Mastodon クライアント互換 REST
- Misskey 風 WebSocket チャンネル抽象
- 複数 `mithic-server` 間でのストリーム共有（現状はプロセス内 broadcast）

OGP 用に `/notes/{id}` と `/profile/{username}` をボット向けに返す（Discord / Twitter 等のクローラは Caddy がバックエンドへ渡す）。

---

## アーキテクチャ

```
  ブラウザ (Leptos CSR / WASM + PWA)
           │  REST / WebSocket
           ▼
  Caddy  — 静的 WASM 配信 + /api・AP・WS をプロキシ
           │
           ▼
  mithic-server  ── HTTP (Axum) と連合配送ワーカーを同一プロセス
           │
     ┌─────┴──────┐
     ▼            ▼
  SurrealDB    Dragonfly
  (永続化)     (キャッシュ / レート制限 / 配送キュー)
     │
     └─ メディア: ローカル FS または S3 互換
```

クレートの役割:

| クレート | 役割 |
|---|---|
| `backend` (`mithic-server`) | HTTP + 連合配送。モデル、クエリ、ルート、フェデレーションを同一クレートのモジュールとして持つ |
| `shared` | フロント↔バックの DTO と Markdown レンダラ（wasm 対応） |
| `frontend` | Leptos UI。Trunk で WASM ビルド |

認証は JWT（Argon2 でパスワードハッシュ）。公開 GET は ETag / Cache-Control。サーバー側の外部 fetch には SSRF ガード。認証系には Dragonfly ベースのレート制限。

---

## 技術スタック

**バックエンド** — Rust (edition 2024)、Axum 0.8、Tokio、SurrealDB 3、Dragonfly（Redis 互換）、apalis（配送キュー）。

**フロント** — Leptos 0.7（CSR/WASM）、Trunk 0.21、Tailwind CSS 4（Trunk の standalone CLI、Node.js 不要）、gloo-net（fetch / WebSocket）。状態は Leptos Signals。トークンは LocalStorage。

**配信** — Docker Compose。Caddy が静的ファイルとリバースプロキシ。本番相当では Nginx でも同じ形にできる。

MSRV はワークスペースで 1.85。Docker イメージは新しい stable Rust を使う。

---

## リポジトリ構成

```text
mithic/
├── backend/           mithic-server（lib + bin）
│   ├── locales/       fluent (ja / en)
│   └── src/           ルート / モデル / db / federation / サービス
├── shared/            共有 DTO + Markdown
├── frontend/          Leptos CSR (Trunk)
├── scripts/           開発用（Windows localhost 中継など）
├── docker-compose.yml
└── Caddyfile
```

---

## 動かし方

### Docker

```bash
# 1. 環境変数 — JWT_SECRET は必須（未設定・placeholder では起動しない）
cp .env.example .env
# .env の JWT_SECRET を十分な長さのランダム文字列に書き換える

# 2. ビルドして起動
docker compose up -d --build

# 3. 確認
curl http://localhost:3000/api/v1/health      # => {"status":"ok"}
# ブラウザで http://localhost:3000 （port 80 でも可: http://localhost）
# サインアップ → 投稿 → タイムライン
```

| サービス | ポート | 役割 |
|---|---|---|
| frontend (Caddy) | 80, 3000, 443 | WASM 配信 + `/api` / ActivityPub / WebSocket のプロキシ |
| backend | 3000（内部） | REST + WebSocket + ActivityPub + 配送ワーカー。ホストからは Caddy 経由 |
| surrealdb | 8000 | データベース |
| dragonfly | 6379 | キャッシュ / 配送キュー |

メディアはデフォルトでローカル FS（`media_files` ボリューム）。S3 互換を使う場合は `.env` に `STORAGE_TYPE=s3` と `STORAGE_S3_*` を書く。

### Windows + Podman で `ERR_CONNECTION_REFUSED` になる場合

Podman Machine (WSL) がコンテナの公開ポートを **WSL 側 IP には出すが Windows の `127.0.0.1` には転送しない**ことがある。コンテナ内ヘルスチェックは通っていても、ブラウザだけ `localhost:3000` に繋がらない。

```powershell
# 一時回避: Windows localhost を WSL IP へ中継（管理者権限不要）
python scripts/localhost_proxy.py
# 別ターミナルで確認
curl http://127.0.0.1:3000/api/v1/health
```

恒久対策の候補:

1. Podman Desktop で **User mode networking** を有効化してマシンを再作成 / 再起動する
2. `.wslconfig` で `networkingMode=mirrored` を試す（要 `wsl --shutdown`）
3. 上記プロキシを開発時に併用する

### ローカル開発

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

---

## 設定

主な環境変数（詳細は [`.env.example`](.env.example)）。

| 変数 | 内容 |
|---|---|
| `JWT_SECRET` | 必須。空や `change-me-in-production` は起動拒否 |
| `INSTANCE_URL` / `INSTANCE_NAME` | 公開 URL と表示名。連合の Actor URI の土台 |
| `SURREALDB_*` / `DRAGONFLY_URL` | データストア |
| `STORAGE_TYPE` | `local`（既定）または `s3` |
| `CORS_ALLOWED_ORIGINS` | 許可オリジン |
| `TRUST_PROXY` | リバースプロキシ背後でのみ `true`（`X-Forwarded-For` をクライアント IP にする） |
| `VAPID_PRIVATE_KEY` | 任意。未設定でも購読 API は動くが **Push 配送は no-op** |

Web Push を使う場合:

```bash
# 例: npx web-push generate-vapid-keys
VAPID_PRIVATE_KEY=...
VAPID_CONTACT=mailto:admin@example.com
```

---

## 開発

品質ゲート:

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude frontend -- -D warnings
cargo test --workspace --exclude frontend
cargo check -p frontend --target wasm32-unknown-unknown
```

- 開発ガイドライン: [`CLAUDE.md`](CLAUDE.md)
- 実装状況 / これから: [`TODO.md`](TODO.md)

フロントとバックの型は `shared` クレートで揃える。API は REST、エラーとステータスはルート側で統一する。

---

## 現状

コア（認証、ノート、タイムライン、連合の送受信骨格、PWA、通知ストリーム）は動く想定。製品としてまだ薄いところ:

- **DM** — 画面のみ。API は無い
- **管理 UI** — API はあるが画面は「準備中」
- **ローカル投稿のアンケート** — UI と vote API はあるが、作成時に poll を永続化していない
- **ピン / ブロック / ミュート** — API 済み、一部 UI が未接続
- **TOTP** — コア実装のみ（ルート・設定 UI なし）
- 自動テストは HTTP Signature 単体以外ほぼ無い

詳細と優先順は [`TODO.md`](TODO.md)。

---

## ライセンス

[AGPL-3.0](LICENSE)
