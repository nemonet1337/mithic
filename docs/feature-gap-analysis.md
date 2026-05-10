# 機能網羅性検証レポート

**検証日**: 2026-05-10  
**対象**: `backend/` (Rust/Axum) および `frontend/` (Flutter) vs `old-src/` (Misskey/TypeScript)

---

## バックエンド未実装・不完全機能

### 実装カバレッジ概況

- **実装済み**: 約70〜75%
- **未実装・不完全**: 約25〜30%

---

### 1. Admin APIエンドポイント (未実装・不完全)

`backend/src/routes/admin.rs` に不足している機能：

| エンドポイント | 内容 |
|---|---|
| `admin/accounts/create` | 管理者によるユーザーアカウント作成 |
| `admin/change-password` | 管理者によるユーザーパスワードリセット |
| `admin/delete-all-files-of-a-user` | ユーザーの全ファイル削除（包括的実装） |
| `admin/drive/clean-remote-files` | リモートキャッシュファイルのクリーンアップ |
| `admin/drive/cleanup` | データベースクリーンアップ |
| `admin/drive/show-file` | 特定のドライブファイル詳細表示 |
| `admin/emoji/add` | カスタム絵文字の追加 |
| `admin/emoji/remove` | カスタム絵文字の削除 |
| `admin/emoji/update` | カスタム絵文字の更新 |
| `admin/federation/delete-all-files` | リモートファイルの全削除 |
| `admin/federation/remove-all-following` | 全フォロー関係の削除 |
| `admin/federation/update-instance` | リモートインスタンスメタデータ更新 |
| `admin/get-table-stats` | データベーステーブル統計 |
| `admin/queue/deliver-delayed` | 遅延配送キューのジョブ一覧（ホスト別） |
| `admin/queue/inbox-delayed` | 遅延inboxキューのジョブ一覧（ホスト別） |
| `admin/update-meta` | サーバーメタデータ更新 |
| `admin/update-remote-user` | リモートユーザー情報更新 |
| `admin/vacuum` | データベースVACUUM最適化 |
| `admin/resync-chart` | チャートデータの再同期 |

---

### 2. ドライブ (Drive) エンドポイント (未実装)

`backend/src/routes/drive.rs` に不足している機能：

| エンドポイント | 内容 |
|---|---|
| `drive/files/attached-notes` | ファイルを参照しているノートの取得 |
| `drive/files/check-existence` | MD5ハッシュによるファイル存在チェック |
| `drive/files/find-by-hash` | ハッシュ値によるファイル検索 |
| `drive/files/find` | 条件によるファイル検索 |
| `drive/stream` | ドライブ用WebSocketストリーム |
| ファイル重複排除 | ハッシュベースの重複ファイル検出 |

---

### 3. ノート・タイムライン関連 (未実装)

| エンドポイント | 内容 |
|---|---|
| `notes/search-by-tag` | タグベースの高度なノート検索（返信/リノート/ファイル/投票フィルタ） |
| `notes/unrenote` | リノート解除の専用エンドポイント |
| `hashtags/users` | 特定ハッシュタグを使用しているユーザー一覧 |

---

### 4. ユーザー関連エンドポイント (未実装)

| エンドポイント | 内容 |
|---|---|
| `users/search` | 専用ユーザー検索エンドポイント |
| `users/relation` | ユーザー間の関係ステータス取得 |
| `users/lists/pull` | ユーザーリストからアカウント削除 |
| `/username/available` | ユーザー名の空き確認 |

---

### 5. ActivityPub (不完全)

| 機能 | 内容 |
|---|---|
| featured collection | ピン留めノートのコレクションエンドポイント (`/users/:id/collections/featured`) |
| `i/pin`, `i/unpin` | エンドポイントは存在するがActivityPubへの完全公開が不完全 |

---

### 6. チャート (不完全)

`backend/src/routes/charts.rs` に不足しているチャート種別：

- `charts/drive` — ドライブ使用量チャート
- `charts/federation` — フェデレーション統計チャート
- `charts/hashtag` — ハッシュタグ使用量チャート
- `charts/user/drive` — ユーザー別ドライブチャート
- `charts/user/following` — ユーザー別フォロー/フォロワーチャート
- `charts/user/reactions` — ユーザー別リアクションチャート

---

### 7. WebSocketストリーミングチャンネル (未実装)

現在実装済み: `admin`, `global_timeline`, `hashtag`, `home_timeline`, `queue_stats`, `server_stats`

不足しているチャンネル：

- **drive** — ドライブファイルのリアルタイム変更通知
- **apLog** — ActivityPubイベントログ
- **user-list** — ユーザーリスト更新通知

---

### 8. モデル (未実装)

