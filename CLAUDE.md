# Mithic 開発ガイドライン

## 1. 使用している技術スタック

### バックエンド

- **言語**: Rust (edition 2024)
- **Webフレームワーク**: Axum 0.8
- **データベース**: SurrealDB 3.0
- **キャッシュ/キュー**: Dragonfly (Redis互換)
- **非同期ランタイム**: Tokio 1.0
- **認証**: JWT (`jsonwebtoken`) + Argon2 (パスワードハッシュ)
- **ActivityPub**: HTTP Signatures (`sigh`)
- **WebSocket**: Axum WebSocket
- **国際化**: fluent + unic-langid
- **MFMパーサ**: pest
- **Web Push**: web-push

### フロントエンド

- **言語**: Rust (edition 2024)
- **フレームワーク**: Leptos 0.7
- **ビルドツール**: Trunk 0.21
- **スタイリング**: Tailwind CSS 3.x
- **状態管理**: Leptos Signals (組み込み) + leptos_query (サーバー状態キャッシュ)
- **ルーティング**: leptos_router (Leptos組み込み)
- **HTTPクライアント**: gloo-net 0.6 (fetch / WebSocket)
- **ローカルストレージ**: gloo-storage 0.3
- **セキュアストレージ**: web-sys + IndexedDB
- **WebSocket**: gloo-net 0.6
- **国際化**: leptos-i18n 0.5
- **アイコン**: icondata + leptos_icons (Lucide Icons含む)
- **ユーティリティ**: leptos-use 0.15
- **型シリアライズ**: serde + serde-wasm-bindgen

### インフラ

- **コンテナ**: Docker + Docker Compose
- **リバースプロキシ**: Nginx
- **データベース**: SurrealDB (コンテナ)
- **キャッシュ**: Dragonfly (コンテナ)

## 2. ディレクトリ構成

```text
mithic/
├── Cargo.toml
├── Cargo.lock
│
├── crates/
│   │
│   ├── server/                 # Axum main binary
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   │
│   ├── worker/                 # background worker
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   │
│   ├── frontend-web/           # Leptos CSR/WASM
│   │   ├── src/
│   │   │   ├── api/
│   │   │   ├── components/
│   │   │   ├── pages/
│   │   │   ├── state/
│   │   │   ├── ws/
│   │   │   ├── i18n/
│   │   │   ├── app.rs
│   │   │   └── main.rs
│   │   │
│   │   ├── style/
│   │   ├── public/
│   │   ├── index.html
│   │   ├── Trunk.toml
│   │   ├── tailwind.config.js
│   │   └── Cargo.toml
│   │
│   ├── shared/                 # DTO/shared types
│   │   ├── src/
│   │   │   ├── user.rs
│   │   │   ├── note.rs
│   │   │   ├── notification.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── core/
│   │   ├── src/
│   │   │   ├── models/
│   │   │   ├── services/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── db/
│   │   ├── src/
│   │   │   ├── schema/
│   │   │   ├── queries/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── api/
│   │   ├── src/
│   │   │   ├── routes/
│   │   │   ├── middleware/
│   │   │   ├── extractors/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── federation/
│   │   ├── src/
│   │   │   ├── actor/
│   │   │   ├── inbox/
│   │   │   ├── outbox/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── stream/
│   │   ├── src/
│   │   │   ├── ws/
│   │   │   ├── timeline/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── mfm/
│   │   ├── src/
│   │   │   ├── parser/
│   │   │   ├── renderer/
│   │   │   ├── ast/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── i18n/
│   │   ├── locales/
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   └── config/
│       ├── src/
│       │   └── lib.rs
│       └── Cargo.toml
│
├── docs/
├── infra/
├── scripts/
```

## 3. 各クレートの実装内容と現状

### `crates/core/src/models/` — エンティティ定義 (実装済み)

28エンティティ定義済み: actor, note, follow, notification, instance, file (DriveFile/DriveFolder), emoji, hashtag, poll, reaction, renote, bookmark, block, mute, filter, antenna, clip, user_list, relay, push_subscription, oauth, export, chart, used_username, user_note_pining, user_publickey, note_unread

### `crates/core/src/services/` — サービス層 (未実装)

