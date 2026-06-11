# Mithic 統合 Todo / 実装ロードマップ

**更新日**: 2026-06-11
**参照**: `old-src/` との機能比較, 全クレート実装インベントリ調査 (2026-06-04), `docs/performance-optimization-plan.md`, `docs/feature-gap-analysis.md`

> **本ファイルについて (2026-06-04 全面刷新)**:
> 実コードの全インベントリ調査に基づきチェックボックスを実態へ同期し、
> `docs/` のロードマップ（縦切り＝動くSNS優先 ＋ 連合並行）を本ファイルに統合した。
> 構成は **Phase 単位のロードマップ** を主軸とし、各 Phase 内に従来の B-/F-/P- 課題IDを対応付けて配置する。
> クレートはリポジトリルート直下 (`api/`, `db/` …) に配置。`crates/foo/...` 表記は現在の `foo/...` を指す。

---

## 実装状況サマリー（2026-06-04 実測）

| クレート | 状態 | 備考 |
|---|---|---|
| `core/models/` | **完了** | 28 エンティティ定義済み |
| `core/services/` | **最小** | `auth.rs` のみ。note/timeline/notification/drive/search 等は未 |
| `mfm/` + `shared/mfm/` | **完了(基本)** | 基本MFMパーサ。カスタム絵文字/位置/アニメ/数式は未 |
| `db/queries/` | **動作** | actors/notes/timeline(home含む)/follows/reactions/favorites/notifications/drive 実装済 |
| `db` プール/キャッシュ | **未** | コネクションプール無し。Dragonfly キャッシュヘルパ未使用。schema は `surreal.rs` 内インライン |
| `api/middleware/` | **完了** | 7 ミドルウェア。HTTP署名は**検証**実装済(テスト付)、**生成**は placeholder |
| `api/routes/` | **基盤完了** | 認証/ノート/タイムライン/ユーザー/通知/ドライブの主要 35 ハンドラ実装・配線済 |
| `federation/` | **配送は動作** | 配送ワーカー/queue/Accept・Reject/broadcast 実装。HTTP署名生成・AP受信エンドポイント・リレー・リモートactorパースは未 |
| `stream/` | **基盤完了** | Channel トレイト/レジストリ/接続/9チャンネル実装。`/api/streaming` 未配線 |
| `server/` | **最小起動** | DB接続・`init_schema()` 済。ミドルウェア層適用/WS/graceful shutdown 未 |
| `worker/` | **最小起動** | 配送ワーカー spawn のみ。他ジョブ(Push/メディア/export/chart)未 |
| `frontend-web/api/` | **一部** | auth/notes/users/notifications/dm/client 実装。timeline/drive/reactions 他は未 |
| `frontend-web/pages/` | **UI完了** | 全ページUI有。Home/Local/Global/Login は実API接続。他はサンプルデータ |
| `frontend-web/components/` | **一部** | avatar/compose/mfm/post_card/protected/shell。ComposeModal は実投稿未接続 |
| `shared/` | **最小限** | User/Note/Notification/Signin/Signup/Reaction 系のみ |

凡例: `[x]` 実装・配線済 / `[~]` 部分実装 / `[ ]` 未着手

---

## ロードマップ全体像

```
Phase 0  基盤の活性化（プール・起動補完・ビルド最適化）      ← P-0/P-1, B-4
Phase 1  認証縦串の完成（ComposeModal 実投稿・共通UI）       ← B-1認証, F-1, F-3
Phase 2  タイムライン縦串（fan-out・キャッシュ・ページング）  ← B-2, B-3, F-2, P-2
Phase 3  ソーシャルグラフ＆通知                              ← B-1 users/i, B-3
Phase 4  ドライブ＆メディア                                  ← B-1 drive, B-7, F-5, F-12
Phase 5  WebSocket ストリーミング配線                        ← stream/, B-4
  ── 並行（Phase 2 以降いつでも開始可）──
Phase F1 ActivityPub 受信基盤（Actor/WebFinger/inbox/outbox）← B-6
Phase F2 連合送信の成立（HTTP署名生成・キュー堅牢化）        ← B-5, B-8, P-2
Phase F3 リレー＆リモートアクター                            ← B-6
  ──────────
Phase 6  二次機能（検索/タグ/リスト/クリップ/チャンネル/アンテナ/投票）
Phase 7  Admin・モデレーション・メタ・チャート・OAuth・Push
Phase 8  フロント機能拡張（絵文字/DM/設定/MFM高度/UIライブラリ）
Phase 9  パフォーマンス仕上げ・観測性・負荷テスト・CI         ← P-3/P-4, I-2
```

