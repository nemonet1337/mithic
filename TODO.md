# Mithic 統合 Todo リスト

**更新日**: 2026-05-19  
**参照**: `docs/feature-gap-analysis.md`, `old-src/` との機能比較, `crates/` 実装調査結果

---

## 現在の実装状況サマリー

| クレート | 状態 | 備考 |
|---|---|---|
| `crates/core/models/` | **完了** | 28 エンティティ定義済み |
| `crates/mfm/` | **完了** | 基本MFMパーサ実装済み |
| `crates/stream/` | **設計済み** | アーキテクチャ定義、実装未 |
| `crates/api/middleware/` | **完了** | 7 ミドルウェア実装済み |
| `crates/api/routes/` | **未実装** | `mod.rs` が空 — 最重要ギャップ |
| `crates/db/` | **スタブ** | DBクライアントのみ、スキーマ・クエリ未 |
| `crates/core/services/` | **未実装** | サービス層なし |
| `crates/federation/` | **スタブ** | `FederationService` 定義のみ |
| `crates/server/` | **スタブ** | `main.rs` のみ |
| `crates/worker/` | **スタブ** | `main.rs` のみ |
| `crates/frontend-web/pages/` | **UI完了** | 10ページ、サンプルデータのみ |
| `crates/frontend-web/api/` | **一部** | auth/notes のみ |
| `crates/shared/` | **最小限** | Note, User, Notification のみ |

---

## 優先度 CRITICAL — バックエンド基盤

### B-1. APIルート実装 (`crates/api/src/routes/`)

`crates/api/src/routes/mod.rs` が完全に空。以下を実装する。

#### 認証 (`routes/auth.rs`)
- [ ] `POST /api/signin` — JWTログイン
- [ ] `POST /api/signup` — アカウント登録
- [ ] `POST /api/signout` — ログアウト
- [ ] `GET /api/i` — 自分のプロフィール取得

#### ノート (`routes/notes.rs`)
- [ ] `POST /api/notes/create` — 投稿作成
- [ ] `POST /api/notes/delete` — 投稿削除
- [ ] `POST /api/notes/show` — 投稿詳細
- [ ] `POST /api/notes/reactions/create` — リアクション追加
- [ ] `POST /api/notes/reactions/delete` — リアクション削除
- [ ] `POST /api/notes/favorites/create` / `delete` — ブックマーク
- [ ] `POST /api/notes/renote` / `unrenote` — リノート
- [ ] `POST /api/notes/search` — 投稿検索
- [ ] `POST /api/notes/search-by-tag` — ハッシュタグ検索
- [ ] `POST /api/notes/mentions` — メンション一覧
- [ ] `POST /api/notes/children` — 返信ツリー取得
- [ ] `POST /api/notes/replies` — 直接返信一覧
- [ ] `POST /api/notes/renotes` — リノート一覧
- [ ] `POST /api/notes/conversation` — 会話スレッド取得
- [ ] `POST /api/notes/state` — 投稿のブックマーク済み・リアクション済み状態
- [ ] `POST /api/notes/polls/vote` — 投票

#### タイムライン (`routes/timeline.rs`)
- [ ] `POST /api/notes/timeline` — ホームタイムライン
- [ ] `POST /api/notes/local-timeline` — ローカルタイムライン
- [ ] `POST /api/notes/global-timeline` — グローバルタイムライン
- [ ] `POST /api/notes/user-list-timeline` — リストタイムライン
- [ ] `POST /api/notes/hashtag` — ハッシュタグタイムライン

#### ユーザー (`routes/users.rs`)
- [ ] `POST /api/users/show` — ユーザー詳細
- [ ] `POST /api/users/search` — ユーザー検索
- [ ] `POST /api/users/relation` — 関係ステータス取得
- [ ] `POST /api/users/followers` — フォロワー一覧
- [ ] `POST /api/users/following` — フォロー中一覧
- [ ] `POST /api/users/notes` — ユーザーノート一覧
- [ ] `POST /api/following/create` / `delete` — フォロー
- [ ] `POST /api/following/requests/accept` / `reject` / `cancel` / `list` — フォローリクエスト
- [ ] `POST /api/blocking/create` / `delete` / `list` — ブロック
- [ ] `POST /api/muting/create` / `delete` / `list` — ミュート
- [ ] `POST /api/username/available` — ユーザー名チェック

