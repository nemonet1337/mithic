# Mithic SNS アーキテクチャ指示書

## 概要

MithicはMisskey互換のSNSのバックエンドです。Dolphinのコードをベースに Rust で構築します。

> 📝 本指示書は **バックエンド実装** に関する指示のみを扱います。

---

## 1. 技術スタック

### バックエンド

| 技術 | バージョン | 用途 |
|---|---|---|
| Rust | edition 2024 | メイン言語 |
| Axum | 0.8 | Webフレームワーク |
| SurrealDB | 3.0 | メインDB |
| Dragonfly | 最新 | キャッシュ / キュー (Redis互換) |
| Tokio | 1.0 | 非同期ランタイム |
| JWT + Argon2 | - | 認証 / パスワードハッシュ |
| HTTP Signatures (sigh) | - | ActivityPub署名 |
| Axum 組み込み WebSocket | 0.8 | WebSocketストリーミング |
| fluent + unic-langid | - | 国際化 |
| pest | - | MFMパーサ |
| web-push | - | Web Push通知 |

> ⚠️ **注意**: `tokio-tungstenite` は使用しない。Axum 0.8 組み込みの WebSocket を使うこと。

### インフラ

| 技術 | 用途 |
|---|---|
| Docker + Docker Compose | コンテナ管理 |
| Nginx | リバースプロキシ |
| SurrealDB (コンテナ) | DB |
| Dragonfly (コンテナ) | キャッシュ |
| Meilisearch (コンテナ) | 全文検索 (推奨追加) |

---

## 2. ディレクトリ構成

```text
mithic/
├── backend/                 # Rustバックエンド
│   ├── src/
│   │   ├── models/         # データモデル定義
│   │   ├── routes/         # APIルート定義
│   │   ├── services/       # ビジネスロジック
│   │   ├── middleware/     # ミドルウェア
│   │   ├── db/            # データベース関連
│   │   ├── stream/        # WebSocketストリーミング
│   │   ├── mfm/           # MFMパーサ
│   │   └── i18n/          # 国際化
│   ├── locales/           # 多言語リソース
│   └── schemas/           # データベーススキーマ
├── docs/                   # ドキュメント
└── nginx.conf             # Nginx設定
```

---

## 3. アーキテクチャ全体像

```
Client (REST API / WebSocket クライアント)
      ↓
   Axum (Backend)
      ↓
  SurrealDB (永続データ) + Dragonfly (キャッシュ)
      ↓
ActivityPub Federation (他サーバーと連携)
```

### 役割分担

| 層 | 担当 |
|---|---|
| SurrealDB | ノート・ユーザー・フォロー関係などの永続データ |
| Dragonfly | タイムラインキャッシュ・セッション・レートリミット・配送キュー |

---

## 4. 推奨 Cargo.toml 依存関係

```toml
[dependencies]
# Web フレームワーク
axum = "0.8"
tokio = { version = "1.0", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "compression-gzip"] }

# DB
surrealdb = "2"

# キャッシュ
redis = { version = "0.25", features = ["tokio-comp"] }

# 認証
jsonwebtoken = "9"
argon2 = "0.5"

# タスクキュー (ActivityPub配送に必須)
apalis = "0.6"

# バリデーション
validator = "0.18"

# エラーハンドリング
thiserror = "2.0"
anyhow = "1.0"

# シリアライズ
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# トレーシング
tracing = "0.1"
tracing-subscriber = "0.3"

# MFMパーサ
pest = "2.7"
pest_derive = "2.7"

# 国際化
fluent = "0.16"
unic-langid = "0.9"

# HTTP Signatures
sigh = "0.4"

# Web Push
web-push = "0.10"
```

---

## 5. SurrealDB スキーマ設計

### ユーザー