---

## Phase 0 — 基盤の活性化（最優先・低難度高効果） ＜P-0 / P-1 / B-4＞

- [x] **SurrealDB コネクションプール** — 複数 `Surreal<Any>` をラウンドロビンする `SurrealClient` プール（`db/src/lib.rs::create_pool`、`SURREALDB_POOL_SIZE` で設定）
- [x] **Dragonfly プール** — `ConnectionManager` 化（自動再接続）。ワーカー BRPOP は `dedicated_connection()` で専用接続を分離（`db/src/lib.rs`）
- [x] `FederationService` を `AppState` のプール済 `reqwest::Client` で初期化（worker 側も同一設定で共有）
- [x] **server/main.rs 補完** — CORS/Trace/Compression レイヤ適用、graceful shutdown（SIGINT/SIGTERM）。rate_limit の全体適用は未
- [x] `init_schema()` を server / worker 双方の起動時に呼ぶ
- [x] **ビルド最適化**（ルート `Cargo.toml`）: `lto="fat"`, `codegen-units=1`, `opt-level=3`, `strip=true`（WASM は `opt-level="z"`）
- [x] `mimalloc` をグローバルアロケータに設定（server/worker の main）
- [ ] `RUSTFLAGS="-C target-cpu=native"` をデプロイ手順に記載

---

## Phase 1 — 認証縦串の完成 ＜B-1認証 / F-1 / F-3＞

### バックエンド（実装済み）
- [x] `POST /api/signup` / `POST /api/signin` / `GET /api/i` / `POST /api/signout`
- [x] 認証サービス `core/services/auth.rs`（Argon2 + JWT `typ:"access"`、auth_middleware と整合）

### フロント F-1（API クライアント基盤）
- [x] APIベースURL `/api`（Trunk proxy）、`AuthStore` JWT 付与共通クライアント、429リトライ
- [ ] APIエラー共通ハンドリング（401/400/422/500/ネットワーク）
- [ ] ローディング / 空状態 / エラー状態 共通UIコンポーネント
- [x] `shared` DTO と実 API レスポンスの差分確認・修正（`/api/v1/*` ルートを新設しフロントと整合。LoginRequest/TokenPair/RefreshRequest/StreamEvent を shared に追加）

### フロント F-3（ComposeModal 実投稿）
- [x] **`ComposeModal` 送信を `api::notes::create_note` に接続**
- [~] 本文・公開範囲・CW・NSFW は送信。添付ファイルID・投票・予約日時・返信先IDの UI は未
- [x] 送信中ボタン disabled / 二重送信防止
- [x] 成功時: モーダルを閉じ下書き削除（TL差し込みは WebSocket 経由）／失敗時: 入力保持・エラー表示
- [x] `Ctrl+Enter` / `Cmd+Enter` で送信

---

## Phase 2 — タイムライン縦串（Fan-out on Write） ＜B-2 / B-3 / F-2 / P-2＞

### DB（一部実装済み・最適化が残る）
- [x] `timeline.rs`: ローカル/グローバル/ホーム（フォローグラフ）取得
- [x] `follows.rs`: follow/unfollow/block/mute、`is_following`/`is_blocking`/`is_muting`、`get_followers`/`get_following`
- [x] `notes.rs`/`timeline.rs`: 作成/削除/取得 + **N+1解消（`actor_id.* AS author` で著者同梱、`NoteWithAuthor`）**
- [ ] スキーマ: `note.host` 非正規化＋複合インデックス `(visibility,host,id)`、follow 双方向インデックス `(out,in)`/`(in,out)`

### Dragonfly キャッシュ（`db/src/cache/` 新設） ＜未着手＞
- [~] タイムライン Sorted Set ヘルパ実装済（`db/src/cache.rs::timeline_push/timeline_range`）。書き込みパスへの組込みは未
- [ ] `note:{id}` ボディキャッシュ（MGET）／ `noteresp:{id}` プリレンダJSON
- [ ] block/mute セットキャッシュ、ユーザープロフィール、インスタンスメタ、カスタム絵文字

### サービス層（`core/src/services/`）
- [x] `note.rs`（`api/src/services/note.rs`）— 投稿作成（タグ抽出・通知・ストリーム配信・AP配送フック込み）
- [ ] `timeline.rs` — Fan-out on Write（フォロワー<10,000 Push、≥10,000 Pull ハイブリッド）

