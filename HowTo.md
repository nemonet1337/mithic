# HowTo — Mithic 実装ガイド（AI 実装エージェント向け）

> **このドキュメントの目的**
> 本ファイルは、**ローカル環境で他の AI モデル（コーディングエージェント）に Mithic の機能を実装させる**ことを前提とした作業ガイドである。
> 各 AI エージェントはこのファイルだけを起点に、(1) 環境を構築し → (2) `TODO.md` からタスクを 1 つ選び → (3) 既存パターンを流用して縦串で実装し → (4) 品質ゲートを通し → (5) ドキュメントを更新し → (6) コミットする、までを単独で完遂できるようにする。
>
> **まず読むもの（順番厳守）**
> 1. `CLAUDE.md` — 開発ガイドライン（技術スタック・責務分離・互換性方針・ドキュメント更新義務）。**最優先で従う**。
> 2. `TODO.md` — Phase 単位の実装ロードマップ。実装タスクはここから選ぶ。
> 3. `docs/feature-gap-analysis.md` — 機能/性能ギャップ台帳。
> 4. `docs/performance-optimization-plan.md` — 性能設計（fan-out / プール / キャッシュ / 配送 / HTTP署名 / リレー）。

---

## 0. 前提知識（最重要・5 行）

- 言語は **Rust (edition 2024)**。バックエンドは Axum 0.8、DB は SurrealDB 3.0、キャッシュ/キューは Dragonfly(Redis互換)、フロントは Leptos 0.7(CSR/WASM)。
- バックエンド・フロントの**共通型は `shared/` クレート**に置く（serde）。新機能は必ず DTO 先行。
- **1 機能 = 縦串**: `db/queries` → `core/services` → `api/routes` → `frontend-web/src/api` → `frontend-web/src/pages|components`。
- 実装済み/未実装の真の状態は `TODO.md` のチェックボックスが**実測同期済み**（2026-06-04）。推測せずここを見る。
- 品質ゲート: `cargo fmt --all` / `cargo clippy --all -D warnings` / `cargo check --all` / フロントは `trunk build`。

---

## 1. リポジトリ構成（クレートマップ）

```
mithic/
├── CLAUDE.md              # 開発ガイドライン（最優先）
├── TODO.md                # Phase 0〜9 / F1〜F3 ロードマップ＋進捗
├── docs/                  # 設計書・ギャップ分析
├── Cargo.toml             # ワークスペース（12 クレート）
├── docker-compose.yml     # surrealdb / dragonfly / server / worker / frontend
├── .env.example           # 環境変数テンプレート
├── infra/nginx/           # リバースプロキシ設定
│
├── shared/    # ★ バック・フロント共通 DTO（serde）。新機能はここから
├── config/    # AppConfig::from_env()（環境変数読込）
├── core/      # models/（28 エンティティ）, services/（ビジネスロジック）, error.rs, auth.rs
├── db/        # surreal.rs(接続+init_schema), dragonfly.rs(接続+キャッシュキー), queries/（SQL）
├── api/       # routes/（HTTP ハンドラ）, middleware/, services/, state.rs(AppState), dto.rs
├── federation/# service.rs（ActivityPub 配送/受信/HTTP署名）
├── stream/    # WebSocket チャンネル基盤（Channel トレイト・レジストリ）
├── mfm/       # MFM パーサ（pest）
├── i18n/      # fluent ロケール
├── server/    # API サーバ バイナリ（main.rs）
├── worker/    # バックグラウンドワーカー バイナリ（main.rs）
└── frontend-web/ # Leptos WASM（api/ pages/ components/ store/ ws ）
```

各クレートの実装状況は `TODO.md` 冒頭の「実装状況サマリー」表を参照。

---

## 2. ローカル環境構築

### 2.1 必要ツール

| ツール | 用途 | インストール |
|---|---|---|
| Rust (stable, edition 2024 対応) | バック・フロント両方 | `rustup` |
| `wasm32-unknown-unknown` target | フロント WASM | `rustup target add wasm32-unknown-unknown` |
| Trunk 0.21 | フロントビルド/devサーバ | `cargo install trunk` |
| Tailwind CSS 3.x | スタイル | Trunk が呼び出す（`tailwind.config.js` 参照） |
| Docker + Docker Compose | SurrealDB / Dragonfly | OS パッケージ |

### 2.2 インフラ起動（SurrealDB + Dragonfly のみ）