`backend/src/models/` に不足しているモデル（old-src比較）：

| モデル | 内容 |
|---|---|
| `note-unread` | 未読ノートのステータス管理 |
| `user-note-pinings` | ピン留めノートの管理 |
| `user-publickey` | ユーザー公開鍵の独立管理 |
| `used-username` | 削除済みユーザー名の再利用防止 |

---

### 9. サービス (未実装・不完全)

`backend/src/services/` に不足しているサービス：

| サービス | 内容 |
|---|---|
| `drive/image-processor` | サムネイル生成のための画像処理 |
| `drive/generate-video-thumbnail` | 動画サムネイル生成 |
| `drive/upload-from-url` | URLからのファイルアップロード |
| `fetch-nodeinfo` | リモートインスタンスのNodeInfo取得 |
| `following/requests/accept-all` | 全フォローリクエストの一括承認 |
| `note/polls/update` | 投票データの更新 |
| `note/read` / `note/unread` | ノート既読・未読管理 |
| `suspend-user` / `unsuspend-user` | ユーザーサスペンション管理 |
| `update-hashtag` | ハッシュタグ統計の更新 |

---

### 10. その他の不足機能

- **ワードミュート/フィルター**: 単語ベースのコンテンツフィルタリング
- **キューシステム**: ジョブキュー管理（deliver/inbox）
- **リモートインスタンスクリーンアップ**: キャッシュされたリモートファイルのクリーンアップ
- **ユーザー名追跡**: 削除済みユーザー名の再利用防止

---

## フロントエンド未実装・不完全機能

### 実装カバレッジ概況

実装済みの主要画面: auth, home_timeline, note, profile, notification, settings, compose, search, favorites, bookmarks, blocks, mutes, follow_requests, hashtag, filters, lists, antennas, clips, relays, federation, admin, oauth, two_factor

---

### 1. ダイレクトメッセージ (未実装)

- old-srcでは `messages.vue` として `visibility: 'specified'` のノートを表示するDM画面が存在
- 新フロントエンドには**DM機能が完全に未実装**

---

### 2. Admin管理画面 (不完全)

old-srcにある詳細管理ページ (`old-src/src/client/pages/instance/`)：

| ページ | 内容 |
|---|---|
| `instance/index.vue` | インスタンス設定、ファイル設定、プロキシアカウント管理 |
| `instance/emojis.vue` | カスタム絵文字管理（ローカル＋リモート） |
| `instance/files.vue` | サーバーファイル管理、ストレージ管理 |
| `instance/monitor.vue` | サーバー監視（CPU/メモリ/ディスク/ネットワークのグラフ） |
| `instance/queue.vue` | ジョブキューの監視 |
| `instance/stats.vue` | サーバー統計・分析 |
| `instance/users.vue` | 詳細なユーザー管理インターフェース |

新フロントエンドには `AdminScreen` と `AdminUsersScreen` のみ存在。

---

### 3. タイムライン種別 (不完全)

| タイムライン | old-src | 新フロントエンド |
|---|---|---|
| ホームタイムライン | `index.home.vue` | 実装済み |
| グローバルタイムライン | `index.global.vue` | **未実装** |
| ローカルタイムライン | 存在 | **未実装** |
| リストタイムライン | `list.timeline.vue` | **未実装** |

---

### 4. 設定画面 (不完全)

old-srcにある詳細設定ページ：

| ページ | 内容 |
|---|---|
| `settings.profile.vue` | プロフィール編集 |
| `settings.privacy.vue` | プライバシー設定 |
| `settings.security.vue` | セキュリティ設定 |
| `settings.drive.vue` | ファイル/ドライブ管理（ブラウズ/ダウンロード/削除） |
| `settings.reaction.vue` | カスタムリアクション設定 |
| `settings.general.vue` | 一般設定 |
| `settings.import-export.vue` | データのインポート/エクスポート |

新フロントエンドは基本的なテーマ/言語/データ保存設定のみ。

---

### 5. リスト管理 (不完全)

- old-src: `manage-lists/index.vue`（リスト一覧）、`manage-lists/list.vue`（リスト編集）
- 新フロントエンド: `ListsScreen`（基本表示のみ）
- **不足**: リスト編集、リスト作成ダイアログ、リスト管理インターフェース

---

### 6. ウェルカム・オンボーディング (未実装)

- old-src: `index.welcome.vue`（ウェルカムページ）、`index.welcome.signin.vue`（サインインフロー）、`index.welcome.setup.vue`（初期セットアップ）
- 新フロントエンド: `LoginScreen` のみ
- **不足**: ウェルカムページ、完全なオンボーディングフロー

---