#### 自分自身 (`routes/i.rs`)
- [ ] `GET  /api/i` — 自分のプロフィール取得 (認証済み)
- [ ] `POST /api/i/update` — プロフィール更新 (bio, avatar, display name 等)
- [ ] `POST /api/i/change-password` — パスワード変更
- [ ] `POST /api/i/2fa/register` / `done` / `unregister` — 2FA 管理
- [ ] `POST /api/i/regenerate-token` — APIトークン再生成
- [ ] `POST /api/i/update-email` — メールアドレス変更
- [ ] `POST /api/i/pin` / `unpin` — ノートのピン留め

#### メタ情報 (`routes/meta.rs`)
- [ ] `POST /api/meta` — インスタンスメタ情報取得
- [ ] `POST /api/stats` — インスタンス統計情報
- [ ] `POST /api/sw/register` / `unregister` — Web Push 購読登録

#### ドライブ (`routes/drive.rs`)
- [ ] `POST /api/drive/files/create` — ファイルアップロード
- [ ] `POST /api/drive/files/show` / `delete` / `find` — ファイル操作
- [ ] `POST /api/drive/files/upload-from-url` — URL からアップロード
- [ ] `POST /api/drive/files/attached-notes` — 添付先ノート一覧
- [ ] `POST /api/drive/folders/create` / `show` / `delete` — フォルダ操作
- [ ] `WS  /api/drive/stream` — ドライブ WebSocket ストリーム

#### 通知 (`routes/notifications.rs`)
- [ ] `POST /api/notifications/list` — 通知一覧
- [ ] `POST /api/notifications/read` / `mark-all-as-read` — 既読
- [ ] `POST /api/notifications/delete` — 削除

#### ユーザーリスト (`routes/user_lists.rs`)
- [ ] `POST /api/users/lists/create` / `show` / `list` / `delete` / `update`
- [ ] `POST /api/users/lists/push` / `pull` — メンバー追加/削除

#### アンテナ (`routes/antennas.rs`)
- [ ] `POST /api/antennas/create` / `show` / `list` / `delete` / `update`
- [ ] `POST /api/antennas/notes` — アンテナのノート一覧

#### ハッシュタグ (`routes/hashtags.rs`)
- [ ] `POST /api/hashtags/list` — 一覧
- [ ] `POST /api/hashtags/show` — 詳細
- [ ] `POST /api/hashtags/trend` — トレンド
- [ ] `POST /api/hashtags/users` — 使用ユーザー一覧

#### チャート (`routes/charts.rs`)
- [ ] `POST /api/charts/instance` / `notes` / `users` / `drive` / `federation`
- [ ] `POST /api/charts/hashtag`
- [ ] `POST /api/charts/user/notes` / `following` / `drive` / `reactions`

#### Admin (`routes/admin/`)
- [ ] `POST /api/admin/accounts/create` / `delete` / `suspend` / `unsuspend`
- [ ] `POST /api/admin/emoji/add` / `list` / `remove` / `update`
- [ ] `POST /api/admin/federation/delete-all-files` / `update-instance`
- [ ] `POST /api/admin/queue/clear` / `stats` / `jobs`
- [ ] `POST /api/admin/relays/add` / `list` / `remove`
- [ ] `POST /api/admin/drive/clean-files` / `cleanup`
- [ ] `POST /api/admin/update-meta` / `vacuum` / `server-info` / `get-table-stats`

#### クリップ (`routes/clips.rs`)
- [ ] `POST /api/clips/create` / `delete` / `show` / `list` / `update`
- [ ] `POST /api/clips/add-note` / `remove-note` / `notes` — クリップへのノート管理