```sql
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username      ON user TYPE string;
DEFINE FIELD display_name  ON user TYPE string;
DEFINE FIELD host          ON user TYPE option<string>; -- nullならローカル
DEFINE FIELD uri           ON user TYPE option<string>; -- ActivityPub URI
DEFINE FIELD inbox_url     ON user TYPE option<string>;
DEFINE FIELD public_key    ON user TYPE string;
DEFINE FIELD created_at    ON user TYPE datetime DEFAULT time::now();

-- インデックス
DEFINE INDEX user_username ON user FIELDS username UNIQUE;
DEFINE INDEX user_uri      ON user FIELDS uri UNIQUE;
```

### ノート

```sql
DEFINE TABLE note SCHEMAFULL;
DEFINE FIELD author        ON note TYPE record<user>;
DEFINE FIELD content       ON note TYPE string;
DEFINE FIELD visibility    ON note TYPE string; -- public/home/followers/specified
DEFINE FIELD uri           ON note TYPE option<string>;
DEFINE FIELD reply_to      ON note TYPE option<record<note>>;
DEFINE FIELD renote_of     ON note TYPE option<record<note>>;
DEFINE FIELD created_at    ON note TYPE datetime DEFAULT time::now();

-- インデックス (必須)
DEFINE INDEX note_timeline  ON note FIELDS author, created_at;
DEFINE INDEX note_created   ON note FIELDS created_at;
DEFINE INDEX note_uri       ON note FIELDS uri UNIQUE;
```

### フォロー / リアクション (グラフエッジ)

```sql
-- フォロー
DEFINE TABLE follows SCHEMAFULL TYPE RELATION FROM user TO user;
DEFINE FIELD created_at ON follows TYPE datetime DEFAULT time::now();

-- リアクション
DEFINE TABLE reaction SCHEMAFULL TYPE RELATION FROM user TO note;
DEFINE FIELD emoji ON reaction TYPE string;
```

---

## 6. SurrealDB クエリパターン

### フォロー関係 (グラフクエリ)

```sql
-- フォロー
RELATE user:alice -> follows -> user:bob SET created_at = time::now();

-- フォロワー取得
SELECT <-follows<-user AS followers FROM user:alice;

-- フォロー取得
SELECT ->follows->user AS following FROM user:alice;
```

### タイムライン取得 (FETCH句でN+1を防ぐ)

```sql
-- ❌ 悪い例: N+1が発生する
SELECT * FROM note;

-- ✅ 良い例
SELECT *, author.* FROM note
WHERE timeline_id = $timeline_id
FETCH author, reactions, files
ORDER BY created_at DESC
LIMIT 20;
```

---

## 7. キャッシュ戦略 (Dragonfly)

```rust
async fn get_timeline(user_id: &str, db: &Surreal<Client>, redis: &RedisPool) {
    let cache_key = format!("timeline:{}", user_id);

    // 1. まずDragonflyから取得
    if let Some(cached) = redis.get(&cache_key).await? {
        return cached;
    }

    // 2. なければSurrealDBから取得
    let notes: Vec<Note> = db.query(
        "SELECT *, author.* FROM note
         WHERE author IN (->follows->user FROM $user)
         FETCH author
         ORDER BY created_at DESC
         LIMIT 20"
    )
    .bind(("user", user_id))
    .await?;

    // 3. Dragonflyにキャッシュ (30秒)
    redis.setex(&cache_key, 30, &notes).await?;
    notes
}
```

### キャッシュ対象一覧

| キー | 内容 | TTL |
|---|---|---|
| `timeline:{user_id}` | タイムライン最新20件 | 30秒 |
| `session:{token}` | JWTセッション | 24時間 |
| `ratelimit:{user_id}` | レートリミット | 1分 |
| `ap:queue:{note_id}` | ActivityPub配送キュー | - |
| `relay:buffer:{instance}` | リレー経由の非関与投稿（連合TL用一時バッファ） | 5分 |

---

## 8. WebSocket 実装方針

Axum 0.8 の組み込み WebSocket を使う。`tokio-tungstenite` は使用しない。

```rust
use axum::extract::ws::{WebSocket, WebSocketUpgrade};

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}
```

---

## 9. Axum ミドルウェア構成

