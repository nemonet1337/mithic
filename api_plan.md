# mithic ネイティブ API 移行プラン

## 目的

Misskey / Mastodon のクライアント API 互換をすべて排除し、mithic 専用の単一 REST API (`/api/v1`) に統合する。

- クライアントは WebUI (Leptos/WASM) と PWA のみ。サードパーティクライアントは一切想定しない
- 外部との接続は ActivityPub のみ
- Misskey「対応」とはクライアント API 互換ではなく、**連合レベルで Misskey サーバーの機能 (絵文字リアクション・引用リノート等) と相互運用できること**を指す

## 現状分析

| ルート群 | 実体 | 処遇 |
|---|---|---|
| `api/src/routes/mastodon/v1.rs` (`/api/v1/*`) | 名前に反して **mithic 独自 REST API**。WebUI (`frontend-web/src/api/client.rs` の `api_base() = "/api/v1"`) が使用中 | **これが本体。残して拡張** |
| `api/src/routes/misskey/*` (`/api/notes/create` 等 POST 一辺倒) | Misskey 互換層。ただし drive / push / streaming / admin / follow requests / block / mute / hashtags / i 系など **v1 に無い機能の実装がここにしかない** | ハンドラロジックを v1 へ移植後、全削除 |
| `api/src/routes/misskey/oauth.rs`, MiAuth 系 | サードパーティクライアント向け | 完全削除 (移植不要) |
| `api/src/routes/activitypub.rs`, `federation/` | 連合 | 維持 + Misskey 拡張を追加 (後述) |
| `api/src/routes/ogp.rs` | OGP 配信 | 維持 |

作業の本質は 2 つ:

1. misskey ルータに残っている機能実装を v1 へ吸収し、`misskey/` `mastodon/` という名前空間を消す
2. Misskey 機能との相互運用を **ActivityPub 側** に実装する

## ターゲット構成

```
api/src/routes/
├── activitypub.rs      # 連合 (維持 + Misskey 拡張)
├── ogp.rs              # OGP (維持)
└── v1/                 # mastodon/v1.rs を分割・改名
    ├── mod.rs          # ルータ定義のみ
    ├── auth.rs         # login / register / refresh / logout
    ├── users.rs        # me / show / follow / block / mute / relation / search
    ├── notes.rs        # CRUD / reactions / renote / favorites / pin / polls / search
    ├── timelines.rs    # home / local / global / hashtag
    ├── notifications.rs
    ├── drive.rs        # upload / list / delete / upload-from-url
    ├── push.rs         # Web Push subscription
    ├── streaming.rs    # WebSocket (/api/v1/streaming)
    ├── instance.rs     # meta / custom emojis / trending hashtags
    └── admin.rs        # accounts suspend / relays
```

## API 設計原則 (v1 全体)

- **REST**: リソース名 + HTTP メソッド。Misskey 式「全部 POST」は廃止
- **認証**: `Authorization: Bearer <JWT>` のみ。MiAuth / OAuth / access_token ボディは廃止
- **エラー**: 既存の `{ status, code, message }` 形式に統一 (フロント `ApiError` がそのまま使える)
- **ページネーション**: `?until_id=` / `?since_id=` のカーソル方式に統一 (offset 禁止)
- **レート制限**: 既存 `rate_limit_middleware` を auth 系 + 書き込み系に適用
- **圧縮/キャッシュ**: 既存 CompressionLayer 維持。公開 GET に `Cache-Control` / ETag 付与
- **コンテンツ記法**: Markdown (comrak) を正とする。MFM は実装しない (連合で受信した MFM はプレーンテキスト+リンクとして扱う)

## エンドポイント一覧 (v1 完成形)

既存 v1 (✓) + misskey ルータから移植 (←移植) :

```
POST   /api/v1/auth/register|login|refresh|logout        ✓
GET    /api/v1/users/me                                  ✓
PATCH  /api/v1/users/me                                  ✓
POST   /api/v1/users/me/password                         ←移植 (change-password)
GET    /api/v1/users/check-handle                        ✓
GET    /api/v1/users/{username}                          ✓
GET    /api/v1/users/{username}/notes                    ✓
GET    /api/v1/users/{id}/relation                       ←移植
GET    /api/v1/users/{id}/following|followers            ←移植
POST/DELETE /api/v1/users/{id}/follow                    ✓
GET    /api/v1/follow-requests                           ←移植
POST   /api/v1/follow-requests/{id}/accept|reject        ←移植
DELETE /api/v1/follow-requests/{id}                      ←移植 (cancel)
POST/DELETE /api/v1/users/{id}/block                     ←移植
POST/DELETE /api/v1/users/{id}/mute                      ←移植
GET    /api/v1/blocks , /api/v1/mutes                    ←移植
GET    /api/v1/users/search?q=                           ←移植
POST   /api/v1/notes                                     ✓
GET/DELETE /api/v1/notes/{id}                            ✓
GET    /api/v1/notes/{id}/replies|quotes                 ✓
POST   /api/v1/notes/{id}/reactions                      ✓
DELETE /api/v1/notes/{id}/reactions/{emoji}              ✓
POST   /api/v1/notes/{id}/renotes                        ✓
DELETE /api/v1/notes/{id}/renote                         ←移植 (unrenote)
POST/DELETE /api/v1/notes/{id}/favorite                  ←移植
POST/DELETE /api/v1/notes/{id}/pin                       ←移植
POST   /api/v1/notes/{id}/vote                           ←移植 (polls)
GET    /api/v1/notes/search?q=                           ←移植
GET    /api/v1/timelines/{home|local|global}             ✓ (local/global を認証不要に)
GET    /api/v1/timelines/hashtag/{tag}                   ←移植
GET    /api/v1/hashtags/trending                         ←移植
GET    /api/v1/notifications                             ✓
POST   /api/v1/notifications/read-all                    ✓
POST   /api/v1/notifications/{id}/read                   ✓
POST   /api/v1/drive/files          (multipart)          ←移植
GET    /api/v1/drive/files?…        (find)               ←移植
GET/DELETE /api/v1/drive/files/{id}                      ←移植
POST   /api/v1/drive/files/from-url                      ←移植
GET    /api/v1/drive/files/{id}/notes                    ←移植 (attached-notes)
GET/POST/DELETE /api/v1/push/subscription                ←移植
GET    /api/v1/streaming            (WebSocket)          ←移植
GET    /api/v1/instance             (meta + カスタム絵文字一覧) 新規 (小)
POST   /api/v1/admin/accounts/{id}/suspend|unsuspend     ←移植
DELETE /api/v1/admin/accounts/{id}                       ←移植
GET/POST/PATCH/DELETE /api/v1/admin/relays               ←移植
GET    /api/v1/health                                    ✓
GET    /uploads/{hash}                                   ✓ (パス維持)
```

