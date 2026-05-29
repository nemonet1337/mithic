# Mithic パフォーマンス最適化 設計プラン

**検証日 / 作成日**: 2026-05-29
**対象**: SurrealDB / Dragonfly / キュー / タイムライン / リレー越し通信
**ブランチ**: `claude/db-performance-optimization-sEC11`
**スコープ**: 本書は**設計ドキュメント**であり、プロダクションコードの変更は含まない。実装フェーズの設計図として用いる。

---

## 1. 背景・目的

「数万人（リレー経由含む）規模の投稿を、タイムライン上で完全に捌き切るパフォーマンス」を確保することがゴール。
DB 構造・グラフ処理・キュー処理・投稿取得方法・リレー越し通信のすべてを調査し、改良余地を洗い出した。

**最重要の発見**: 性能の主要ボトルネックの大半は「遅い実装」ではなく**「未実装のホットパス」**である。
真の最適化とは、ホームタイムラインの fan-out・ワーカーループ・HTTP 署名・リレー取り込みを
**“性能最優先のアーキテクチャ”として設計したうえで実装する**ことに等しい。

> 既存 `TODO.md` 末尾に段階的ロードマップ P-1〜P-4 がある。本書はそれを**置き換えず、根拠と具体設計で肉付け**する。

---

## 2. 確認済みの現状（コード根拠つき）

| 領域 | 現状 | 根拠ファイル | 影響 |
|---|---|---|---|
| スキーマ/インデックス | `init_schema()`（21 インデックス定義）が**どこからも呼ばれない** | `db/src/surreal.rs:53` / `server/src/main.rs` | 本番でスキーマレス・インデックス無しの可能性。全クエリがフルスキャン |
| ホームタイムライン | **不在**。pull のみ。`HomeTimelineChannel` は NOOP | `stream/src/channels.rs` | フォローグラフ join を毎リクエスト実行する前提構造 |
| TL クエリ | 行ごとにレコードリンク deref（`actor_id.host = None`）、`SELECT *, actor_id.id` | `db/src/queries/timeline.rs:13-46` | DB 内 N+1。`ORDER BY id`（ULID 文字列）依存 |
| API 著者解決 | 著者を 1 件ずつ取得（dedup ループ） | `api/src/routes/timeline.rs` | アプリ層 N+1（30 件で最大 ~15 往復） |
| 接続 | SurrealDB 単一 WS、Dragonfly 単一 `MultiplexedConnection`。プール無し | `db/src/lib.rs:13-34`, `api/src/state.rs` | 高同時実行で直列化・WS 1 本に集中 |
| キャッシュ | `cache_key/timeline_key/rate_limit_key` ヘルパは**定義のみ未使用**。TTL/実操作なし | `db/src/dragonfly.rs` | 全リクエストが DB 直撃 |
| ワーカー | Ctrl-C 待ちのみ。`run_delivery_worker()` は**起動されない** | `worker/src/main.rs` | キューが溜まるだけで配送されない |
| キュー | `federation:queue`(LPUSH/BRPOP)・`federation:queue:retry` あり。**retry を誰も drain しない**／DLQ 無し／優先度無し／prefetch 無し／単一ループ | `federation/src/service.rs` | 配送スループット = 1 並列 |
| バッチ | inbox ごとに 1 LPUSH。sharedInbox dedup はあるが**ホスト単位バッチ配送なし** | `federation/src/service.rs:29-48,154-201` | リレー爆発時に N×M リクエスト |
| HTTP 署名 | 署名生成は `"placeholder"`、検証は常に `Ok(true)` | `federation/src/service.rs:50-115`, `api/src/middleware/http_signature.rs:188-203` | 連合配送が拒否される／受信検証バイパス。署名鍵キャッシュ無し |
| HTTP クライアント | `AppState` はプール設定済だが `FederationService::new` が `Client::new()`（無設定）を別途生成 | `api/src/state.rs:30-34`, `federation/src/service.rs:24` | プール再利用されず、接続 2 重 |
| リレー | `relay.rs` はステータスのみ。購読/取り込み/dedup/`should_persist` ロジック・`relay` テーブル無し | `core/src/models/relay.rs`, `db/src/surreal.rs` | リレー流入を処理できない |
| 公開鍵キャッシュ | `actor_key:{key_id}` を 24h TTL でキャッシュ（実装済・良好） | `api/src/middleware/http_signature.rs:221-269` | 無効化戦略なし |
| remote actor 取得 | `fetch_remote_actor` は常に `None`（スタブ） | `federation/src/service.rs:317-332` | リモートアクター解決不可 |

依存（`Cargo.toml`）: `surrealdb 3 (protocol-ws, kv-mem)`, `redis 1.2 (tokio-comp, connection-manager)`, `reqwest 0.13 (json)`, `sigh 1.0.3`（宣言のみ未使用）。