```rust
let app = Router::new()
    .route("/api/v1/notes", post(create_note))
    .route("/api/v1/timeline", get(get_timeline))
    .route("/ws", get(ws_handler))
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http())
    .layer(CompressionLayer::new())
    .layer(AuthLayer::new(jwt_secret));
```

---

## 10. データベース選定記録

### 検討した候補

| DB | 評価 | 結論 |
|---|---|---|
| PostgreSQL | 安定性◎、Misskeyも採用実績あり | **不採用** — RustネイティブSDKの優位性が薄れる |
| ScyllaDB | 超高スループット、水平スケール容易 | **不採用** — Mithicのスケール要件に対して過剰スペック |
| SurrealDB 3.0 | Rust公式SDK、グラフDB機能 | **採用** |

### ScyllaDB を不採用とした理由

- ActivityPubの複雑なリレーション処理に弱い（JOINなし）
- Misskey互換APIはリレーション操作を多用する設計であり相性が悪い
- Rustクライアント (`charybdis`) がまだ発展途上
- 現段階のスケール要件（中小規模SNS）には過剰スペック

### SurrealDB を採用した理由

- Rust公式SDKが充実しており、型安全なクエリが書ける
- グラフDB機能（`RELATE` / `->` 構文）がフォロー・リアクションの表現に最適
- Tokioベースで Axum と相性が良い
- SurrealQL の `FETCH` 句により N+1 問題を回避しやすい

### SurrealDB 採用にあたっての対策事項

**N+1問題の防止**

```sql
-- FETCH句で関連データを一括取得する
SELECT *, author.* FROM note
FETCH author, reactions, files
ORDER BY created_at DESC
LIMIT 20;
```

**インデックス設計（必須）**

```sql
DEFINE INDEX note_timeline ON note FIELDS author, created_at;
DEFINE INDEX note_created  ON note FIELDS created_at;
DEFINE INDEX note_uri      ON note FIELDS uri UNIQUE;
DEFINE INDEX user_username ON user FIELDS username UNIQUE;
DEFINE INDEX user_uri      ON user FIELDS uri UNIQUE;
```

**Dragonfly との役割分担でパフォーマンスを補完**

```
SurrealDB  → 永続データ（ノート・ユーザー・フォロー関係）
Dragonfly  → タイムラインキャッシュ・セッション・レートリミット・配送キュー
```

**対策後の評価**

| 項目 | 評価 |
|---|---|
| グラフリレーション (フォロー・リアクション) | ✅ SurrealDB が得意 |
| パフォーマンス | ✅ Dragonfly キャッシュで補完 |
| Rust との相性 | ✅ 公式 SDK で問題なし |
| 安定性リスク | ⚠️ 本番前に負荷テスト必須 |
| 運用知見の少なさ | ⚠️ ドキュメントが薄い箇所あり |

---

## 11. ActivityPub Relay 実装方針

### Relay とは

ActivityPub Relay は、自サーバーの `public` な投稿を登録済みリレーサーバーへ配送し、逆にリレーサーバーから他インスタンスの投稿を受信する中継の仕組みです。フォロー関係がなくても連合タイムラインを充実させられます。

```
[Mithic] --Announce--> [Relay Server] --Announce--> [他インスタンス群]
[Mithic] <--Announce-- [Relay Server] <--Announce-- [他インスタンス群]
```

### ⚠️ データ保存戦略（重要）

リレー経由では **1日数十万 Activity** が流入する。全てを SurrealDB に保存すると急速にメモリ・ストレージが膨張するため、保存対象を厳密に限定する。

```
✅ SurrealDB に永続保存する（グラフ管理する）
├── relay        ノード  … 購読中のリレーサーバー情報
├── instance     ノード  … 連合先インスタンス情報
├── remote_actor ノード  … フォローした/されたリモートユーザーのみ
└── subscribes_to / distributes エッジ … リレー・インスタンス間の関係性

❌ SurrealDB に保存しない（グラフ管理しない）
└── リレー経由の全投稿本文
    → ローカルユーザーが関与しない投稿は保存しない
    → Dragonfly で一時バッファして連合TLに流した後、破棄
```

**判定ロジック（`RelayAnnounceJob` 内）**