### API / フロント
- [x] `POST /api/notes/{timeline, local-timeline, global-timeline}` 配線済
- [~] フロント F-2: Home/Local/Global 実API接続 + until_id ページング（さらに読み込む）+ WS 差し込み。無限スクロール自動化・WS重複排除は未
- [ ] フロント `api/timeline.rs` 整理（現状 `notes.rs` 内）

---

## Phase 3 — ソーシャルグラフ＆通知 ＜B-1 users/i / B-3＞

### API ルート
- [x] `users/{show,relation,following,followers,notes,search}`、`username/available`
- [x] `following/{create,delete}`、`blocking/{create,delete}`、`muting/{create,delete}`
- [x] `notifications/{list,read,mark-all-as-read}`
- [ ] `following/requests/{accept,reject,cancel,list}` — フォローリクエスト（鍵アカウント）
- [ ] `blocking/list` / `muting/list` — 一覧
- [ ] `i/update`（プロフィール）/ `i/change-password` / `i/regenerate-token` / `i/update-email`
- [ ] `i/pin` / `i/unpin` — ノートのピン留め

### サービス層
- [ ] `core/services/user.rs`（フォロー/ブロック/ミュート管理。現状 `api/services/user.rs` に登録/認証のみ）
- [~] 通知生成: reply/reaction/renote/follow を生成し WS 配信（`api/src/services/note.rs::publish_notification`）。mention は未
- [ ] `core/services/word_mute.rs`（ワードミュート/フィルター）、`suspend_user.rs`

### フロント
- [ ] NotificationsPage を実API化（現状サンプル）
- [ ] ProfilePage を実API化、`FollowButton` コンポーネント（F-13）
- [ ] SettingsProfile / SettingsSecurity を実 API 接続（F-7 一部）
- [ ] フロント `api/i.rs`（プロフィール更新・パスワード変更）

---

## Phase 4 — ドライブ＆メディア ＜B-1 drive / B-7 / F-5 / F-12＞

### DB / API（一部実装済み）
- [x] `db/queries/drive.rs`: ファイル CRUD（create/get/list/delete）
- [x] `drive/files/{create,show,delete}`
- [ ] `drive/files/{find,upload-from-url,attached-notes}`
- [ ] `drive/folders/{create,show,delete}`（フォルダ用クエリ含む）

### サービス層 B-7
- [ ] `core/services/drive.rs` — サムネイル生成（`image` クレートで WebP 変換）
- [ ] 動画サムネイル生成

### フロント
- [ ] F-5: ドロップゾーン（dragenter/over/drop）、最大4ファイル/100MB/MIME検証、進捗UI、プレビュー、ALTテキスト、ファイルマネージャー `/drive`
- [ ] F-12: `MediaImage`(lightbox)/`MediaVideo`/`MediaList`/`DriveFileThumbnail`
- [ ] フロント `api/drive.rs`（アップロード/一覧/削除）

---

## Phase 5 — WebSocket ストリーミング配線 ＜stream/ / B-4＞

- [x] `stream/`: Channel トレイト・レジストリ・接続・9チャンネル（Home/Global/Hashtag/Admin/QueueStats/ServerStats/Drive/ApLog/UserList）実装
- [x] `api/src/routes/streaming.rs`: `GET /api/streaming`（`?token=` JWT 認証 + WS アップグレード）
- [x] `server/main.rs`（`create_router`）に streaming ルート接続
- [~] フロント `connect_stream` と疎通確認済（note/notification イベント受信を E2E 検証）。`Drive` WS は未
- [x] 投稿/通知（reply/reaction/renote/follow）発生時に publish するフック（`AppState::publish_stream`）

---

## Phase F1 — ActivityPub 受信基盤（Phase 2 以降と並行） ＜B-6＞

- [x] Person Actor JSON-LD 生成、公開鍵添付（`api/src/routes/activitypub.rs::build_actor_document`）
- [ ] `api/src/routes/activitypub.rs`:
  - [x] `GET /users/:username`（Actor、`/@:username` エイリアスは未）
  - [x] `GET /.well-known/webfinger`
  - [x] `GET /.well-known/nodeinfo` + `GET /nodeinfo/2.0`
  - [ ] `GET /users/:id/{outbox,followers,following,collections/featured}`
  - [~] `POST /users/:username/inbox` + `POST /inbox`（Follow→自動Accept返送、Undo(Follow) 処理。Create/Like/Announce 等は受理のみ）