### 7. 絵文字・リアクションピッカー (未実装)

old-srcにある関連コンポーネント：

- `emoji-picker.vue` — 絵文字ピッカー
- `reaction-picker.vue` — リアクションピッカー（設定付き）
- `reaction-icon.vue` — リアクション表示
- `reactions-viewer.vue` — リアクション一覧表示
- `reactions-viewer.details.vue` — リアクション詳細

新フロントエンドには**絵文字・リアクションピッカーUIが存在しない**。

---

### 8. ドライブ (Drive) 画面 (未実装)

- `DriveFile` モデルと `DriveEndpoints` APIは実装済み
- **不足**: ドライブ/ファイルマネージャーUIスクリーン全体

---

### 9. ノート表示コンポーネント (不完全)

old-srcにある詳細なノートコンポーネント：

- `note.sub.vue` — サブ/返信ノート表示
- `note-header.vue` — ノートヘッダー
- `note-menu.vue` — ノートコンテキストメニュー
- `note-preview.vue` — ノートプレビュー
- `sub-note-content.vue` — サブノートコンテンツ

新フロントエンドの `NoteCard` は基本実装のみ。

---

### 10. メディア処理コンポーネント (不完全)

| コンポーネント | 内容 |
|---|---|
| `media-image.vue` | 画像表示 |
| `media-video.vue` | 動画表示 |
| `media-list.vue` | メディアコレクション表示 |
| `media-banner.vue` | メディアバナー |
| `drive-file-thumbnail.vue` | ファイルサムネイル表示 |
| `file-type-icon.vue` | ファイルタイプアイコン |

新フロントエンドにはメディアコンポーネントがほぼ存在しない。

---

### 11. 投稿作成UIコンポーネント (不完全)

| コンポーネント | 内容 |
|---|---|
| `post-form-attaches.vue` | 添付ファイル処理UI |
| `poll-editor.vue` | 投票作成UI |
| `visibility-chooser.vue` | 公開範囲選択UI |
| `cw-button.vue` | コンテンツ警告トグル |
| `uploader.vue` | ファイルアップローダー |

新フロントエンドの `ComposeScreen` は基本機能のみ。

---

### 12. UIコンポーネントライブラリ (不完全)

old-srcにある基本UIコンポーネント (`old-src/src/client/components/ui/`)：

- `autocomplete.vue` — フォーム内オートコンプリート
- `url-preview.vue` — URL プレビュー (OGP表示)
- `toast.vue` — トースト通知
- `date-separator.vue` — タイムラインの日付区切り
- `renote-picker.vue` — リノートアクションピッカー
- `follow-button.vue` — フォロー/フォロー解除ボタン
- `mention.vue` — メンション表示/リンク
- `time.vue` — 相対時刻表示コンポーネント

---

### 13. MFMレンダリング (不完全)

- old-src: `misskey-flavored-markdown.vue` — 完全なMFMレンダリング
- 新フロントエンド: `mfm_text.dart` — 基本的なMFMのみ
- **不足**: カスタム絵文字、位置指定、アニメーション等の高度なMFM機能

---

## 優先度別まとめ

### 高優先度（主要機能の欠如）

1. **バックエンド**: 管理者向けAPI（絵文字管理、キュー管理、インスタンス設定）
2. **バックエンド**: 画像・動画サムネイル生成パイプライン
3. **バックエンド**: ドライブの高度な操作（URLアップロード、ハッシュ重複排除）
4. **フロントエンド**: グローバル/ローカルタイムライン
5. **フロントエンド**: ダイレクトメッセージ機能
6. **フロントエンド**: 絵文字・リアクションピッカー
7. **フロントエンド**: ドライブ/ファイルマネージャーUI

### 中優先度（UX改善）

8. **バックエンド**: ノート既読・未読管理
9. **バックエンド**: ワードミュート/フィルター
10. **フロントエンド**: 詳細設定ページ（プロフィール、プライバシー、セキュリティ）
11. **フロントエンド**: リスト管理UI（編集、作成）
12. **フロントエンド**: 管理者インスタンス監視画面（CPU/メモリ/キュー）

### 低優先度（完全性向上）

13. **バックエンド**: チャートデータ（ドライブ、ハッシュタグ、フェデレーション別）
14. **バックエンド**: ActivityPub featuredコレクション完全対応
15. **フロントエンド**: ウェルカム/オンボーディングフロー
16. **フロントエンド**: URL プレビュー (OGP)
17. **フロントエンド**: 高度なMFM機能（カスタム絵文字、アニメーション）
18. **フロントエンド**: メディアギャラリー・動画プレイヤー

---

*このレポートは `old-src/`（Misskey元実装）との比較に基づいて生成されました。*
