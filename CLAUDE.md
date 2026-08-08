# Mithic 開発ガイドライン

## 1. 使用している技術スタック

### バックエンド

- **言語**: Rust (edition 2024)
- **Webフレームワーク**: Axum 0.8
- **データベース**: SurrealDB 3.0
- **キャッシュ/キュー**: Dragonfly (Redis互換)
- **非同期ランタイム**: Tokio 1.0
- **認証**: JWT (`jsonwebtoken`) + Argon2 (パスワードハッシュ)
- **ActivityPub**: HTTP Signatures (`mithic_federation::http_sig` — RSA-SHA256 sign/verify 共通)
- **WebSocket**: Axum WebSocket
- **国際化**: fluent + unic-langid
- **Markdownパーサ**: comrak
- **Web Push**: web-push (VAPID; `VAPID_PRIVATE_KEY` 設定時に配送)

### フロントエンド

- **言語**: Rust (edition 2024)
- **フレームワーク**: Leptos 0.7
- **ビルドツール**: Trunk 0.21
- **スタイリング**: Tailwind CSS 4.x (Trunk standalone CLI, `Trunk.toml` で `tailwindcss = "4.x"`)
- **状態管理**: Leptos Signals (組み込み)
- **ルーティング**: leptos_router (Leptos組み込み)
- **HTTPクライアント**: gloo-net 0.6 (fetch / WebSocket)
- **トークン保管**: gloo-storage 経由の LocalStorage (`store/auth.rs`)
- **WebSocket**: gloo-net 0.6 (WebSocket) + gloo-timers 0.3 (再接続タイマー)
- **アイコン**: icondata + leptos_icons (Feather / Lucide Icons)
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
├── frontend/                # Leptos CSR/WASM (crate: frontend)
│   ├── src/
│   │   ├── api/
│   │   ├── components/
│   │   ├── models/
│   │   ├── pages/
│   │   ├── store/
│   │   ├── app.rs
│   │   └── main.rs
│   ├── style/
│   ├── public/
│   ├── index.html
│   ├── Trunk.toml
│   └── Cargo.toml
│
├── backend/                 # HTTP + ロジック + フェデレーション
│   ├── config/              # 環境変数設定
│   ├── i18n/                # fluent ロケール
│   ├── core/                # モデル + エラー型
│   ├── federation/          # ActivityPub 配送
│   ├── api/                 # ルート / middleware / services
│   │   └── src/routes/
│   │       ├── activitypub.rs
│   │       ├── ogp.rs
│   │       └── v1/          # mithic ネイティブ REST (`/api/v1/*`)
│   └── server/              # bin: mithic-server (HTTP + 連合配送)
│
├── db/                      # SurrealDB / Dragonfly / storage
├── shared/                  # front↔back 型契約 (wasm 対応)
│
├── docs/
├── infra/
└── scripts/
```

## 3. 各クレートの実装内容と現状

### `backend/core/src/models/` — エンティティ定義 (実装済み)

使用中のみ: actor, note, notification, file (DriveFile/DriveFolder), relay, activity

### `backend/core/src/services/` — サービス層

auth.rs のみ（JWT / Argon2 / TOTP）。

### `backend/core/src/misc/` — ユーティリティ (実装済み)

extract_emojis, extract_hashtags, extract_mentions の3モジュール実装済み。

### `backend/api/src/routes/` — APIルート

mithic ネイティブ REST (`routes/v1/`: auth, users, notes, timelines, notifications, drive, push, streaming, instance, admin) と ActivityPub / OGP。Misskey / Mastodon クライアント互換 API は持たない。

### `backend/api/src/middleware/` — ミドルウェア (実装済み)

auth, cors, rate_limit, http_signature の4モジュール実装済み。

### `backend/api/src/services/` — APIサービス層 (一部実装)

note.rs, user.rs, relationship.rs の3モジュール実装済み。

### `db/src/` — データベース層 (実装済み)

SurrealDB / Dragonfly クライアントラッパー実装済み。クエリモジュール (actors, drive, favorites, follows, hashtags, notes, notifications, polls, timeline) あり。

### `federation/src/` — ActivityPub 配送

`FederationService` (キュー配送・signed POST) + `http_sig` (署名/検証の単一実装)。

### WebSocket ストリーミング

`api/events.rs` の process-local broadcast + `/api/v1/streaming` が `shared::StreamEvent` を push。Misskey 風チャンネル抽象は撤去済み。

### `frontend/src/pages/` — フロントエンド画面 (UI実装済み)

`pages/mod.rs` に全画面を実装: HomePage, LocalTimelinePage, GlobalTimelinePage, StatusDetailPage, NotificationsPage, SearchPage, DmPage, DmConversationPage, ProfilePage, SettingsPage, LoginPage, SignupPage, WelcomePage, AdminPage, NotFoundPage の15画面。実APIとWebSocketに一部接続済み。

### `frontend/src/components/` — UIコンポーネント (実装済み)

Shell, TopBar, Sidebar, BottomNav, RightRail, Avatar, PostCard, PostBody, PostActions, MarkdownText, ComposeModal, Protected, LoadMore の13コンポーネント。レイアウトは3カラムレスポンシブ (drawer/sidebar + main + right-rail)。

### `frontend/src/api/` — APIクライアント (一部実装)

auth.rs, client.rs, notes.rs, users.rs, notifications.rs, dm.rs の6モジュール実装済み。

### `frontend/src/store/` — 状態管理 (実装済み)

auth.rs, compose.rs, notifications.rs, stream.rs, mod.rs で状態管理実装済み。

### `frontend/src/models/` — フロントエンドモデル (実装済み)

Note, User, Notification 等のモデル定義済み。

### `shared/src/types/` — 共有DTO (実装済み)

auth.rs, hashtag.rs, note.rs, notification.rs, stream.rs, user.rs の型定義。

### `shared/src/` — 共有コード (実装済み)

shared/src/types/ に DTO 定義 (auth, hashtag, note, notification, relay, stream, user)。shared/src/markdown.rs で comrak Markdown レンダラ実装。

### `backend/server` — エントリポイント

`mithic-server`: HTTP (Axum) と連合配送ワーカー (apalis) を同一プロセスで起動。

## 4. ActivityPub / フロント API

- **ActivityPub**: WebFinger / Actor / inbox (HTTP 署名必須) / outbox 等。配送は backend 内ワーカー + 指数バックオフ
- **フロント API**: `/api/v1/*` (`routes/v1/`) は **mithic ネイティブ REST** (WebUI/PWA 専用)。Misskey / Mastodon クライアント互換は持たない
- **連合**: 外部接続は ActivityPub のみ (Misskey 拡張: `_misskey_reaction` / `quoteUrl` 等)
- **Markdown**: comrak (AP `content` は HTML、`source` に原文)
- **機能セット**: リアクション、リノート、アンケート、ファイル添付、ドライブ

## 5. ActivityPubに必ず準拠したつくりにすること

- **Actor実装**: Person タイプのアクターオブジェクト
- **WebFinger**: ユーザー発見プロトコル実装
- **HTTP Signatures**: すべての連携リクエストに署名 (手書き RSA-SHA256)
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
- **WASM最適化**: `opt-level = "z"` + LTO によるバイナリサイズ最小化
- **画像最適化**: WebP、サムネイル生成
- **CDN対応**: Trunk ビルド成果物の静的配信最適化

### スケーラビリティ

- **コンテナ化**: Docker による水平スケール容易化
- **負荷分散**: Nginx によるリバースプロキシ
- **型安全性**: `shared` クレートによるバックエンド・フロントエンド間の型保証

## 7. `TODO.md` の常時更新

**機能を実装したとき、または未実装の機能を発見したときは必ず `TODO.md` を更新すること。**

- 機能を実装・完了したら、`TODO.md` の対応チェックボックスをチェック済みにする
- 新たに不足を発見した場合は該当セクションに追記する
- バックエンド・フロントエンドどちらの変更でも対象

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

- **対応言語**: 日本語（優先）、英語
- **バックエンド**: fluent + unic-langid (`i18n/`)
- **フロントエンド**: 現状は未着手。将来的に leptos_i18n の導入を検討

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

### Ponytail モード（怠惰なシニア開発者）

実装時は以下の優先順位で検討すること：

1. **YAGNI**: 本当に構築する必要があるか？
2. **既存コードの再利用**: このコードベースに既に存在するか？
3. **標準ライブラリ**: std が既に実行できるか？
4. **プラットフォーム機能**: ネイティブ機能で対応できるか？
5. **依存関係**: 既存の依存関係で解決するか？
6. **最小化**: 1行にできるか？
7. **最小コード**: その後にのみ、動作する最小限のコードを書く

- 余分な抽象化を追加しない
- 不要な新しい依存関係を追加しない
- 誰も求めなかったボイラープレートを追加しない
- 削除を追加より優先する。平凡さを好む
- 理解せずに小さな差分を適用するのは効率ではなく2番目のバグ
- 複雑な要求には質問する