#### チャンネル (`routes/channels.rs`)
- [ ] `POST /api/channels/create` / `delete` / `show` / `list` / `update`
- [ ] `POST /api/channels/follow` / `unfollow` / `followed`
- [ ] `POST /api/channels/timeline` — チャンネルタイムライン
- [ ] `POST /api/channels/featured` — 注目チャンネル一覧

#### OAuth/アプリ (`routes/oauth.rs`)
- [ ] `POST /api/app/create` / `show` — OAuth アプリ登録・取得
- [ ] `POST /api/auth/session/generate` / `userkey` — OAuth セッション
- [ ] `GET  /api/auth/callback` — OAuth コールバック

#### ActivityPub (`routes/activitypub.rs`)
- [ ] `GET /@:username` — Actor オブジェクト
- [ ] `POST /users/:id/inbox` — 受信 inbox
- [ ] `GET /users/:id/outbox` — 送信 outbox
- [ ] `GET /users/:id/followers` / `following` — フォロワー/フォロー中コレクション
- [ ] `GET /users/:id/collections/featured` — ピン留めノート
- [ ] `GET /.well-known/webfinger` — WebFinger
- [ ] `GET /.well-known/nodeinfo` / `GET /nodeinfo/2.0` — NodeInfo

#### WebSocket (`routes/streaming.rs`)
- [ ] `GET /api/streaming` — ストリーミング接続ルーター

---

### B-2. データベース層 (`crates/db/src/`)

現在 SurrealDB/Dragonfly クライアントのラッパーのみ存在。

#### スキーマ (`db/src/schema/`)
- [ ] `actor.surql` — ユーザー/アクター テーブル定義
- [ ] `note.surql` — ノート テーブル定義
- [ ] `follow.surql` / `block.surql` / `mute.surql` — グラフエッジ定義
- [ ] `drive_file.surql` / `drive_folder.surql`
- [ ] `notification.surql`
- [ ] `user_list.surql` / `antenna.surql`
- [ ] `emoji.surql` / `hashtag.surql`
- [ ] `relay.surql` / `instance.surql`

#### クエリ (`db/src/queries/`)
- [ ] `actors.rs` — CRUD + フォロワー数更新
- [ ] `notes.rs` — 作成/削除/取得/タイムライン (FETCH句N+1防止)
- [ ] `timeline.rs` — ホーム/ローカル/グローバル タイムラインクエリ
- [ ] `follows.rs` — グラフクエリ (->follows->)
- [ ] `notifications.rs` — 通知取得・既読
- [ ] `drive.rs` — ファイル/フォルダ CRUD
- [ ] `search.rs` — 全文検索クエリ

#### Dragonflyキャッシュ (`db/src/cache/`)
- [ ] タイムライン Sorted Set (Fan-out on Write)
- [ ] ユーザープロフィールキャッシュ
- [ ] インスタンスメタキャッシュ
- [ ] カスタム絵文字キャッシュ

---

### B-3. サービス層 (`crates/core/src/services/`)

サービスディレクトリが存在するが実装なし。

- [ ] `auth.rs` — 認証・JWT発行/検証・Argon2ハッシュ
- [ ] `note.rs` — 投稿作成・削除・タイムライン構築
- [ ] `user.rs` — フォロー・ブロック・ミュート管理
- [ ] `drive.rs` — ファイルアップロード・サムネイル生成
- [ ] `notification.rs` — 通知生成・配送
- [ ] `search.rs` — ノート/ユーザー検索
- [ ] `timeline.rs` — Fan-out on Write タイムライン管理
- [ ] `suspend_user.rs` — ユーザーサスペンション
- [ ] `word_mute.rs` — ワードミュート/フィルター
- [ ] `poll.rs` — 投票管理・結果集計
- [ ] `push_notification.rs` — Web Push 配送
- [ ] `fetch_nodeinfo.rs` — リモートインスタンス NodeInfo 取得
- [ ] `clip.rs` — クリップ作成・管理・ノート追加
- [ ] `channel.rs` — チャンネル作成・フォロー・タイムライン
- [ ] `meta.rs` — インスタンスメタ情報・統計集計
- [ ] `export.rs` / `import.rs` — データ出力・取り込み処理