- [x] content_negotiation ミドルウェア（`application/activity+json` 振り分け）
- [x] HTTP Signature **検証**（`api/middleware/http_signature.rs`、RSA-SHA256、ユニットテスト付）
- [ ] inbox に HTTP署名検証ミドルウェアを適用

---

## Phase F2 — 連合送信の成立 ＜B-5 / B-8 / P-2＞

- [x] **HTTP署名生成** — RSA-SHA256 実署名（`(request-target) host date digest`、秘密鍵プロセス内キャッシュ）。signup 時に RSA-2048 鍵ペア生成
- [x] 配送ワーカー `run_delivery_worker()` / `process_delivery_task()`（BRPOP ループ実装済）
- [x] `send_accept_follow` / `send_reject_follow` / `broadcast_to_followers` / `queue_delivery`
- [~] **配送の並列化** — 並列4ワーカー（各専用 BRPOP 接続）。ホスト単位セマフォ・prefetch は未
- [x] **`sharedInbox` グルーピング**（`broadcast_to_followers` で重複排除しキュー投入）
- [x] **キュー堅牢化** — `federation:scheduled` ZSET スケジューラ、`federation:dlq`、指数バックオフ+ジッタ
- [ ] **Dead Inbox Circuit Breaker**（`dead_inbox:{host}` 失敗回数記録、閾値超で一時停止）
- [x] visibility フィルタ: `public` のみ配送（`services/note.rs` + `FederationService::should_deliver`）
- [ ] worker 他ジョブ: Web Push 配送 / ファイル処理（サムネイル・WebP）/ export・import / chart 集計（定期）

---

## Phase F3 — リレー＆リモートアクター ＜B-6＞

- [ ] DB: `relay` / `activity` テーブル（`activity.uri` UNIQUE で dedup）
- [x] `fetch_remote_actor` — JSON-LD パース→Actor 変換（`parse_remote_actor`、inbox 受信時に未知アクターを永続化）
- [ ] `remote_actor:{uri}` stale-while-revalidate キャッシュ（フォロー関係のみ永続化）
- [ ] リレー購読フロー（Subscribe: Follow送信→Accept待機→status更新）、Unsubscribe（Undo Follow）
- [ ] リレー受信: `should_persist_note` で関与分のみ DB保存、それ以外は Dragonfly バッファ→破棄
- [ ] リレー配送: ノート作成時に `fanout_to_relays`
- [ ] `i/pin`・`i/unpin` の featured collection 公開

---

## Phase 6 — 二次機能（検索/タグ/リスト/クリップ/チャンネル/アンテナ/投票）

各機能は DBクエリ→サービス→APIルート→フロントAPI→フロントUI の縦串で実装。完了ごとに `shared/` DTO 追加。

### 検索 ＜`db/queries/search.rs`, `core/services/search.rs`＞
- [ ] `notes/{search,search-by-tag,mentions,children,replies,renotes,conversation,state}`
- [x] `users/search`
- [ ] フロント SearchPage を実API化

### ハッシュタグ ＜`routes/hashtags.rs`＞
- [ ] `hashtags/{list,show,trend,users}`、`notes/hashtag`（タイムライン）

### ユーザーリスト ＜`routes/user_lists.rs`＞
- [ ] `users/lists/{create,show,list,delete,update,push,pull}`、`notes/user-list-timeline`
- [ ] フロント F-8: 作成ダイアログ/編集/`/lists/:id`/`pull` 接続

### クリップ ＜`routes/clips.rs`, `core/services/clip.rs`＞
- [ ] `clips/{create,delete,show,list,update,add-note,remove-note,notes}`
- [ ] フロント F-8b: `/clips`/`/clips/:id`/作成削除ダイアログ/NoteMenu 追加項目

### チャンネル ＜`routes/channels.rs`, `core/services/channel.rs`＞
- [ ] `channels/{create,delete,show,list,update,follow,unfollow,followed,timeline,featured}`
- [ ] フロント F-8c: `/channels`/`/channels/:id`/作成編集ダイアログ/フォローボタン

### アンテナ ＜`routes/antennas.rs`＞
- [ ] `antennas/{create,show,list,delete,update,notes}`

### 投票
- [ ] `notes/polls/vote`、`core/services/poll.rs`（集計）、`note/polls/update`
- [ ] フロント `PollView`/`PollEditor`（F-14）

---

## Phase 7 — Admin・モデレーション・メタ・チャート・OAuth・Push