```rust
// ローカルユーザーが関与するか判定してから保存
fn should_persist_note(activity: &Value, state: &AppState) -> bool {
    let object_uri = activity["object"].as_str().unwrap_or_default();

    // ① リプライ先がローカルユーザーの投稿
    // ② メンション先にローカルユーザーが含まれる
    // ③ フォローしているリモートアクターの投稿
    is_reply_to_local(object_uri, state)
        || mentions_local_user(activity, state)
        || is_followed_remote_actor(&activity["actor"], state)
}

async fn process_relay_announce(job: Job<RelayAnnounceJob>, ctx: Data<AppState>) -> Result<(), Error> {
    let activity = &job.activity;

    if should_persist_note(activity, &ctx) {
        // ローカル関与あり → SurrealDB に永続保存
        save_remote_note(activity, &ctx.db).await?;
    } else {
        // ローカル関与なし → Dragonfly に一時バッファして終わり
        buffer_to_federation_timeline(activity, &ctx.redis).await?;
        // TTL 経過後に自動破棄（永続保存しない）
    }

    Ok(())
}
```

### SurrealDB スキーマ

```sql
-- 購読中のリレーサーバー
DEFINE TABLE relay SCHEMAFULL;
DEFINE FIELD inbox_url   ON relay TYPE string;   -- リレーのInbox URL
DEFINE FIELD actor_uri   ON relay TYPE string;   -- リレーのActor URI
DEFINE FIELD status      ON relay TYPE string;   -- pending / accepted / rejected
DEFINE FIELD created_at  ON relay TYPE datetime DEFAULT time::now();

DEFINE INDEX relay_inbox ON relay FIELDS inbox_url UNIQUE;

-- 連合先インスタンス（リレー経由で把握したもの）
DEFINE TABLE instance SCHEMAFULL;
DEFINE FIELD host        ON instance TYPE string;
DEFINE FIELD software    ON instance TYPE option<string>; -- misskey / mastodon 等
DEFINE FIELD created_at  ON instance TYPE datetime DEFAULT time::now();

DEFINE INDEX instance_host ON instance FIELDS host UNIQUE;

-- リモートアクター（フォローした/されたユーザーのみ保存）
DEFINE TABLE remote_actor SCHEMAFULL;
DEFINE FIELD uri         ON remote_actor TYPE string;
DEFINE FIELD host        ON remote_actor TYPE record<instance>;
DEFINE FIELD inbox_url   ON remote_actor TYPE string;
DEFINE FIELD public_key  ON remote_actor TYPE string;
DEFINE FIELD created_at  ON remote_actor TYPE datetime DEFAULT time::now();

DEFINE INDEX remote_actor_uri ON remote_actor FIELDS uri UNIQUE;

-- リレー → インスタンス 配送関係 (グラフエッジ)
DEFINE TABLE distributes SCHEMAFULL TYPE RELATION FROM relay TO instance;
DEFINE FIELD first_seen  ON distributes TYPE datetime DEFAULT time::now();

-- 自サーバー → リレー 購読関係 (グラフエッジ)
DEFINE TABLE subscribes_to SCHEMAFULL TYPE RELATION FROM relay TO relay;
```

### 購読フロー (Subscribe)

リレーへの購読は ActivityPub の `Follow` アクティビティで行う。

```rust
// services/relay.rs

pub async fn subscribe_relay(
    relay_inbox: &str,
    relay_actor_uri: &str,
    state: &AppState,
) -> Result<()> {
    // 1. DBにリレーを pending 状態で登録
    let _: Option<Relay> = state.db
        .create("relay")
        .content(Relay {
            inbox_url: relay_inbox.to_string(),
            actor_uri: relay_actor_uri.to_string(),
            status: "pending".to_string(),
            created_at: Datetime::default(),
        })
        .await?;

    // 2. Follow アクティビティを送信
    let follow_activity = serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "Follow",
        "id": format!("{}/activities/follow-relay/{}", state.config.base_url, uuid::Uuid::new_v4()),
        "actor": format!("{}/actor", state.config.base_url),
        "object": "https://www.w3.org/ns/activitystreams#Public"
    });

    // 3. HTTP Signature付きでリレーのInboxへ POST
    deliver_signed(&follow_activity, relay_inbox, &state.config).await?;

    Ok(())
}
```