---

### B-4. サーバー起動 (`crates/server/src/main.rs`)

- [ ] Axum ルーター組み立て (`crates/api` の routes を接続)
- [ ] ミドルウェアスタック適用 (認証・CORS・レート制限・HTTP署名)
- [ ] SurrealDB 接続初期化
- [ ] Dragonfly 接続初期化
- [ ] WebSocket ストリームハンドラ組み込み
- [ ] graceful shutdown 実装

---

### B-5. ワーカー起動 (`crates/worker/src/main.rs`)

- [ ] ジョブキュー (apalis) 初期化
- [ ] ActivityPub 配送ジョブ (deliver/inbox)
- [ ] Relay Announce ジョブ
- [ ] Web Push 配送ジョブ
- [ ] ファイル処理ジョブ (サムネイル生成・WebP変換)
- [ ] データエクスポートジョブ (export-following, export-notes, export-blocking, export-muting)
- [ ] データインポートジョブ (import-following, import-blocking, import-muting)
- [ ] チャートデータ集計ジョブ (定期実行)

---

## 優先度 HIGH — フェデレーション

### B-6. ActivityPub 実装 (`crates/federation/src/`)

- [ ] `actor/` — Actor オブジェクト生成・署名
- [ ] `inbox/` — 受信アクティビティ処理 (Create/Delete/Follow/Undo/Accept/Reject/Announce/Like/Update)
- [ ] `outbox/` — 送信キューイング・Fan-out
- [ ] Relay 購読フロー (Subscribe: Follow 送信 → Accept 待機 → status 更新)
- [ ] Relay 受信フロー: `should_persist_note` で関与ありのみ DB 保存、それ以外は Dragonfly バッファ→破棄
- [ ] Relay 配送フロー: ノート作成時に `fanout_to_relays` 呼び出し
- [ ] Relay Unsubscribe (Undo Follow 送信)
- [ ] HTTP Signature 検証 (Relay からの受信時に必須)
- [ ] visibility フィルタ: `public` のみ配送。`home` / `followers` は配送しない
- [ ] `remote_actor` 保存条件: フォローした/されたユーザーのみ (Relay 経由の全アクターは保存しない)
- [ ] Dead Inbox Circuit Breaker (`dead_inbox:{host}` に失敗回数を記録、閾値超で一時停止)
- [ ] Shared Inbox グルーピング最適化
- [ ] `i/pin` / `i/unpin` ActivityPub 公開 (featured collection)

---

## 優先度 HIGH — フロントエンド API 接続

### F-1. API クライアント基盤 (`crates/frontend-web/src/api/`)

- [ ] APIベースURL設定 (Trunk proxy `/api` / 本番環境変数)
- [ ] `AuthStore` JWT を全リクエストヘッダに付与する共通クライアント
- [ ] APIエラー共通ハンドリング (401/400/422/500/ネットワーク)
- [ ] ローディング / 空状態 / エラー状態 共通UIコンポーネント
- [ ] `shared` DTO と実際の API レスポンスの差分確認・修正

#### 追加が必要な API モジュール
- [ ] `api/timeline.rs` — ホーム/ローカル/グローバルタイムライン
- [ ] `api/users.rs` — ユーザー取得・フォロー・検索
- [ ] `api/notifications.rs` — 通知取得・既読
- [ ] `api/drive.rs` — ファイルアップロード・一覧・削除
- [ ] `api/messages.rs` — DM会話一覧・詳細・送信
- [ ] `api/reactions.rs` — リアクション送信・削除
- [ ] `api/lists.rs` — ユーザーリスト CRUD
- [ ] `api/clips.rs` — クリップ CRUD・ノート追加/削除
- [ ] `api/channels.rs` — チャンネル CRUD・フォロー
- [ ] `api/antennas.rs` — アンテナ CRUD
- [ ] `api/hashtags.rs` — ハッシュタグ検索・トレンド
- [ ] `api/meta.rs` — インスタンスメタ情報・統計
- [ ] `api/i.rs` — プロフィール更新・パスワード変更・2FA

---