### メタ／統計 ＜`routes/meta.rs`, `core/services/meta.rs`＞
- [ ] `meta`、`stats`、`sw/{register,unregister}`（Web Push 購読）

### チャート ＜`routes/charts.rs`＞
- [ ] `charts/{instance,notes,users,drive,federation,hashtag}`、`charts/user/{notes,following,drive,reactions}`

### Admin ＜`routes/admin/`＞
- [ ] `admin/accounts/{create,delete,suspend,unsuspend}`
- [ ] `admin/emoji/{add,list,remove,update}`
- [ ] `admin/federation/{delete-all-files,update-instance}`
- [ ] `admin/queue/{clear,stats,jobs}`（`get_queue_stats`/`get_queue_jobs` は federation 側に実装済）
- [ ] `admin/relays/{add,list,remove}`、`admin/drive/{clean-files,cleanup}`
- [ ] `admin/{update-meta,vacuum,server-info,get-table-stats}`

### OAuth ＜`routes/oauth.rs`＞
- [ ] `app/{create,show}`、`auth/session/{generate,userkey}`、`GET /api/auth/callback`

### サービス
- [ ] `core/services/push_notification.rs`（VAPID 配送、保守性懸念あれば自前 VAPID 検討）
- [ ] `core/services/fetch_nodeinfo.rs`、`export.rs`/`import.rs`（following/notes/blocking/muting）

### フロント F-9
- [ ] Admin 各画面（instance設定/絵文字/ファイル/監視/キュー/統計）

---

## Phase 8 — フロント機能拡張

### F-4 絵文字・リアクションピッカー
- [ ] `EmojiPicker`（Unicodeカテゴリ別+最近使用）、`ReactionPicker`、カスタム絵文字API接続
- [ ] キーボードナビ（矢印/Enter/Esc）、ComposeModal/PostActions REACT 接続

### F-6 DM 実API接続
- [x] フロント `api/dm.rs`（会話一覧/詳細/送信/作成）
- [ ] DmPage/DmConversationPage を実API化、送信フォーム（`NoteVisibility::Specified`）、未読バッジ同期
- [ ] バックエンド DM 用ルート/クエリ

### F-7 設定画面の充実
- [ ] Profile/Privacy/Security(2FA)/Drive/Reaction/MuteBlock/ImportExport 各ページ
- [ ] 2FA バックエンド: `i/2fa/{register,done,unregister}`

### F-10 オンボーディング
- [ ] `/welcome`、サインアップ3ステップ実API化、インスタンス情報表示

### F-11 MFM 高度機能
- [ ] カスタム絵文字 `:name:`、位置指定 `$[x.right]`/`$[x.left]`、アニメーション `$[jelly]`/`$[spin]`
- [ ] 数式（KaTeX）、URLプレビュー（OGP）— `mfm/` と `shared/mfm/` 両対応

### F-13 UIコンポーネント補完
- [ ] `Autocomplete`/`UrlPreview`/`Toast`/`DateSeparator`/`RenotePicker`/`FollowButton`/`RelativeTime`/`UserHoverCard`

### F-14 ノート表示コンポーネント補完
- [ ] `NoteSubView`/`NoteHeader`/`NoteMenu`/`NotePreview`/`PollView`/`PollEditor`/`VisibilityChooser`

### 国際化
- [ ] `i18n/locales/ja.ftl`・`en.ftl` 整備、フロントに leptos-i18n 導入（現状ハードコード日本語のみ）

---

## Phase 9 — パフォーマンス仕上げ・観測性・負荷テスト・CI ＜P-3 / P-4 / I-2＞

### 観測性 §3-H
- [ ] `metrics` + `metrics-exporter-prometheus`（API レイテンシ P50/95/99、DBクエリ時間、Dragonfly ヒット率、AP キュー深度、fan-out レイテンシ）+ Grafana
- [ ] `tokio-console`（`console-subscriber`）

### P-3 最適化
- [ ] WebSocket ペイロード MessagePack（`rmp-serde`）
- [ ] REST JSON を `simd-json`
- [ ] SurrealDB ラウンドロビン `DbPool` 拡張（Phase 0 着手分の発展）
- [ ] `Arc<str>`/`bytes::Bytes` でクローン削減
- [ ] `tower-http` Brotli、Nginx HTTP/2 + HTTP/3 QUIC

### P-4 高難度
- [ ] SurrealDB TiKV バックエンド + Read Replica
- [ ] 全文検索 Meilisearch / Tantivy（日本語: lindera + Tantivy）
- [ ] サービス分割候補（api/federation/timeline/search を gRPC 連携）