## ActivityPub レベルの Misskey 機能対応 (現状ゼロ、要実装)

コードベースに `_misskey_reaction` / `quoteUrl` の実装は現存しない。Misskey サーバーとの相互運用に必要なのは以下 (実装先は `federation/src/` と `api/src/routes/activitypub.rs`):

### 受信 (inbox)

- **絵文字リアクション**: `Like` アクティビティの `content` (`:emoji_name:` または Unicode) と `tag` (Emoji オブジェクト = カスタム絵文字の URL) を解釈し、reaction として保存。`content` 無しの素の `Like` は `⭐` 等のデフォルトリアクションにマップ
- **引用リノート**: `Note` の `quoteUrl` / `_misskey_quote` / FEP-e232 `quote` プロパティを引用として解決
- **カスタム絵文字**: 受信 Note / Like の `tag: [{type: "Emoji", icon: {url}}]` をリモート絵文字としてキャッシュ (`core/src/models/emoji.rs` は定義済み)
- **投票**: `Question` オブジェクトの受信と `Note` (name のみ) による投票の反映

### 送信 (outbox / 配送)

- **絵文字リアクション**: mithic のリアクションを `Like` + `content` + Emoji `tag` として配送 (Misskey がそのまま表示できる形式)
- **引用リノート**: `quoteUrl` と `quote` (FEP-e232) を併記して配送
- **Undo**: リアクション取消は `Undo(Like)`

### コンテキスト

- JSON-LD `@context` に Misskey 拡張 (`_misskey_reaction`, `_misskey_quote`, `quoteUrl`) を追加

## 移行フェーズ

### Phase 1: v1 への機能吸収 (ハンドラ移植)

misskey ルータのハンドラは services 層 (`api/src/services/`) を呼ぶだけのものが大半なので、ロジックはそのまま流用し、v1 側に RESTful なルート + extractor (Path/Query) を追加する。

1. `routes/mastodon/` → `routes/v1/` に改名し、v1.rs をリソース別ファイルに分割
2. streaming / drive / push を v1 に移植
3. follow-requests / block / mute / relation / search / polls / pin / favorite を移植
4. admin (suspend / relays) を移植
5. `GET /api/v1/instance` を新設 (インスタンス名・説明・カスタム絵文字一覧を 1 回で返す)

### Phase 2: フロントエンド追従

- `frontend-web/src/api/` の各モジュールを新パスに合わせる (client.rs は変更不要)
- streaming の接続先を `/api/v1/streaming` へ
- drive / 設定 / 管理画面の呼び出しを v1 パスへ

### Phase 3: 互換層の削除

1. `api/src/routes/misskey/` ディレクトリ全削除
2. `api/src/routes/mastodon/` の名残 (mod.rs) 削除、`routes/mod.rs` は v1::router のみ merge
3. MiAuth / OAuth 関連コード (`core/src/models/oauth.rs` 含む未使用部分) を削除
4. 未使用 DTO を削除
5. `cargo build` + フロント動作確認、TODO.md 更新

### Phase 4: ActivityPub Misskey 拡張

上記「ActivityPub レベルの Misskey 機能対応」を実装。受信 (inbox) → 送信 (配送) の順。federation クレートが現状スタブなので、inbox/outbox の基本実装と同時に進める。

### Phase 5: 高性能化 (必要になったものだけ)

- タイムライン系 GET に Dragonfly キャッシュ (既存 `db/src/cache.rs` を活用)
- 公開リソースに ETag / Cache-Control
- streaming イベント名の整理 (フロント `store/stream.rs` の期待と揃える)

## 削除されるもの (明示)

- Misskey クライアント API 全ルート (`/api/notes/create` 等) — 互換目的の API は一切残さない
- MiAuth / OAuth / アプリ登録
- Mastodon 互換を意図した命名・DTO (実体は既に独自なので実質リネーム)
- `access_token` をボディに含める認証方式
- MFM (Markdown に一本化済み)

## リスクと対策

- **連合の後方互換**: 既存 ActivityPub ルートは無変更。Misskey 拡張は追加のみなので他実装との接続に影響なし
- **ビルド継続性**: Phase 1 で新旧ルート併存 → Phase 2 でフロント切替 → Phase 3 で旧削除、の順なら常にビルド可能
- **絵文字リアクションの連合互換**: Misskey 系 (Misskey/Firefish/Sharkey) は `_misskey_reaction` 方式、Mastodon は素の `Like` しか解釈しない。送信は両対応 (`Like` + content) 一本で問題ない