### F-2. タイムライン実データ接続

- [ ] `HomePage` のモックデータを `fetch_timeline` に置換
- [ ] `/local` / `/global` を実 API エンドポイントに接続
- [ ] `since_id` / `until_id` / `limit` ページング実装
- [ ] WebSocket 新着差し込みと初回取得データの重複排除
- [ ] 無限スクロール実装

---

### F-3. ComposeModal 実投稿

- [ ] `ComposeModal` の送信を `api::notes::create_note` に接続
- [ ] 本文・公開範囲・CW・NSFW・添付ファイルID・投票・予約日時・返信先ID
- [ ] 送信中ボタン disabled / 二重送信防止
- [ ] 投稿成功時: モーダルを閉じ、下書き削除、タイムライン先頭に差し込み
- [ ] 投稿失敗時: 入力保持・エラー表示
- [ ] `Ctrl+Enter` / `Cmd+Enter` で送信

---

## 優先度 MEDIUM — フロントエンド機能

### F-4. 絵文字・リアクションピッカー

- [ ] `EmojiPicker` コンポーネント (Unicode カテゴリ別 + 最近使用)
- [ ] `ReactionPicker` コンポーネント
- [ ] カスタム絵文字 API 接続・表示
- [ ] キーボードナビゲーション (矢印/Enter/Esc)
- [ ] ComposeModal 絵文字ボタンに接続
- [ ] PostActions REACT ボタンに接続

---

### F-5. Drive / メディア添付 UI

- [ ] ComposeModal ドロップゾーン実装 (dragenter/dragover/drop)
- [ ] ファイル制限: 最大4ファイル / 100MB / MIMEバリデーション
- [ ] アップロード進捗 UI
- [ ] 添付プレビュー (画像/動画/その他)
- [ ] ALT テキスト入力
- [ ] ファイルマネージャー画面 (`/drive`) 設計・実装

---

### F-6. DM 実API接続

- [ ] DM 会話一覧を実 API 取得に置換
- [ ] DM 会話詳細を実 API 取得に置換
- [ ] DM 送信フォーム実装 (`NoteVisibility::Specified`)
- [ ] 未読バッジを API / WebSocket と同期

---

### F-7. 設定画面の充実

old-src `settings.*.vue` との差分:

- [ ] `SettingsProfilePage` — プロフィール編集フォームを実 API に接続
- [ ] `SettingsPrivacyPage` — プライバシー設定
- [ ] `SettingsSecurityPage` — パスワード変更・2FA
- [ ] `SettingsDrivePage` — ドライブ管理
- [ ] `SettingsReactionPage` — カスタムリアクション設定
- [ ] `SettingsMuteBlockPage` — ミュート/ブロック一覧・管理
- [ ] `SettingsImportExportPage` — データエクスポート

---

### F-8. リスト管理 UI

- [ ] リスト作成ダイアログ
- [ ] リスト編集 (メンバー追加/削除)
- [ ] リストタイムライン画面 (`/lists/:id`)
- [ ] `users/lists/pull` API 接続 (メンバー削除)

---

### F-8b. クリップ管理 UI

- [ ] クリップ一覧ページ (`/clips`)
- [ ] クリップ作成・削除ダイアログ
- [ ] ノートをクリップに追加するメニュー項目 (`NoteMenu`)
- [ ] クリップ詳細ページ (`/clips/:id`) — クリップ内ノート表示

---

### F-8c. チャンネル機能 UI

- [ ] チャンネル一覧ページ (`/channels`) — 参加中 / 注目
- [ ] チャンネル作成・編集ダイアログ
- [ ] チャンネルタイムラインページ (`/channels/:id`)
- [ ] チャンネルフォロー/アンフォローボタン

---

### F-9. Admin 管理画面の充実

new Frontend では `AdminPage` と `AdminUsersPage` のみ。old-src との差分:

- [ ] インスタンス設定ページ (`instance/index`)
- [ ] カスタム絵文字管理 (`instance/emojis`)
- [ ] ファイル管理 (`instance/files`)
- [ ] サーバー監視 (`instance/monitor`) — CPU/メモリ/ディスク
- [ ] キュー監視 (`instance/queue`)
- [ ] 統計ページ (`instance/stats`)