### 負荷テスト §4
- [ ] シードジェネレータ（10k/50k ユーザー・べき乗則フォローグラフ）
- [ ] `criterion` マイクロベンチ、k6/vegeta 負荷シナリオ（投稿スパイク/同時TL取得/リレーバースト）
- [ ] 合格基準: TL P95<50ms(キャッシュヒット)、10kフォロワー fan-out 数百ms、バースト後キュー深度単調減少

### CI I-2 ＜`.github/workflows/`＞
- [x] `cargo fmt --check` / `cargo clippy -D warnings` / `cargo test` / frontend wasm check（`.github/workflows/ci.yml`）

---

## 横断的な進め方（全フェーズ共通）

1. **DTO先行**: 各機能で `shared/` に Request/Response 型を先に定義し、バック・フロントで共有（CreateFollowRequest, BlockRequest, MuteRequest, FileUploadRequest, PollVoteRequest, ClipRequest, ChannelRequest, ListRequest, AntennaRequest, MetaResponse 等）
2. **縦串で完結**: 1機能 = DBクエリ→サービス→APIルート→フロントAPI→フロントUI を1単位で完成
3. **ドキュメント更新（CLAUDE.md §7 必須）**: 機能完了ごとに本ファイルのチェックボックスを更新、`docs/feature-gap-analysis.md` の該当項目を更新し冒頭「検証日」を書き換え
4. **品質ゲート**: 各フェーズ完了時に `cargo fmt --all` / `cargo clippy --all -D warnings` / `cargo check --all`、`trunk build` を通す
5. **コミット規約**: Conventional Commits

## エンドツーエンド検証

- **Phase 1-2 後**: `cargo run -p mithic-server` + `trunk serve` → signup→login→投稿→Home/Local/Global TL→相互フォローで fan-out 確認
- **Phase 3-5 後**: フォロー通知、ドライブ添付サムネイル、2ブラウザでリアルタイム差し込み
- **Phase F1-F3 後**: WebFinger/Actor 取得、外部インスタンス（Mastodon テスト）と Follow/Note 双方向疎通、テストリレー流入の dedup 確認
- **Phase 9**: 負荷テストで合格基準測定、Prometheus/Grafana 可視化、CI グリーン

---

## 完了済み（参考）

- [x] `core/models/` 28 エンティティ定義
- [x] `api/middleware/` 7 種（auth/cors/rate_limit/http_signature(検証)/content_negotiation/locale）
- [x] 認証・ノート・タイムライン(home含む)・ユーザー・通知・ドライブの主要 API ルート 35 ハンドラ
- [x] `db/queries/` actors/notes/timeline/follows/reactions/favorites/notifications/drive
- [x] federation 配送ワーカー・Accept/Reject/broadcast・queue 管理
- [x] `stream/` チャンネル基盤（9 チャンネル）
- [x] I-1: `openssl` 依存を `rsa`/`sha2`（純Rust）へ置換、Windows ビルド対応
- [x] db クエリ層を surrealdb 3.0.5 API へ追従、`cargo check`/`clippy -D warnings` 通過
- [x] (2026-06-11) `/api/v1/*` 互換ルート群（auth login/register/refresh/logout、users me/check-handle/show/notes/follow、timelines、notes CRUD/replies/quotes/reactions/renotes、notifications、health）— フロントエンドと完全整合
- [x] (2026-06-11) DB 層の重大バグ修正: SurrealDB 3 レコードID正規化（`rows_to` で `table:ulid` を剥離）、`type::thing`→`type::record` 全置換、`$token` 予約変数回避、RELATE の括弧構文、`reactions` の `TYPE object FLEXIBLE` 化、モデル serde を DB の snake_case に整合
- [x] (2026-06-11) E2E スモークテスト合格: 登録→ログイン→投稿→3種TL→フォロー→リアクション→リノート→返信→通知→WebSocket リアルタイム配信→WebFinger/NodeInfo/Actor→リフレッシュトークン
- [x] (2026-06-11) Docker 動作確認の障害除去: Dockerfile のパス誤り修正（`crates/` 前提）、healthcheck 用 curl 追加、`.env` を任意化、nginx に `/api/streaming` WS・ActivityPub プロキシ追加、フロント release ビルドのコンパイルエラー修正
- [x] CI: `.github/workflows/ci.yml`（fmt/clippy/test/wasm check、既存）— Phase 9 I-2 充足