### 受信フロー (Inbox)

リレーから届く `Announce` を Inbox で受け取り、ジョブキューで処理する。

```rust
// routes/activitypub.rs

async fn inbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(activity): Json<serde_json::Value>,
) -> impl IntoResponse {
    // HTTP Signature 検証
    if verify_http_signature(&headers, &activity, &state).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match activity["type"].as_str() {
        // リレーからの購読承認
        Some("Accept") => {
            enqueue_job(&state.queue, RelayAcceptJob { activity }).await;
        }
        // リレーからの投稿中継
        Some("Announce") => {
            enqueue_job(&state.queue, RelayAnnounceJob { activity }).await;
        }
        _ => {}
    }

    StatusCode::ACCEPTED.into_response()
}
```

### 配送フロー (Outbox → Relay)

自サーバーに `public` な投稿が作成されたとき、登録済みリレー全てに `Announce` を配送する。

```rust
// services/relay.rs

pub async fn fanout_to_relays(note: &Note, state: &AppState) -> Result<()> {
    // visibility が public のノートのみ対象
    if note.visibility != "public" {
        return Ok(());
    }

    // accepted 状態のリレーを全件取得
    let relays: Vec<Relay> = state.db
        .query("SELECT * FROM relay WHERE status = 'accepted'")
        .await?
        .take(0)?;

    let announce = serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "Announce",
        "id": format!("{}/activities/announce/{}", state.config.base_url, note.id),
        "actor": format!("{}/actor", state.config.base_url),
        "object": note.uri,
        "to": ["https://www.w3.org/ns/activitystreams#Public"]
    });

    // apalis でジョブキューに積んで非同期配送
    for relay in relays {
        enqueue_job(&state.queue, RelayDeliverJob {
            inbox_url: relay.inbox_url,
            activity: announce.clone(),
        }).await?;
    }

    Ok(())
}
```

### ジョブ定義 (apalis)

```rust
// services/jobs.rs

#[derive(Serialize, Deserialize)]
pub struct RelayDeliverJob {
    pub inbox_url: String,
    pub activity: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct RelayAnnounceJob {
    pub activity: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct RelayAcceptJob {
    pub activity: serde_json::Value,
}

// RelayDeliverJob のワーカー
async fn process_relay_deliver(job: Job<RelayDeliverJob>, ctx: Data<AppState>) -> Result<(), Error> {
    deliver_signed(&job.activity, &job.inbox_url, &ctx.config).await
        .map_err(|e| Error::Failed(e.to_string().into()))
}

// RelayAcceptJob のワーカー: status を accepted に更新
async fn process_relay_accept(job: Job<RelayAcceptJob>, ctx: Data<AppState>) -> Result<(), Error> {
    let actor_uri = job.activity["actor"].as_str().unwrap_or_default();
    ctx.db
        .query("UPDATE relay SET status = 'accepted' WHERE actor_uri = $uri")
        .bind(("uri", actor_uri))
        .await?;
    Ok(())
}
```

### Relay 管理 API

```rust
// routes/admin.rs

// GET  /api/v1/admin/relays        → 登録リレー一覧
// POST /api/v1/admin/relays        → リレー購読 (subscribe_relay を呼ぶ)
// DELETE /api/v1/admin/relays/:id  → リレー購読解除 (Undo Follow を送信)

async fn unsubscribe_relay(
    Path(relay_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let relay: Option<Relay> = state.db.select(("relay", &relay_id)).await?;
    let Some(relay) = relay else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let undo_activity = serde_json::json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "Undo",
        "actor": format!("{}/actor", state.config.base_url),
        "object": {
            "type": "Follow",
            "actor": format!("{}/actor", state.config.base_url),
            "object": "https://www.w3.org/ns/activitystreams#Public"
        }
    });

    deliver_signed(&undo_activity, &relay.inbox_url, &state.config).await?;
    state.db.delete(("relay", &relay_id)).await?;

    StatusCode::NO_CONTENT.into_response()
}
```