---

## 3. 最適化設計（領域別）

各項目に **[Pn]**（TODO.md フェーズ対応）/ **期待効果** / **対象ファイル** を付す。

### A. スキーマ & インデックス活性化　[P-1 / 即効・最重要]

- **起動時に `init_schema()` を必ず実行**（`server/src/main.rs` / `worker/src/main.rs` の接続直後）。冪等（`IF NOT EXISTS`）なので安全。
- インデックス追加・見直し:
  - `note`: 既存 `actor_id`, `created_at`, `(visibility,created_at)`, `renote_id`, `reply_id`。**ローカル TL 用に `host` を `note` へ非正規化**（`actor_id.host` deref を排除）し `(visibility, host, id)` 複合インデックスを追加。
  - `follow`: `(out, in)` / `(in, out)` 双方向。フォロワー列挙・被フォロー列挙の両系統を O(log n) 化。
  - `block`/`mute`: `(in,out)` 済。TL フィルタ用に `in` 先頭複合を確認。
  - リレー dedup 用 `activity` テーブル + `idx_activity_uri UNIQUE(uri)`、リレー管理 `relay` テーブルを新設。
- **記法統一**: `ORDER BY id` は ULID 文字列順＝時刻順で正しい。`id` を確実に文字列 ULID として保存する前提を明文化（`since_id/until_id` カーソル比較の健全性）。
- **期待効果**: フルスキャン → インデックススキャン。TL/ルックアップが O(n) → O(log n)。
- **対象**: `db/src/surreal.rs`, `server/src/main.rs`, `worker/src/main.rs`。

### B. コネクションプール　[P-3 を一部前倒し / 高効果]

- **SurrealDB `DbPool`**: 複数の `Surreal<Any>` クライアントを保持しラウンドロビン/least-busy で貸し出す薄いラッパを `db` クレートに新設。WS 1 本のヘッドオブラインブロッキングを解消。読み取り主体のため read プール優先。
- **Dragonfly**: 単一 `MultiplexedConnection` を `ConnectionManager` もしくは小プール（`bb8-redis` / `deadpool-redis`）で複数本化。**ワーカーの BRPOP 用は専用接続に分離**（ブロッキング系が多重化接続を占有しうるため）。
- **期待効果**: 同時実行スループット向上、ブロッキングコマンドによる詰まり回避。
- **対象**: `db/src/lib.rs`, `db/src/surreal.rs`, `db/src/dragonfly.rs`, `api/src/state.rs`, `Cargo.toml`。

### C. グラフ処理・クエリ最適化　[P-2 / 高効果]

- **アプリ層 N+1 解消**: `resolve_authors` を廃止し、TL クエリ側で `FETCH actor_id`（または `SELECT ... , actor_id.{...}`）して 1 クエリで著者同梱。
- **ローカル TL の deref 排除**: A の `note.host` 非正規化により行ごと join を撤廃。
- **ブロック/ミュート反映**: ホーム/ローカル TL に反映。対象ユーザーの blocked/muted 集合を Dragonfly Set（`blocks:{uid}` / `mutes:{uid}`）にキャッシュし、毎回グラフ走査せず除外。
- **フォロワー列挙の効率化**: fan-out 元の `SELECT in FROM follow WHERE out = $uid` を B のインデックスで高速化。大量フォロワーはチャンク/ストリーミング取得。
- **期待効果**: TL 1 リクエストの DB 往復を ~16 → 1〜2 に削減。

### D. タイムライン・アーキテクチャ（ハイブリッド fan-out）　[P-2 / 最重要]

現状不在のホーム TL を**性能最優先**で設計する。本最適化の中核。

- **fan-out-on-write（push）を基本**: 投稿作成時、ローカルフォロワーの `timeline:{uid}:home`（Dragonfly **Sorted Set**, score=ULID/timestamp）へ note id を push。
  - `ZADD` → `ZREMRANGEBYRANK` で**上限 300 件**維持、**TTL 24h**。
  - 配送は**ワーカージョブ化**（投稿 API は ZADD エンキューのみ＝書き込みレイテンシ最小）。`redis` パイプラインでフォロワー一括反映。