---

## 優先度 MEDIUM — バックエンド補完

### B-7. 未実装サービス

- [ ] `drive/image-processor` — サムネイル生成 (WebP変換)
- [ ] `drive/generate-video-thumbnail` — 動画サムネイル
- [ ] `word_mute` / `word_filter` — 単語ベースコンテンツフィルタリング
- [ ] `suspend_user` / `unsuspend_user` — ユーザーサスペンション
- [ ] `fetch_nodeinfo` — リモートインスタンス NodeInfo 取得
- [ ] `note/polls/update` — 投票データ更新

---

### B-8. キューシステム完全実装 (`apalis`)

- [ ] deliver キュー (ActivityPub 配送)
- [ ] inbox キュー (受信処理)
- [ ] 指数バックオフリトライ
- [ ] Dead Letter Queue
- [ ] キュー管理 Admin API 接続

---

## 優先度 LOW — 品質・完全性

### F-10. ウェルカム/オンボーディング

- [ ] ウェルカムページ (`/welcome`)
- [ ] サインアップフロー改善 (3ステップ → 実 API)
- [ ] インスタンス情報表示

---

### F-11. MFM 高度機能

現在は基本的なMFM解析のみ。

- [ ] カスタム絵文字レンダリング (`:emoji_name:`)
- [ ] 位置指定 (`$[x.right ...]` / `$[x.left ...]`)
- [ ] アニメーション (`$[jelly ...]` / `$[spin ...]`)
- [ ] 数式レンダリング (KaTeX)
- [ ] URL プレビュー (OGP 取得 → `url-preview.vue` 相当)

---

### F-12. メディアコンポーネント

- [ ] `MediaImage` — lightbox対応画像表示
- [ ] `MediaVideo` — 動画プレイヤー
- [ ] `MediaList` — メディアコレクション表示
- [ ] `DriveFileThumbnail` — ファイルサムネイル

---

### F-13. UIコンポーネントライブラリ補完

- [ ] `Autocomplete` — メンション・ハッシュタグ補完
- [ ] `UrlPreview` — OGP プレビューカード
- [ ] `Toast` — トースト通知
- [ ] `DateSeparator` — タイムライン日付区切り
- [ ] `RenotePicker` — リノートアクションダイアログ
- [ ] `FollowButton` — フォロー/フォロー解除
- [ ] `RelativeTime` — 相対時刻表示
- [ ] `UserHoverCard` — ユーザーホバーカード

---

### F-14. ノート表示コンポーネント補完

`old-src` の `note.sub.vue` / `note-header.vue` / `note-menu.vue` / `note-preview.vue` / `sub-note-content.vue` に相当するもの:

- [ ] `NoteSubView` — 引用・返信先のネスト表示
- [ ] `NoteHeader` — アバター・ユーザー名・時刻ヘッダ
- [ ] `NoteMenu` — 編集/削除/ピン/報告等のコンテキストメニュー
- [ ] `NotePreview` — 通知やDMの中に埋め込まれるプレビューカード
- [ ] `PollView` — 投票選択肢・進捗バー・結果表示 (`poll.vue` 相当)
- [ ] `PollEditor` — 投票作成 UI (`ComposeModal` 内)
- [ ] `VisibilityChooser` — 公開範囲選択ドロップダウン

---

## パフォーマンス最適化 (段階的導入)

### P-1. Phase 1 — 低難易度・高効果（早期導入推奨）

- [ ] `mimalloc` または `jemalloc` をグローバルアロケータに設定 (+10〜30%)
- [ ] リリースビルドに `lto = "fat"`, `codegen-units = 1`, `opt-level = 3`, `strip = true`
- [ ] `RUSTFLAGS="-C target-cpu=native"` で native 向け最適化
- [ ] ActivityPub 配送で `sharedInbox` にグルーピング (同一インスタンスへは1回のみ POST)
- [ ] `reqwest::Client` をアプリ全体で共有しコネクション再利用 (`pool_max_idle_per_host = 32`)