ローカル開発では**アプリは cargo / trunk で直接起動**し、DB だけ Docker で立てるのが速い。

```bash
cp .env.example .env          # 値はローカル向けに調整（下記 2.3）
docker compose up -d surrealdb dragonfly
```

ポート: SurrealDB `:8000`、Dragonfly `:6379`。

### 2.3 ローカル用 `.env`（Docker 外からアプリを動かす場合）

`.env.example` はコンテナ間ホスト名（`surrealdb` / `dragonfly`）を使う。**ホストから直接 cargo 実行する場合は `localhost` に置換**する:

```env
SURREALDB_ENDPOINT=ws://localhost:8000
DRAGONFLY_URL=redis://localhost:6379
JWT_SECRET=local-dev-secret-please-change
SERVER_PORT=3000
CORS_ALLOWED_ORIGINS=http://localhost:1420
INSTANCE_URL=http://localhost:3000
LOCAL_STORAGE_PATH=./.local-files
```

環境変数の全項目は `config/src/lib.rs` の `AppConfig::from_env()` を参照（未設定時のデフォルトあり）。

### 2.4 起動

```bash
# API サーバ（:3000）
cargo run -p mithic-server

# ワーカー（別ターミナル。配送キュー消費）
cargo run -p mithic-worker

# フロント dev サーバ（:1420、/api を :3000 へプロキシ）
cd frontend-web && trunk serve
```

`frontend-web/Trunk.toml` が `/api` → `http://127.0.0.1:3000/api/`、`/api/streaming` → WS をプロキシ済み。
ブラウザで `http://localhost:1420` を開く。

---

## 3. 実装ワークフロー（タスク 1 件の進め方）

### Step 1. タスクを選ぶ
`TODO.md` の Phase から **未チェック `[ ]` を 1 つ**選ぶ。依存順は Phase 番号どおり（F 系は Phase 2 以降並行可）。
迷ったら **Phase 0 → 1 → 2** の順（縦切りで動く SNS を最短で通す方針）。

### Step 2. DTO を先に定義
`shared/src/types/` に Request/Response 型を追加（serde derive）。`shared/src/types/mod.rs` で re-export。
バック・フロント双方がこの型を `use shared::types::...` で共有する。

### Step 3. 縦串で実装
機能の性質に応じて下から積む:

```
db/queries/<feature>.rs       … SurrealDB クエリ（§4.3 テンプレート）
core/services/<feature>.rs    … ビジネスロジック（必要な場合）
api/routes/<feature>.rs       … HTTP ハンドラ（§4.1 テンプレート）
api/routes/mod.rs             … ルート登録（public / protected を選ぶ）
frontend-web/src/api/<f>.rs   … fetch ラッパ（§4.4 テンプレート）
frontend-web/src/pages|components … UI 接続（サンプルデータを実 API に置換）
```

### Step 4. 品質ゲート
```bash
cargo fmt --all
cargo clippy --all -- -D warnings
cargo check --all
cd frontend-web && trunk build      # フロント変更時
```

### Step 5. ドキュメント更新（CLAUDE.md §7・必須）
- `TODO.md` の対応チェックボックスを `[x]`（または部分実装は `[~]`）に更新
- `docs/feature-gap-analysis.md` の該当項目を更新し、冒頭「検証日」を当日へ書き換え
- 新たな不足を発見したら両ファイルに追記

### Step 6. コミット（§6 規約）

---

## 4. 再利用パターン／テンプレート（コピー元の実コード付き）

> **原則**: 新規コードを書く前に、必ず近い既存実装を読んで同じ流儀に合わせる。下記は「どのファイルを写経元にするか」の地図。

### 4.1 API ルートハンドラ
**写経元**: `api/src/routes/notes.rs`, `api/src/routes/users.rs`

```rust
use axum::{Json, extract::State};
use axum::Extension;
use mithic_core::{auth::AuthUser, error::AppError};
use crate::state::AppState;

// 認証必須エンドポイント
pub async fn my_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,   // 認証必須のときだけ
    Json(req): Json<MyRequest>,             // shared の DTO
) -> Result<Json<MyResponse>, AppError> {
    // state.surreal() / state.dragonfly() / state.config()
    //   / state.http_client() / state.federation_service() / state.rate_limiter()
    let rows = mithic_db::queries::my_query(state.surreal(), &req.id).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(MyResponse { /* ... */ }))
}
```