- **インフルエンサーは pull ハイブリッド**: フォロワー **≥ 10,000** は push せず、TL 取得時にそのアカウント分だけ追加 pull してマージ（fan-out 爆発回避）。閾値は設定化。
- **TL 取得**: `ZREVRANGE timeline:{uid}:home` で id 列 → note 本体は Dragonfly note キャッシュ（`note:{id}`）から **MGET**、欠損のみ DB から `SELECT ... WHERE id IN [...]`（1 クエリ）。
- **Pre-rendered Response Cache** [P-2]: 完成 DTO（`shared::NoteDto`）を**シリアライズ済バイト列**で `noteresp:{id}` にキャッシュし、TL 応答を組み立て時に連結。再シリアライズ CPU を削減。
- **キャッシュ整合**: 削除/編集/リアクション更新時に `note:{id}` / `noteresp:{id}` をイベント駆動で無効化。
- **期待効果**: TL 取得が DB グラフ join → Dragonfly ZSET 読み＋MGET。P95 を大幅短縮し読み取りスケールを確保。
- **対象（新設想定）**: `core/src/services/timeline.rs`, `core/src/services/note.rs`, `db/src/dragonfly.rs`, `stream/src/channels.rs`, `api/src/routes/timeline.rs`。

### E. キュー & ワーカー　[P-1〜P-2 / 最重要]

- **ワーカーを実際に起動**: `worker/src/main.rs` で `run_delivery_worker()` を呼ぶ＋他ジョブ（fan-out, 通知, Web Push, メディア, チャート集計）を並走。graceful shutdown。
- **並列配送**: 単一 BRPOP ループ → **N 並列ワーカー**（`tokio::task` プール、並列度設定可 e.g. 32〜128）＋**ホスト単位の同時接続上限**（セマフォ）で過負荷インスタンスを保護。
- **prefetch / バッチ**: 複数ジョブを `LMPOP`/パイプラインでまとめ取得。配送はホスト単位グルーピング。
- **retry キュー統合**: `federation:queue:retry` を**遅延込みで本キューへ戻す**スケジューラ（ZSET `federation:scheduled` に `retry_after` を score、満了分を本キューへ移送）。現状の「誰も drain しない」を解消。
- **指数バックオフ + ジッタ**: サンダリングハード回避のため乱数ジッタを付与。
- **Dead Letter Queue**: 5 回超過は `federation:dlq` に保存（破棄でなく可観測化）。
- **Dead Inbox Circuit Breaker** [TODO B-6]: `dead_inbox:{host}` に失敗回数、閾値超で一時停止しキューを浪費しない。
- **ジョブライブラリ判断**: 自前 Redis キュー堅牢化（推奨・依存最小）か `apalis` 採用かを比較。デフォルトは自前堅牢化＋抽象化トレイト。
- **期待効果**: 配送スループット 1 並列 → N 並列。バースト後にキューが収束する性質を確立。

### F. フェデレーション / リレー越し通信　[P-1〜P-2 / 高効果＋機能必須]

- **HTTP クライアント一本化**: `FederationService` は `AppState` のプール済 `reqwest::Client` を共有。`Client::new()` の二重生成を廃止。`timeout`（接続/全体）設定、**HTTP/2 有効**、`pool_max_idle_per_host` 拡大。
- **HTTP 署名の実装** [機能必須・連合の前提]: `sigh`（宣言済）または `rsa`/`ring` で RSA-SHA256 署名生成・検証を実装。**署名秘密鍵を `actor.private_key` からデコードしてプロセス内キャッシュ**（毎回 DB 取得・毎回 PEM パースを排除）。`(request-target) host date digest` を正しく署名。
- **ホスト単位バッチ配送**: 同一 sharedInbox 宛は 1 ジョブに集約。`broadcast_to_followers` のグルーピングを**キュー投入前**へ移動。
- **リレー取り込み**（高スループット経路）:
  - 受信は専用の高スループットキュー（peer-to-peer と分離）。
  - **`activity.id`（uri）で dedup**（`activity` テーブル UNIQUE or Dragonfly `seen:{uri}` TTL）。リレーは同一投稿を多重配送するため必須。
  - **`should_persist_note`**: 自インスタンスが関与する投稿のみ DB 保存。無関係はバッファ→破棄（DB 肥大防止）。[TODO B-6]
  - リレー購読フロー（Follow 送信→Accept 待ち→status 更新）、Unsubscribe。
  - visibility フィルタ: `public` のみ配送。
- **remote actor 取得/キャッシュ**: `fetch_remote_actor` の JSON-LD パース実装＋**stale-while-revalidate キャッシュ**（`remote_actor:{uri}` TTL）。バッチ流入時は並列取得。フォローした/された相手のみ永続化。
- **WebFinger / Actor / inbox エンドポイント**: 受信の前提として `/.well-known/webfinger`, `GET /users/{id}`, 共有/個別 inbox を用意。署名検証ミドルウェアを inbox に付与。
- **期待効果**: 連合配送が実際に成立し、リレー大量流入を dedup・選別しながら捌ける。
- **対象**: `federation/src/service.rs`, `federation/src/lib.rs`(+actor/inbox/outbox), `api/src/middleware/http_signature.rs`, `api/src/routes/`, `core/src/models/relay.rs`, `db/src/surreal.rs`。