ディレクトリは存在するが実装なし。`TODO.md` B-3 参照。

### `crates/api/src/routes/` — APIルート (未実装)

`mod.rs` が空。全エンドポイントがこれから実装。`TODO.md` B-1 参照。

### `crates/api/src/middleware/` — ミドルウェア (実装済み)

auth, cors, rate_limit, http_signature, content_negotiation, locale の7モジュール実装済み。

### `crates/db/src/` — データベース層 (スタブ)

SurrealDB / Dragonfly クライアントラッパーのみ。スキーマ定義・クエリ実装が未。`TODO.md` B-2 参照。

### `crates/federation/src/` — ActivityPub (スタブ)

`FederationService` 定義のみ。actor/inbox/outbox 実装が未。`TODO.md` B-6 参照。

### `crates/stream/src/` — WebSocket ストリーミング (設計済み)

チャンネルアーキテクチャ設計済み (HomeTimeline, GlobalTimeline, Hashtag, Admin, QueueStats, ServerStats, Drive, ApLog, UserList)。実装未。

### `crates/mfm/src/` — MFMパーサ (実装済み)

mention, hashtag, URL, code, bold, italic, strikethrough, quote, math を解析。カスタム絵文字・アニメーション等の高度機能は未実装。

### `crates/frontend-web/src/pages/` — フロントエンド画面 (UI実装済み)

HomePage, LocalTimelinePage, GlobalTimelinePage, StatusDetailPage, NotificationsPage, SearchPage, DmPage, DmConversationPage, ProfilePage, SettingsPage, LoginPage, SignupPage, AdminPage の13画面。サンプルデータ表示のみ、実API未接続。

### `crates/frontend-web/src/components/` — UIコンポーネント (実装済み)

Shell, TopBar, Sidebar, BottomNav, RightRail, Avatar, PostCard, PostBody, PostActions, MfmText, ComposeModal の11コンポーネント。

### `crates/frontend-web/src/api/` — APIクライアント (一部実装)

`auth.rs`, `notes.rs`, `client.rs` のみ。timeline, users, notifications, drive, messages 等が未実装。

### `crates/shared/src/` — 共有DTO (最小限)

Note, User, Notification, CreateNoteRequest, MediaAttachment, NoteVisibility のみ。API全体に必要な型の大半が未定義。

### `crates/server/src/main.rs` / `crates/worker/src/main.rs` — エントリポイント (スタブ)

起動ロジック未実装。`TODO.md` B-4, B-5 参照。

## 4. Misskey互換として製作していること

- **API互換性**: Mastodon API準拠で既存クライアントとの互換性確保
- **MFM対応**: Misskey Markup Language のパーサ (pest) とレンダラ (`mfm_renderer.rs`) を実装
- **機能セット**:
  - ノート（投稿）のリアクション、リノート
  - アンケート機能
  - ファイル添付（画像、動画等）
  - ユーザーリスト、アンテナ
  - チャンネル機能（予定）
  - ドライブ機能（ファイル管理）

## 5. ActivityPubに必ず準拠したつくりにすること

- **Actor実装**: Person タイプのアクターオブジェクト
- **WebFinger**: ユーザー発見プロトコル実装
- **HTTP Signatures**: すべての連携リクエストに署名 (`sigh`)
- **アクティビティ対応**:
  - Create, Delete, Follow, Undo
  - Accept, Reject, Announce, Like, Update
- **非同期配送**: Dragonfly を使用したアクティビティのキューイング
- **リトライ機構**: 指数バックオフによる失敗時の再試行

## 6. SNSとして過不足のないパフォーマンスを実現すること

### パフォーマンス対策

- **データベース**: SurrealDB の高パフォーマンスを活用
- **キャッシュ**: Dragonfly による積極的なキャッシュ
- **非同期処理**: Tokio による完全非同期設計
- **ストリーミング**: WebSocket によるリアルタイム更新 (gloo-net / tokio-tungstenite)
- **サーバー状態**: leptos_query による stale-while-revalidate キャッシュ
- **WASM最適化**: `opt-level = "z"` + LTO によるバイナリサイズ最小化
- **画像最適化**: WebP、サムネイル生成
- **CDN対応**: Trunk ビルド成果物の静的配信最適化