### 実装チェックリスト

| 項目 | 説明 |
|---|---|
| ✅ `relay` / `instance` / `remote_actor` テーブル定義 | `schemas/relay.surql` に記述 |
| ✅ Subscribe (Follow送信) | `POST /api/v1/admin/relays` |
| ✅ Accept 受信 → status更新 | Inbox → `RelayAcceptJob` |
| ✅ Announce 受信 → **関与ありのみ** SurrealDB保存、それ以外は Dragonfly バッファ→破棄 | Inbox → `RelayAnnounceJob` の `should_persist_note` で判定 |
| ✅ 自投稿のリレー配送 | ノート作成時に `fanout_to_relays` 呼び出し |
| ✅ Unsubscribe (Undo Follow送信) | `DELETE /api/v1/admin/relays/:id` |
| ⚠️ HTTP Signature 検証 | リレーからの受信時に必須 |
| ⚠️ visibility フィルタ | `public` のみ配送。`home` / `followers` は配送しない |
| ⚠️ remote_actor の保存条件 | フォローした/されたユーザーのみ。リレー経由の全アクターは保存しない |

---

## 12. パフォーマンス改良案（さらなる高速化）

中規模以上のトラフィックを想定し、上記の基本構成からさらに高速化する場合の改良案を以下に整理する。優先度・実装難易度・期待効果を考慮して段階的に導入する。

### 12.1 タイムライン配信戦略

#### Push型タイムライン（Fan-out on Write）

現状は Pull 型（読み出し時にフォロー先を辿る）だが、フォロワーの多いユーザーへの対応として Push 型を併用する。

```rust
// ノート作成時に各フォロワーの「ホームTLキャッシュ」へ事前挿入
async fn fanout_to_home_timelines(note: &Note, state: &AppState) -> Result<()> {
    let followers: Vec<UserId> = state.db
        .query("SELECT <-follows<-user.id AS id FROM $author")
        .bind(("author", &note.author))
        .await?.take(0)?;

    // 各フォロワーのホームTLキャッシュへ ZADD（時系列ソート済みセット）
    let mut pipe = redis::pipe();
    for follower in &followers {
        let key = format!("home_tl:{}", follower);
        pipe.zadd(&key, &note.id, note.created_at.timestamp());
        pipe.zremrangebyrank(&key, 0, -301); // 上限300件
        pipe.expire(&key, 86400);             // 24時間
    }
    pipe.query_async(&mut state.redis.get().await?).await?;
    Ok(())
}
```

**ハイブリッド戦略の判定**

| ユーザー種別 | 戦略 | 理由 |
|---|---|---|
| フォロワー < 10,000 | Push型 | 配送コストが低く読み出しが速い |
| フォロワー ≥ 10,000（インフルエンサー） | Pull型 | Push の fan-out コスト爆発を回避 |

読み出し時は両方をマージする。

#### Sorted Set によるタイムライン管理

タイムラインは Redis/Dragonfly の `ZSET`（スコア=タイムスタンプ）で管理する。

- 範囲取得が `O(log N + M)` で高速
- ページング・カーソル取得が単純
- 上限件数を `ZREMRANGEBYRANK` で簡単に維持

### 12.2 SurrealDB パフォーマンス改善

#### コネクションプール最適化

```rust
// Surreal クライアントは内部で接続を再利用するが、複数インスタンスを保持して負荷分散
pub struct DbPool {
    clients: Vec<Surreal<Client>>,
    counter: AtomicUsize,
}

impl DbPool {
    pub fn get(&self) -> &Surreal<Client> {
        let i = self.counter.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        &self.clients[i]
    }
}
```

#### Read Replica の活用

SurrealDB の TiKV バックエンド構成にして、Read/Write を分離する。

```
Write → Primary Node
Read  → Replica Nodes (ラウンドロビン)
```

タイムライン取得・プロフィール表示などの参照系を Replica に逃がす。