### G. シリアライズ & ランタイム　[P-1 / P-3 / 低難度・効果中]

- `mimalloc` または `jemalloc` をグローバルアロケータ化（server/worker、+10〜30%）。
- リリースプロファイル: `lto="fat"`, `codegen-units=1`, `opt-level=3`, `strip=true`（ルート `Cargo.toml`）。`RUSTFLAGS="-C target-cpu=native"` はデプロイ手順に明記。
- REST JSON を `simd-json`、WebSocket ペイロードを MessagePack(`rmp-serde`) に [P-3]。
- 頻出クローンを `Arc<str>` / `bytes::Bytes` 化 [P-3]（DTO/note 本文）。
- `tower-http` compression（br）と Nginx 側 HTTP/2・Brotli [P-3]。

### H. 可観測性（最適化の前提）　[P-2]

- `metrics` + `metrics-exporter-prometheus` で API レイテンシ(P50/95/99)、DB クエリ時間、Dragonfly ヒット率、AP キュー深度/配送スループット、fan-out レイテンシを計測。Grafana 可視化。
- `tokio-console`(`console-subscriber`) でブロッキング箇所を特定 [P-3]。
- これらが**最適化前後の定量比較**の基盤になる。

---

## 4. 負荷テスト / ベンチハーネス設計

目的: 「数万人＋リレー流入を捌けるか」を定量検証し、最適化前後を比較可能にする。

1. **シードジェネレータ**（`scripts/seed/` or `xtask`）:
   - N ユーザー（例 10k/50k）、フォローグラフ（ベキ分布：一部インフルエンサー ≥10k フォロワー）、過去 note を一括投入。
   - リレー流入を模す Activity ジェネレータ（dedup/`should_persist` 検証のため重複・無関係を混在）。
2. **マイクロベンチ（`criterion`）**: 各 crate に `benches/`。
   - TL 取得（キャッシュヒット/ミス）、fan-out（1 投稿→N フォロワー ZADD）、federation 配送（署名生成・バッチ化）、グラフクエリ（フォロワー列挙）。
3. **負荷シナリオ**（`k6`/`vegeta` または Rust 製ロードクライアント）:
   - 投稿スパイク（fan-out スループット）、TL 同時取得（P95 レイテンシ）、リレー流入バースト（取り込みキュー深度の収束）。
4. **指標と合格基準（草案・要調整）**:
   - TL 取得 P95 < 50ms（キャッシュヒット）
   - fan-out で 10k フォロワーへ反映 < 数百 ms
   - リレーバースト後にキュー深度が単調減少（ワーカーが追いつく）
   - 配送ワーカーが目標 RPS を維持
5. **CI**: ベンチはオンデマンド（負荷が高いため通常 CI から分離）。

---

## 5. 優先度ロードマップ（実装時の順序）

1. **基盤（P-1 + 一部 P-3）**: スキーマ起動、接続プール、HTTP クライアント一本化、アロケータ/ビルドフラグ、メトリクス土台。低リスク・即効。
2. **ホットパス（P-2）**: ハイブリッド fan-out TL + note/応答キャッシュ、ワーカー並列化 + retry/DLQ/CB、グラフ N+1 解消。最大効果。
3. **連合成立（機能必須）**: HTTP 署名実装 + 鍵キャッシュ、ホスト単位バッチ配送、リレー取り込み（dedup/`should_persist`）、AP/WebFinger エンドポイント。
4. **スケールアウト（P-3/P-4）**: simd-json/MessagePack、`Arc<str>`/`Bytes`、SurrealDB Read Replica(TiKV)、Meilisearch 全文検索、サービス分割。長期。

---

## 6. 対象ファイル（代表）

- **DB/スキーマ**: `db/src/surreal.rs`, `db/src/dragonfly.rs`, `db/src/lib.rs`, `db/src/queries/timeline.rs`, `db/src/queries/notes.rs`
- **接続/状態**: `api/src/state.rs`, `server/src/main.rs`, `worker/src/main.rs`, `config/src/lib.rs`
- **タイムライン/サービス**: `core/src/services/timeline.rs`(新), `core/src/services/note.rs`(新), `stream/src/channels.rs`, `api/src/routes/timeline.rs`
- **連合/リレー**: `federation/src/service.rs`, `federation/src/lib.rs`(+actor/inbox/outbox), `api/src/middleware/http_signature.rs`, `core/src/models/relay.rs`
- **ビルド/依存**: ルート `Cargo.toml`、各 crate `Cargo.toml`
- **ベンチ/シード**: `*/benches/`, `scripts/seed/`