### P-2. Phase 2 — 中難易度・大効果

- [ ] Push型タイムライン (Fan-out on Write) — ZSET にスコア=タイムスタンプで管理
  - フォロワー < 10,000: Push 型、≥ 10,000 (インフルエンサー): Pull 型ハイブリッド
  - `ZREMRANGEBYRANK` で上限300件維持、TTL 24時間
- [ ] Pre-rendered Response Cache — シリアライズ済みJSON を Dragonfly にバイト列で保存して返す
- [ ] Prometheus メトリクス (`metrics` + `metrics-exporter-prometheus`) + Grafana 可視化
  - API レイテンシ (P50/P95/P99)、DB クエリ時間、Dragonfly ヒット率、AP キュー深度

### P-3. Phase 3 — 中難易度

- [ ] WebSocket ペイロードを MessagePack (`rmp-serde`) に変更 (JSON より高速・小サイズ)
- [ ] REST API の JSON パースを `simd-json` に変更 (2〜3倍高速)
- [ ] SurrealDB コネクションプール: 複数クライアントをラウンドロビンで使い回す `DbPool`
- [ ] `tokio-console` (`console-subscriber`) でランタイムのブロッキング箇所を可視化
- [ ] `Arc<str>` / `bytes::Bytes` で頻繁なクローンを削減
- [ ] Nginx で Brotli 圧縮有効化 (`tower-http` の `compression-br` feature)
- [ ] Nginx で HTTP/2 (`listen 443 ssl http2`) + HTTP/3 QUIC 対応

### P-4. Phase 4 — 高難易度

- [ ] SurrealDB を TiKV バックエンドで構成し Read Replica 追加 (読み取りスケール)
- [ ] 全文検索を Meilisearch または Tantivy (日本語: lindera + Tantivy) に専門化
- [ ] サービス分割候補: `mithic-api` / `mithic-federation` / `mithic-timeline` / `mithic-search` (gRPC 連携)

---

## 既知リスクと対策

| 優先度 | 問題 | 対策 |
|---|---|---|
| 高 | SurrealDB 3.0 の本番安定性 | **早期に負荷テストを実施する** |
| 中 | Dragonfly の一部 Redis 非互換 | Stream系・Pub/Sub コマンドを事前確認 |
| 中 | `web-push` クレートの保守性 | 代替手段 (VAPID 自前実装) を調査 |
| 低 | 全文検索が SurrealDB 任せ | Meilisearch の追加を検討 |

---

## インフラ・ビルド

### I-1. Windows 開発環境 OpenSSL 問題

- [ ] `openssl` 依存を `rustls` 系へ置換するか検討
- [ ] または `vendored` feature 使用の手順を `docs/` に追記
- [ ] `cargo check -p mithic-server` が Windows で通るようにする

---

### I-2. CI/CD

- [ ] `cargo check --all` が CI で通ることを確認
- [ ] `cargo fmt --check --all` を CI に追加
- [ ] `cargo clippy --all` を CI に追加
- [ ] フロントエンド `trunk build` を CI に追加

---

## 横断タスク

- [ ] 機能を実装・完了したら本ファイル (`TODO.md`) の対応チェックボックスをチェック済みにする
- [ ] `crates/shared/` の DTO を実 API レスポンスに合わせて拡充 (CreateFollowRequest, BlockRequest, MuteRequest, FileUploadRequest, ReactionRequest, PollVoteRequest, ClipRequest, ChannelRequest, ListRequest, AntennaRequest, MetaResponse 等)
- [ ] `crates/i18n/locales/` に `ja.ftl` / `en.ftl` を整備
- [ ] 主要 UI のアクセシビリティ確認 (focus ring / aria-label / keyboard nav)
- [ ] モバイル表示を実機幅で確認
- [ ] `cargo fmt --all` を常時維持
- [ ] SurrealDB 負荷テストを早期に実施し、ボトルネックを特定する
- [ ] Dragonfly で使用するすべての Redis コマンドを事前に動作確認する