エラーは `AppError`（`core/src/error.rs`、`IntoResponse`＋i18n 実装済み）を返す:
`AppError::{Unauthorized, Forbidden, NotFound, Validation, Internal}(String)`。

### 4.2 ルート登録
**写経元**: `api/src/routes/mod.rs` の `create_router`。
- 認証必須 → `protected` ルータ（`auth_middleware` 適用済み。`Extension<AuthUser>` が使える）
- 公開 → `public` ルータ
登録するだけでミドルウェアは自動適用される。

### 4.3 DB クエリ（SurrealDB 3.0.5）
**写経元**: `db/src/queries/notes.rs`, `db/src/queries/follows.rs`

```rust
use crate::SurrealClient;
use crate::queries::rows_to;   // Value -> T 変換ヘルパ

pub async fn my_query(client: &SurrealClient, id: &str) -> anyhow::Result<Vec<MyRow>> {
    let mut res = client
        .query("SELECT * FROM note WHERE actor_id = type::thing('user', $id) ORDER BY id DESC LIMIT $lim")
        .bind(("id", id.to_string()))
        .bind(("lim", 30i64))
        .await?;
    let rows: Vec<surrealdb::types::Value> = res.take(0)?;
    rows_to::<MyRow>(rows)         // serde で変換
}
```

**落とし穴（必読）**:
- surrealdb 3.0.5 は `res.take(0)?` で `Vec<Value>` を取り出し、`rows_to::<T>()` で serde 変換する流儀。直接 `take::<Vec<T>>()` ではなくこのヘルパを使う。
- レコードリンクは `type::thing('table', $id)`。インデックスは `db/src/surreal.rs` の `init_schema()` で定義済み（WHERE 句で活用する）。
- N+1 を避けるため、関連取得は `FETCH actor_id` 等を検討（Phase 2 の最適化対象）。

### 4.4 フロント API クライアント
**写経元**: `frontend-web/src/api/notes.rs`, `frontend-web/src/api/client.rs`

```rust
use crate::api::client::{request, ApiError};
use shared::types::{MyRequest, MyResponse};

pub async fn my_call(token: &str, body: &MyRequest) -> Result<MyResponse, ApiError> {
    request::<MyResponse, _>("POST", "v1/my/endpoint", Some(token), Some(body)).await
}
```

- ベース URL は `/api`（`client.rs::api_base()`）。Bearer トークンは `AuthStore`（`store/auth.rs`）が保持。
- 429 リトライは `request_with_retry` 済み。エラーは `ApiError { status, code, message }`。

### 4.5 フロント ページ/状態
**写経元**: `frontend-web/src/pages/mod.rs` の TimelinePage（実 API 接続済みの良い例）

- データ取得は `Effect::new` + `wasm_bindgen_futures::spawn_local`、状態は `RwSignal<Vec<T>>`。
- 認証は `expect_context::<AuthStore>()`、`Protected` でルートガード。
- サンプルデータのページ（Notifications/Search/DM/Profile/StatusDetail）を実 API に置換するのが主作業。

### 4.6 モデルのコンストラクタ
**写経元**: `core/src/models/`
`Note::new(...)`, `Actor::new_local(...)`, `Notification::new(...)` など既存コンストラクタを使う。手動で全フィールド埋めない。

### 4.7 認証サービス
`core/src/services/auth.rs`: `hash_password` / `verify_password` / `generate_jwt`(`typ:"access"` 付与) / `verify_jwt`。
**注意**: JWT は `typ:"access"` を含む。検証は `api/src/middleware/auth.rs` の `auth_middleware` が `Authorization: Bearer <token>` を読む。

---

## 5. 既知の落とし穴・重要注意点

- **HTTP 署名「生成」は未実装**: `federation/src/service.rs` に `"placeholder"` が残る。連合送信（Phase F2）を触るならここが本丸。**検証側**（`api/src/middleware/http_signature.rs`）は実装済み・テスト付き。
- **`fetch_remote_actor` は HTTP 取得のみ**: JSON-LD パース未実装で `None` を返す（Phase F3）。
- **AP 公開ルート・`/api/streaming` は未配線**: `api/src/routes/mod.rs` に登録が無い（Phase F1 / Phase 5）。
- **server/main.rs は最小**: ミドルウェア層の明示適用・graceful shutdown が未（Phase 0）。`init_schema()` は server では呼ばれるが worker では未。
- **コネクションプール無し**: SurrealDB/Dragonfly とも単一接続（Phase 0）。
- **OpenSSL 不使用**: 署名は純 Rust の `rsa`/`sha2` を使う（Windows ビルド対応のため。`openssl` を再導入しない）。
- **i18n**: バックは fluent（`i18n/`）、フロントは現状ハードコード日本語。文言追加時は `i18n/locales/{ja,en}.ftl` を意識。
- **互換性**: Mastodon/Misskey クライアント互換を崩さない（CLAUDE.md §4）。エンドポイント名・レスポンス形を勝手に変えない。