#### バッチ書き込み

リアクション・閲覧履歴など書き込み頻度の高いデータは、即時書き込みせず一定間隔でバッチ INSERT する。

```rust
// バッファリング → 100ms ごとに flush
pub struct ReactionBatcher {
    buffer: Mutex<Vec<Reaction>>,
}

impl ReactionBatcher {
    pub async fn flush_loop(self: Arc<Self>, db: Surreal<Client>) {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let batch = std::mem::take(&mut *self.buffer.lock().await);
            if !batch.is_empty() {
                db.query("INSERT INTO reaction $data").bind(("data", batch)).await.ok();
            }
        }
    }
}
```

### 12.3 シリアライズ高速化

#### MessagePack による WebSocket 通信

JSON より高速・小サイズ。WebSocket ペイロードに使う。

```toml
[dependencies]
rmp-serde = "1.3"
```

```rust
// WebSocket 送信
let bytes = rmp_serde::to_vec(&event)?;
socket.send(Message::Binary(bytes)).await?;
```

#### `simd-json` の採用

REST API の JSON パースを SIMD 命令で高速化（標準 `serde_json` の 2〜3 倍）。

```toml
[dependencies]
simd-json = "0.13"
```

#### Pre-rendered Response Cache

ノートの API レスポンス JSON を **完成形のバイト列のままキャッシュ** する。

```rust
// 通常: Note 構造体 → serde_json::to_vec で毎回シリアライズ
// 改良: シリアライズ済みバイト列を Dragonfly に置き、そのまま返す
let cached: Option<Vec<u8>> = redis.get(&format!("note_json:{}", note_id)).await?;
if let Some(bytes) = cached {
    return Response::builder()
        .header("content-type", "application/json")
        .body(Body::from(bytes))?;
}
```

### 12.4 HTTP / ネットワーク層の最適化

#### HTTP/2 + HTTP/3 対応

Nginx で HTTP/2 / HTTP/3 (QUIC) を有効化。WebSocket 多重化のためにも有用。

```nginx
listen 443 ssl http2;
listen 443 quic reuseport;
add_header Alt-Svc 'h3=":443"; ma=86400';
```

#### Brotli 圧縮

`tower-http` の `CompressionLayer` で gzip だけでなく Brotli も有効化。

```toml
tower-http = { version = "0.6", features = ["compression-br", "compression-gzip"] }
```

#### Keep-Alive と Connection Pooling

ActivityPub 配送先サーバーへの HTTP クライアントは `reqwest::Client` を **アプリ全体で1つ共有** し、接続を使い回す。

```rust
let http_client = reqwest::Client::builder()
    .pool_max_idle_per_host(32)
    .pool_idle_timeout(Duration::from_secs(90))
    .http2_prior_knowledge()  // 既知の HTTP/2 サーバー向け
    .build()?;
```

### 12.5 ActivityPub 配送の最適化

#### 配送先のグルーピング（Shared Inbox）

同一インスタンスの複数フォロワーへの配送は、相手の `sharedInbox` に **1回だけ POST** する。

```rust
// 配送先を host ごとにグルーピング
let mut grouped: HashMap<String, Vec<&User>> = HashMap::new();
for follower in remote_followers {
    grouped.entry(follower.shared_inbox.clone().unwrap_or(follower.inbox_url.clone()))
        .or_default()
        .push(follower);
}
// 各 shared_inbox に1回ずつ配送
for (inbox, _users) in grouped {
    enqueue_job(&state.queue, ApDeliverJob { inbox, activity: activity.clone() }).await?;
}
```

#### ジョブ並列度の調整

`apalis` のワーカー数を CPU コア数に応じて動的に調整する。

```rust
let workers = num_cpus::get() * 4;  // I/O bound なので多めに
WorkerBuilder::new("ap-deliver")
    .concurrency(workers)
    .build_fn(process_ap_deliver);
```

#### Dead Inbox の Circuit Breaker

応答しないインスタンスへの配送を一定回数失敗したら一時的に止める。