### スケーラビリティ

- **コンテナ化**: Docker による水平スケール容易化
- **負荷分散**: Nginx によるリバースプロキシ
- **型安全性**: `shared` クレートによるバックエンド・フロントエンド間の型保証

## 7. old-srcとの機能比較を行い、不足している機能がないかを常にチェックすること

### old-src（元 Misskey/Dolphin 実装）の主要機能

- Vue.js ベースのクライアント
- TypeScript サーバー実装
- 豊富なUIコンポーネント
- 高度な MFM 実装

### 比較チェックリスト

- [ ] MFM の全機能実装（カスタム絵文字、位置指定等）
- [ ] UI/UX のパリティ（ドラッグ＆ドロップ等）
- [ ] 管理者機能
- [ ] プラグインシステム
- [ ] API の完全互換性
- [ ] モデレーション機能

### 定期的な確認

- old-src の機能追加を監視
- 差分分析と実装優先度付け
- 互換性テストの実施

### `docs/feature-gap-analysis.md` と `TODO.md` の常時更新

**機能を実装したとき、または未実装の機能を発見したときは必ず `docs/feature-gap-analysis.md` と `TODO.md` を更新すること。**

- 機能を実装・完了したら、`TODO.md` の対応チェックボックスをチェック済みにする
- `docs/feature-gap-analysis.md` の対応項目を削除またはチェック済みに変更する
- 新たに不足を発見した場合は両ファイルの該当セクションに追記する
- バックエンド・フロントエンドどちらの変更でも対象
- `docs/feature-gap-analysis.md` 冒頭の「検証日」を更新日に書き換える

## 8. フロントエンドとバックエンドの機能分割を着実に行うこと

### 責務分離

- **バックエンド** (Axum / SurrealDB):
  - データ永続化
  - ビジネスロジック
  - フェデレーション処理
  - 認証・認可
  - ファイル処理

- **フロントエンド** (Leptos / WASM):
  - UI表示・コンポーネント
  - ユーザーインタラクション
  - クライアント側 Signals による状態管理
  - leptos_query によるサーバーデータキャッシュ
  - WebSocket によるリアルタイム更新表示

- **shared クレート**:
  - バックエンド・フロントエンド共通の型定義
  - serde によるシリアライズ定義

### API設計原則

- RESTful設計
- 状態レス性
- 適切な HTTP ステータスコード
- エラーハンドリングの統一
- レスポンス形式の標準化

## 9. 開発に関する追加事項

### セキュリティ

- **認証**: JWT トークンの有効期限管理
- **パスワード**: Argon2 による安全なハッシュ化
- **トークン保管**: IndexedDB (web-sys) によるセキュアなクライアント保管
- **HTTPS**: 本番環境での必須実装
- **CORS**: 適切なオリジン設定
- **レート制限**: API 乱用防止

### テスト

- **単体テスト**: Rust の組み込みテストフレームワーク
- **結合テスト**: APIエンドポイントのテスト
- **インテグレーションテスト**: フェデレーション機能のテスト
- **E2Eテスト**: Playwright / wasm-bindgen-test による UI テスト

### 国際化

- **対応言語**: 日本語、英語（優先）
- **バックエンド**: fluent + unic-langid (`crates/i18n/`)
- **フロントエンド**: leptos-i18n (`ja.ftl` / `en.ftl`)
- **翻訳管理**: `crates/frontend-web/src/i18n/` で一元管理

### デプロイ

- **Trunk ビルド**: `trunk build --release` で `dist/` に静的ファイル出力
- **Nginx**: `dist/` を静的配信、`/api/` をバックエンドへプロキシ
- **Docker Compose**: 開発・ステージング環境
- **Kubernetes**: 本番環境（推奨）
- **CI/CD**: GitHub Actions 等の自動化
- **監視**: ログ集約、メトリクス収集

### コントリビューション

- **コーディング規約**: rustfmt (バックエンド・フロントエンド共通)
- **コミット規約**: Conventional Commits
- **ドキュメント**: コードコメント、APIドキュメント
- **ライセンス**: AGPL-3.0 の遵守