---

## 6. コミット／ブランチ規約

- **コミット**: Conventional Commits（`feat:` / `fix:` / `docs:` / `refactor:` / `perf:` / `test:` / `chore:`）。本文は日本語可。
- **ブランチ**: 機能ごとに作業ブランチを切る。`main` へ直接 push しない。
- **PR**: 明示依頼があるときのみ作成。
- **rustfmt 必須**: コミット前に `cargo fmt --all`。
- **ライセンス**: AGPL-3.0 を遵守（ヘッダ・依存ライセンスに注意）。

---

## 7. 複数 AI で並行作業するときの衝突回避

- **1 エージェント = 1 Phase / 1 機能**を担当し、縦串（§3 Step 3 のファイル群）を丸ごと持つ。
- 共有点である **`shared/src/types/mod.rs`**, **`api/src/routes/mod.rs`**, **`frontend-web/src/api/mod.rs`**, **`TODO.md`** は最後に追記マージする箇所として意識し、コンフリクトを最小化する（追記は末尾に寄せる）。
- 着手前に `TODO.md` の対象チェックボックスへ担当を明示（例: `[ ] ... （担当: model-X 作業中）`）すると二重実装を防げる。
- DB スキーマ変更（`db/src/surreal.rs` の `init_schema`）は影響が広いので、変更時は Phase 0 担当に集約するか、PR を分けてレビューする。

---

## 8. 検証（実装が「動く」ことの確認）

機能を「完了」とする前に、最低限のエンドツーエンドを手で確認する。

```bash
# 1) インフラ起動
docker compose up -d surrealdb dragonfly
# 2) サーバ + ワーカー + フロント起動（§2.4）
# 3) ブラウザ http://localhost:1420
```

確認シナリオ（Phase 別、`TODO.md` 末尾「エンドツーエンド検証」と対応）:
- **Phase 1-2**: signup → login → 投稿 → Home/Local/Global TL に表示 → 相互フォローで fan-out。
- **Phase 3-5**: フォロー通知が届く、ドライブ添付のサムネイル、2 ブラウザでリアルタイム差し込み。
- **Phase F1-F3**: WebFinger→Actor 取得、外部インスタンス（Mastodon テスト）と Follow/Note 双方向、リレー流入の dedup。

API 単体確認の例:
```bash
curl -s localhost:3000/api/signup -H 'content-type: application/json' \
  -d '{"username":"alice","password":"pw-pw-pw-1234"}'
curl -s localhost:3000/api/signin -H 'content-type: application/json' \
  -d '{"username":"alice","password":"pw-pw-pw-1234"}'      # -> token
# 取得した token を Bearer で投稿
curl -s localhost:3000/api/notes/create -H "authorization: Bearer <token>" \
  -H 'content-type: application/json' \
  -d '{"text":"hello","visibility":"Public","cw":null,"is_nsfw":false,"file_ids":[],"reply_id":null,"poll_choices":[],"scheduled_at":null}'
```
（実際のフィールドは `shared/src/types/note.rs` の `CreateNoteRequest` を正とする。）

---

## 9. チェックリスト（タスク完了の定義 / Definition of Done）

- [ ] `shared/` に DTO を定義し re-export した
- [ ] 縦串（db→service→route→front api→front ui）のうち必要層を実装した
- [ ] `cargo fmt --all` 済み
- [ ] `cargo clippy --all -- -D warnings` がクリーン
- [ ] `cargo check --all` が通る（フロント変更時は `trunk build` も）
- [ ] 最低限の E2E をローカルで手動確認した
- [ ] `TODO.md` のチェックボックスを更新した
- [ ] `docs/feature-gap-analysis.md` を更新し「検証日」を当日にした
- [ ] Conventional Commits でコミットした