```rust
// dead_inbox:{host} に失敗回数を ZINCRBY
// 一定閾値超過で TTL 付きの "blocked:{host}" フラグを立てる
// 配送前にフラグをチェックしてスキップ
```

### 12.6 メモリ・CPU 最適化

#### `jemalloc` / `mimalloc` の採用

Rust デフォルトアロケータより 10〜30% 高速。

```toml
[dependencies]
mimalloc = "0.1"
```

```rust
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

#### LTO + ターゲットCPU最適化

リリースビルド設定：

```toml
[profile.release]
lto = "fat"
codegen-units = 1
strip = true
opt-level = 3
```

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

#### `Arc<str>` / `Bytes` の活用

頻繁にコピーされる文字列は `String` ではなく `Arc<str>` で共有。HTTP ボディは `bytes::Bytes` でゼロコピー。

### 12.7 オブザーバビリティ（性能ボトルネック特定）

#### メトリクス収集

```toml
metrics = "0.23"
metrics-exporter-prometheus = "0.15"
```

Prometheus + Grafana で以下を可視化：

- API レイテンシ（P50 / P95 / P99）
- DB クエリ実行時間
- Dragonfly ヒット率
- ActivityPub 配送キュー深度
- ワーカー処理レート

#### `tokio-console` でランタイム可視化

```toml
console-subscriber = "0.4"
```

タスクのブロッキング箇所を即座に特定できる。

### 12.8 アーキテクチャレベルの拡張

#### サービス分割（マイクロサービス化）

将来スケールが必要になった場合の分割候補：

| サービス | 責務 |
|---|---|
| `mithic-api` | REST / WebSocket エンドポイント |
| `mithic-federation` | ActivityPub 送受信専用ワーカー |
| `mithic-timeline` | タイムライン生成・配信 |
| `mithic-search` | Meilisearch ラッパー |

各サービス間は gRPC（`tonic`）で通信する。

#### 全文検索の専門化

SurrealDB の全文検索ではなく **Meilisearch** または **Tantivy** を使う。日本語形態素解析を含めるなら lindera + Tantivy が高速。

### 12.9 改良の優先順位（推奨ロードマップ）

| Phase | 内容 | 期待効果 | 難易度 |
|---|---|---|---|
| 1 | jemalloc/mimalloc、LTO、target-cpu=native | 全体 10〜30% 高速化 | 低 |
| 1 | Shared Inbox 配送 | 連合配送 80% 削減 | 低 |
| 1 | `reqwest::Client` 共有 | 接続オーバーヘッド削減 | 低 |
| 2 | Push型タイムライン（ZSET） | 読み出し 5〜10倍高速 | 中 |
| 2 | Pre-rendered Response Cache | ノート取得 3〜5倍高速 | 中 |
| 2 | Prometheus メトリクス導入 | ボトルネック可視化 | 中 |
| 3 | simd-json + MessagePack | パース/転送 2〜3倍高速 | 中 |
| 3 | SurrealDB Read Replica | 読み込みスケール | 高 |
| 4 | サービス分割 | 水平スケール | 高 |

---

## 13. 既知リスクと対策

| 優先度 | 問題 | 対策 |
|---|---|---|
| 🔴 高 | SurrealDB 3.0 の本番安定性 | **早期に負荷テストを実施する** |
| 🟡 中 | Dragonfly の一部Redis非互換 | Stream系・Pub/Subコマンドを事前確認 |
| 🟡 中 | web-push クレートの保守性 | 代替手段 (vapid自前実装) を調査 |
| 🟢 低 | 全文検索が未定義 | Meilisearch の追加を検討 |

---

## 14. 今後の優先タスク

1. **SurrealDB 負荷テスト** → 早期に実施、ボトルネックを特定
2. **スキーマ定義** → `schemas/` 配下に全テーブルを定義
3. **Dragonfly 動作確認** → 使用するRedisコマンドをすべて検証
4. **ActivityPub Inbox/Outbox 実装** → apalis でジョブキュー化
5. **WebSocket ストリーム設計** → リアルタイム通知の設計を固める
