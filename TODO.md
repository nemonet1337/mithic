# TODO — コードベース監査結果 (2026-07-26)

全クレート (api / config / core / db / federation / frontend-web / i18n / server / shared / stream / worker) を対象に、
処理の重複・コンパイラ警告・パフォーマンス・セキュリティ・冗長な処理を監査した結果。
`cargo clippy --workspace --all-targets`: **0 エラー / 91 警告**。

凡例: 🔴 = セキュリティ/重大バグ、🟠 = バグ・機能不全、🟡 = パフォーマンス、🔵 = 重複・冗長・警告

---

## 1. 🔴 セキュリティ (最優先)

### 1.1 ActivityPub inbox が HTTP 署名を検証していない
- 場所: `api/src/routes/activitypub.rs:36-37` (`/users/{username}/inbox`, `/inbox`)
- 問題: `verify_http_signature` ミドルウェア (`api/src/middleware/http_signature.rs`) は実装・テスト済みなのに、inbox ルートに **一切適用されていない**。誰でも任意のアクターを騙って Follow/Undo を POST できる。
- 修正: `activitypub::router()` の inbox 2 ルートに `axum::middleware::from_fn_with_state(state, verify_http_signature)` を layer する。

### 1.2 SurrealQL インジェクション (drive/files/find)
- 場所: `api/src/routes/misskey/drive.rs:207-216` (`find`)
- 問題: `name` / `mime_type` を `format!(" AND name CONTAINS '{}'", name)` で直接文字列連結。`'` を含む入力でクエリを破壊・改変できる。
- 修正: 他クエリ同様 `$name` / `$mime` の bind に置き換える (`AND name CONTAINS $name`)。

### 1.3 レートリミットが全く機能していない
- 場所: `api/src/middleware/rate_limit.rs` / `api/src/routes/mod.rs`
- 問題:
  - `rate_limit_middleware` がどのルーターにも layer されていない (`AppState.rate_limiter` は作られるだけ)。signin/signup へのブルートフォース、投稿スパムが素通し。
  - `cleanup()` を呼ぶ者がいないため、適用した場合も `HashMap` が無限に成長する (メモリリーク)。
  - キーが `X-Forwarded-For` / `X-Real-IP` の**信頼できないヘッダ**そのままなので、ヘッダ偽装で無限に回避できる。`unknown` フォールバックは全未知クライアントが 1 バケツを共有する。
  - `RateLimitConfig::burst_size` は未使用。`limit / 60` の整数除算で refill_rate=1 (limit<60 なら 0 = 補充されない)。
- 修正: 認証系 (`/api/signin`, `/api/signup`, `/oauth/*`) に最低限適用し、`tokio::spawn` で定期 cleanup、キーは `ConnectInfo<SocketAddr>` を基本に信頼済みプロキシのみ XFF を採用。burst_size を使うか削除。

### 1.4 admin 判定が常に false / admin ルートの権限チェック漏れ
- 場所: `api/src/middleware/auth.rs:41-45`、`api/src/routes/misskey/relays.rs`
- 問題:
  - `auth_middleware` が `AuthUser { is_admin: false, username: "" }` をハードコードして挿入。DB の `is_admin` を見ないため、`admin/mod.rs` の suspend/unsuspend/delete は**正規の管理者でも常に 403** (機能不全)。
  - 逆に `relays.rs` の add/list/remove/update (`/api/admin/relays/*`) は `is_admin` を**チェックしていない**ため、認証済みユーザーなら誰でもリレー設定を変更できる。
- 修正: auth_middleware で DB からユーザーを引いて `is_admin`/`username` を実値で埋める (キャッシュ可)。relays.rs 4 ハンドラに admin/mod.rs と同じ `if !auth.is_admin` ガードを追加。

### 1.5 OAuth 実装がスタブのまま公開されている
- 場所: `api/src/routes/misskey/oauth.rs`
- 問題:
  - `token()` はクライアント認証・認可コード検証を一切せず、誰でも POST するだけで「アクセストークン」を発行 (幸い JWT でないため実際には使えないが、API として嘘の成功を返す)。
  - `create_app()` は `client_id` と `client_secret` を**同一 ULID の前半/後半 8 バイト**から生成しており、公開値 (client_id + レスポンスの id) から secret が導出可能。
  - `authorize()` のコードは永続化されず検証不能。
- 修正: DB 永続化 (`core/src/models/oauth.rs` は定義済み) と検証を実装するまで、`/api/apps`, `/oauth/*` ルートを**ルーターから外す**。secret は `rand` による独立乱数で生成。

### 1.6 SSRF — upload-from-url と公開鍵フェッチ
- 場所: `api/src/routes/misskey/drive.rs:241-305` (`upload_from_url`)、`api/src/middleware/http_signature.rs:290-338` (`fetch_actor_public_key`)
- 問題: 任意 URL をサーバーがそのまま fetch する。内部ネットワーク (メタデータエンドポイント、Dragonfly、SurrealDB HTTP 等) へのアクセスに悪用できる。サイズ上限もなくメモリに全読み込み。
- 修正: スキーム http/https 限定、名前解決後のプライベート IP (127.0.0.0/8, 10/8, 172.16/12, 192.168/16, 169.254/16, ::1 等) を拒否、`Content-Length`/ストリーム読みでサイズ上限 (例 32MB) を設ける。

### 1.7 ファイルアップロードの上限・検証なし
- 場所: `api/src/routes/misskey/drive.rs:29-106` (`upload_file`)
- 問題: multipart を `field.bytes()` で**全量メモリに読む**。サイズ上限なし (axum 既定の 2MB 制限は Multipart 抽出器では効くが、明示設定がなく意図不明)。MIME はクライアント申告をそのまま信用し、`serve_upload` がその MIME で配信するため、`text/html` を申告すれば**同一オリジンで任意 HTML を配信 = ストアド XSS** が可能。
- 修正: `DefaultBodyLimit` を明示、MIME は magic bytes (`infer` クレート等) で判定するか許可リスト化、`serve_upload` に `Content-Disposition: attachment` か `X-Content-Type-Options: nosniff` + 画像/動画のみ inline を設定。

### 1.8 ノートの可視性 (visibility) が読み取り側で全く強制されていない
- 場所: `api/src/routes/misskey/notes.rs` (`show`, `search_notes`, `renote`)、`db/src/queries/timeline.rs` (`get_user_notes`)
- 問題:
  - `notes/show` は認証不要・可視性チェックなしで任意 ID のノートを返す (followers/specified 投稿も漏れる)。
  - `search_notes` は認証不要で **全ノート** (DM 相当の specified 含む) を全文検索できる。
  - `renote` は非公開ノートも Public でリノートできる。
  - `get_user_notes` も visibility フィルタなし。
- 修正: 読み取り系クエリに `visibility = 'public' OR (認証ユーザーに応じた条件)` を必ず付与する共通ヘルパを db 層に作り、各ルートから使う。

### 1.9 JWT シークレットのデフォルト値と CORS `*`
- 場所: `config/src/lib.rs:61-62, 72-74`
- 問題: `JWT_SECRET` 未設定時に `"change-me-in-production"` で**黙って起動**する。誰でもトークンを偽造できる。CORS も既定 `*`。
- 修正: `JWT_SECRET` 未設定 (またはデフォルト値のまま) なら `from_env()` でエラーにして起動拒否。CORS デフォルトは `instance_url` のみ。

### 1.10 トークン失効が機能していない
- 場所: `api/src/routes/misskey/auth.rs:53` (`signout`)、`i.rs:112` (`regenerate_token`)、`i.rs:76` (`change_password`)
- 問題: `user.token` カラムを更新するだけで、`auth_middleware` は JWT 署名しか見ないため、signout / token 再生成 / パスワード変更後も**旧 JWT は有効期限まで使える**。DB の token カラムは書くだけで読まれない (無駄な書き込み)。
- 修正: どちらかに統一する — (a) ステートレス JWT に割り切り DB token カラムと signout の token 更新を削除、または (b) auth_middleware で DB/Dragonfly の token と突き合わせて失効を有効化。(b) 推奨。

### 1.11 HTTP 署名検証の不備 (検証を適用した後の話)
- 場所: `api/src/middleware/http_signature.rs`
- 問題:
  - 署名対象ヘッダを送信側の `headers="..."` 指定のまま受け入れるため、`headers="date"` だけ署名したリクエストも通る (request-target/host/digest がリクエストに束縛されない)。
  - `to_bytes(body, usize::MAX)` で**無制限に body をバッファ** (DoS)。
  - digest 比較が非定数時間 (低リスクだが `subtle` で潰せる)。
  - 公開鍵フェッチで毎回 `reqwest::Client::builder()` を新規生成 — `state.http_client()` を使えばコネクションプールが効く。
  - `verify_digest` の `!parts[0].eq_ignore_ascii_case("sha-256") && !...("SHA-256")` は同じ条件を 2 回書いている冗長。
- 修正: POST では `(request-target)`, `host`, `date`, `digest` が署名対象に含まれることを必須化。body は上限付き (例 1MB) で読む。state の共有クライアントを使用。重複条件を 1 つに。

### 1.12 remote actor 保存時のなりすまし検証なし
- 場所: `api/src/routes/activitypub.rs:294-323` (`resolve_remote_actor`)、`federation/src/service.rs:591` (`parse_remote_actor`)
- 問題: 取得した JSON の `id` と取得元 URL の一致検証がなく、`preferredUsername` の衝突チェックもない。1.1 と併せると任意アクターの偽登録が可能。また `parse_remote_actor` が `Actor::new_local` を使い回すのは紛らわしい (private_key こそ無いが local 用初期値が混入するリスク)。
- 修正: `data["id"] == actor_url` を検証。`Actor::new_remote(...)` コンストラクタを追加。

---

## 2. 🟠 バグ・機能不全

### 2.1 フロントエンドと `/api/v1` のエンドポイント不一致 — 認証が動かない
- 場所: `frontend-web/src/api/auth.rs` ⇔ `api/src/routes/mastodon/v1.rs:43-82`
- 問題: フロントは `auth/signin` / `auth/signup` / `auth/me` / `auth/signout` / `2fa/*` を呼ぶが、バックエンドは `auth/login` / `auth/register` / `users/me` / `auth/logout` しか持たず、`2fa/*` ルートは**存在しない**。ログイン・サインアップ・起動時検証 (`fetch_me`) がすべて 404。
- 修正: パス名をどちらかに統一 (バックエンドに合わせるのが最小差分: フロント側 4 箇所の文字列変更)。2FA はルート実装まで UI から隠す。

### 2.2 `api/src/routes/v1/` ディレクトリ全体がデッドコード
- 場所: `api/src/routes/v1/` (13 ファイル: admin, auth, conversations, files, notes, notifications, polls, relationships, search, streaming, timelines, users, mod)
- 問題: `routes/mod.rs` は `activitypub / mastodon / misskey / ogp` しか宣言しておらず、`routes/v1/` は**コンパイルすらされていない** (git status で編集中だが未接続)。実際の `/api/v1` は `routes/mastodon/v1.rs` が担っており、mastodon という名前も実態 (フロント専用 API) と不一致。
- 修正: `routes/v1/` を削除するか、`mastodon/v1.rs` の内容をこちらへ移して配線し直す。`mastodon` モジュール名は `frontend_api` 等へリネーム (本物の Mastodon 互換 API は別途)。

### 2.3 OGP のバイトスライスでパニック (日本語ノートで即 500)
- 場所: `api/src/routes/ogp.rs:34-36, 105-107`
- 問題: `&text[..150]` は **UTF-8 文字境界を無視したバイトスライス**。日本語 (3 バイト/字) の 150 バイト目が境界でなければ panic。日本語優先の SNS では 150 バイト超のほぼ全ノートで落ちる。
- 修正: `text.char_indices().nth(150).map(|(i, _)| &text[..i]).unwrap_or(text)` 等、文字単位で切る。共通ヘルパ化して 2 箇所から使う。

### 2.4 ストリームイベントバスに購読者がいない — リアルタイム配信が届かない
- 場所: `api/src/events.rs` / `api/src/state.rs:104` (`subscribe_stream`) / `api/src/routes/misskey/streaming.rs`
- 問題: `publish_stream` (新規ノート・通知) は `broadcast::channel` に送るが、`subscribe_stream()` の呼び出し箇所が**ゼロ**。WebSocket ハンドラは別系統の `mithic_stream::StreamConnection` を使っており、投稿・通知イベントはどこにも配信されない。ストリーミング系が 2 系統併存している。
- 修正: `handle_socket` 内で `state.subscribe_stream()` も select し、`StreamBroadcast::Note` は購読チャンネルへ、`Notification` は user_id 一致時に送る。もしくは stream クレート側に一本化して events.rs を削除。

### 2.5 `t!` / `t_args!` マクロが存在しない `I18N` を参照
- 場所: `i18n/src/lib.rs:119-141`
- 問題: マクロが `$crate::I18N` (static) を参照するが、そのような static は定義されていない。使った瞬間コンパイルエラーになる地雷。また `core/src/error.rs:35` はエラーレスポンスの**たびに `I18n::new()`** を呼び、FTL 2 ファイルを毎回パースしている (パフォーマンス)。
- 修正: `pub static I18N: LazyLock<I18n> = LazyLock::new(I18n::new);` を i18n に追加し、error.rs もこれを使う。

### 2.6 queue_batch_delivery が同一ホストの配送を黙って捨てる
- 場所: `federation/src/service.rs:100-148`
- 問題: 「sharedInbox があれば使う」というコメントに反し、同一ホストに複数 inbox がある場合**先頭の 1 つに送って残りを破棄**している (`inboxes.into_iter().next()`)。個別 inbox 宛の配送が失われる。
- 修正: sharedInbox 判定は呼び出し元 (`broadcast_to_followers` が既に shared_inbox 優先で dedupe 済み) に任せ、この関数は全 inbox を push する。実質 `queue_delivery` と同一になるので**関数ごと削除**して `queue_delivery` を使うのが最短。

### 2.7 pin_note — 上限チェックが機能していないデッドクエリ
- 場所: `api/src/routes/misskey/notes.rs:275-284`
- 問題: `SELECT ... LIMIT 5` の結果を**捨てている** (件数チェックなし)。ピン上限が無く、同一ノートの重複ピンも可能。しかも `WHERE user_id = ...` は RELATION テーブルに存在しないフィールド (正しくは `in`)。
- 修正: `SELECT count() FROM user_note_pining WHERE in = ... GROUP ALL` で件数を取り、5 件以上なら Validation エラー。重複は `in, out` の UNIQUE インデックスで防ぐ。

### 2.8 attached_notes が常に空配列を返す
- 場所: `api/src/routes/misskey/drive.rs:307-324`
- 問題: クエリを実行して結果を捨て、`Ok(Json(Vec::new()))` を返す (TODO コメントあり)。DB への無駄クエリ + 嘘のレスポンス。
- 修正: 実装するか、実装まで `501 Not Implemented` を返しクエリを削除。

### 2.9 init_schema — block / mute テーブルを二重定義
- 場所: `db/src/surreal.rs:153-170` と `242-263`
- 問題: `block` / `mute` を `TYPE RELATION` で定義後、後段で `TYPE RELATION IN user OUT user` として**再定義**。定義が食い違い、後勝ちに依存している。
- 修正: 前段 (153-170) の定義を削除し、`IN user OUT user` + インデックス付きの後段だけ残す。

### 2.10 unrenote の renote_count 計算が壊れている
- 場所: `api/src/routes/misskey/notes.rs:245-252`
- 問題: `renote_count = <int>(renote_count OR 1) - 1` — count が 0 のとき `0 OR 1 = 1` となり `-1` ではなく 0 になる意図に見えるが、実際は 0→0、1→0、2→1 と DELETE 件数と無関係に常に 1 減算。複数リノートを一括 DELETE した場合に整合しない。また負値ガードとしても誤り。
- 修正: `renote_count = math::max(renote_count - {deleted_count}, 0)` 相当に。DELETE の RETURN で件数を取得して減算。

### 2.11 signin が凍結アカウントを拒否しない / 2FA を無視
- 場所: `api/src/services/user.rs:72-106`
- 問題: `is_suspended` チェックがなく、凍結ユーザーもログイン可能。totp_secret / totp_verified がスキーマにあるのに `requires_2fa` は常に `None` (2FA 未接続)。
- 修正: `authenticate_user` で `is_suspended` なら 403。2FA はサーバー側フロー実装 (temp_token 発行 → `verify_totp`) まで UI と DTO フィールドを外す。

### 2.12 worker のリトライ・指数バックオフが未構成
- 場所: `worker/src/main.rs:67-76`
- 問題: CLAUDE.md は「指数バックオフによる再試行」を要求するが、apalis Worker に RetryLayer/backoff 設定がなく、`#[allow(deprecated)] register_with_count` を使用。`DLQ_KEY` (`federation/src/service.rs:26`) も定義だけで未使用。失敗ジョブの行方が不定。
- 修正: apalis の `RetryPolicy` (backoff 付き) を layer し、最大試行超過で DLQ_KEY へ LPUSH。deprecated API を現行 API へ更新。

### 2.13 delete_account がユーザーレコードしか消さない
- 場所: `api/src/routes/misskey/admin/mod.rs:57-76`
- 問題: `DELETE user WHERE ...` のみで、ノート・フォロー関係・通知・drive_file・リアクション等が孤児として残る。連合への Delete アクティビティ配送もない。
- 修正: 関連テーブルを削除する一連のクエリ (またはバックグラウンドジョブ) にする。

### 2.14 webfinger / users.rs の未使用変数 (clippy 警告と一致)
- clippy: `unused variable: host`, `unused variable: notes`, `unused variable: i` — 途中まで書いて放置されたロジックの兆候。各箇所を確認して意図した処理 (可視性フィルタ等) を完成させるか変数を削除。

---

## 3. 🟡 パフォーマンス

### 3.1 search_notes の N+1 クエリ
- 場所: `api/src/routes/misskey/notes.rs:441-448`
- 問題: ノートごとに `get_actor_by_id` を await するループ。20 件で 21 クエリ。
- 修正: `timeline.rs` の `NOTE_WITH_AUTHOR_FIELDS` (`actor_id.* AS author`) パターンを再利用して 1 クエリにする。※ 同パターンが他ルート (users.rs 等) にもないか横展開して確認。

### 3.2 serve_upload がファイル全量をメモリに載せる
- 場所: `api/src/routes/misskey/drive.rs:176-186`
- 問題: `get_result.bytes()` で全読みしてから返す。動画などの大きいファイルでメモリを食い潰す。キャッシュヘッダもない。
- 修正: `get_result.into_stream()` を `Body::from_stream` で流す。`Cache-Control: public, max-age=31536000, immutable` (ハッシュ URL なので安全) を付与。本番は Nginx/CDN 直配信へ。

### 3.3 エラーレスポンス毎の I18n::new() (再掲 2.5)
- FTL パース × 2 が全エラーで発生。`LazyLock` 化で解消。

### 3.4 create_note_service のカウンタ更新・通知処理が直列
- 場所: `api/src/services/note.rs:60-120`
- 問題: author 取得 → カウンタ UPDATE → 返信通知 (get_note_by_id) → メンションごとに get_actor_by_username を**リクエスト内で直列 await**。メンションが多い投稿でレイテンシ増。
- 修正: 通知生成ブロックを fan-out 同様 `tokio::spawn` に逃がす (レスポンスに不要)。メンションのユーザー解決は `WHERE username_lower IN $names` の 1 クエリへ。

### 3.5 federation の無限成長キャッシュ
- 場所: `federation/src/service.rs:42-44` (`key_cache`, `host_semaphores`)
- 問題: ホスト数・アクター数に比例して無限成長。長期稼働でメモリリーク。
- 修正: `ponytail:` コメント付きで容認するか、`moka` 等の TTL/LRU キャッシュに差し替え (依存追加を避けるなら定期クリアの spawn で十分)。

### 3.6 shared_inbox 配信のブロードキャスト設計
- 場所: `api/src/events.rs` — `broadcast::channel(1024)`
- 問題: 全 WebSocket 接続が全ノートイベントを受け、各接続でフィルタする設計 (2.4 修正後)。接続数 × 投稿数の複製が発生。当面は可、スケール時にチャンネル別配信へ。
- 修正: `ponytail:` コメントで上限を明記しておく (接続 1000 超で per-user チャネルへ)。

### 3.7 frontend: PostCard ごとの comrak 実行
- 場所: `frontend-web/src/components/markdown.rs`
- 問題: 同一ノートの再レンダリングごとに Markdown→HTML 変換。WASM では comrak が重め。
- 修正: 現状は許容。タイムライン仮想化やメモ化 (`Memo`) 導入時に合わせて対応。

---

## 4. 🔵 処理の重複・冗長

### 4.1 JWT Claims / 検証ロジックが 3 箇所に重複
- 場所: `core/src/services/auth.rs:54-59` (Claims + verify_jwt)、`api/src/middleware/auth.rs:12-18, 50-64` (Claims + validate_token)、`api/src/routes/misskey/streaming.rs:26-47` (Claims + authenticate)
- 問題: 同じ Claims 構造体と検証が 3 実装。middleware 版は `exp < now` を手動チェックしているが jsonwebtoken の `Validation` が既に exp を検証しており冗長。streaming 版は exp 検証を Validation 任せにするなど微妙に挙動が違い、修正漏れの温床。
- 修正: `core::services::auth::verify_jwt` に `typ` チェックを足して唯一の実装にし、middleware / streaming はそれを呼ぶ。手動 exp チェック削除。

### 4.2 TOTP ビルダーの重複
- 場所: `core/src/services/auth.rs:13-30` と `33-52`
- 問題: `TOTP::new(Algorithm::SHA1, 6, 1, 30, ...)` の組み立てが 2 回。
- 修正: `fn build_totp(secret: Secret) -> Result<TOTP>` に抽出。

### 4.3 OGP テンプレートの重複
- 場所: `api/src/routes/ogp.rs:46-80` と `115-149`
- 問題: ほぼ同一の HTML テンプレート + truncate ロジックが note/profile で二重。
- 修正: `fn ogp_html(title, description, url, image: Option<&str>) -> String` + `fn truncate_chars(s, n)` に統合 (2.3 の修正と同時に)。

### 4.4 Relay DTO 変換の重複
- 場所: `api/src/routes/misskey/relays.rs` — `Relay { id, inbox, status, created_at, updated_at }` の組み立てが 3 箇所 + `status_to_dto`/`status_from_dto` の手書きマッチ。
- 修正: `impl From<core::Relay> for shared::Relay` を 1 箇所に。status は両 enum が同型なので `From` 実装 2 つで十分。

### 4.5 SurrealConfig の組み立て重複
- 場所: `server/src/main.rs:25-31` と `worker/src/main.rs:33-39`
- 問題: `AppConfig` → `SurrealConfig` の写経が 2 箇所。
- 修正: `impl From<&AppConfig> for SurrealConfig` を db または config に置く。tracing 初期化・dotenv 等の起動前処理も `mithic_config::init()` 的な 1 関数にまとめられる。

### 4.6 reqwest::Client 構築の重複
- 場所: `api/src/state.rs:49-53`、`worker/src/main.rs:48-52`、`api/src/middleware/http_signature.rs:304-307`
- 問題: 同じ builder 設定 (pool_max_idle 32 / idle_timeout 90s) が複製され、http_signature は毎リクエスト新規生成。また `unwrap_or_default()` は設定エラーを黙って握り潰す。
- 修正: `fn build_http_client() -> reqwest::Client` を 1 箇所に。builder 失敗は `?` で伝播。

### 4.7 actor 行→Actor デシリアライズの重複
- 場所: `federation/src/service.rs:503-515`、`api/src/routes/activitypub.rs:296-309` (ほか db/queries にも同型があるはず)
- 問題: `query → take → into_json_value → strip_record_prefixes → from_value::<Actor>` の 5 行イディオムが散在。
- 修正: `db::queries::get_actor_by_uri(client, uri) -> Result<Option<Actor>>` を追加して置き換え。

### 4.8 cache.rs の block/mute 関数群の重複
- 場所: `db/src/cache.rs:113-190`
- 問題: `block_set_*` と `mute_set_*` はキープレフィックスだけ違う同一実装 × 8 関数。`get_json_with_metrics` も `get_json` とほぼ同一。
- 修正: プレフィックスを引数に取る内部関数 (`set_add(kind, ...)`) に畳む。metrics 版は使用箇所を確認し、未使用なら削除 (CacheMetrics 自体 `/metrics` に未接続)。

### 4.9 /metrics がダミー
- 場所: `api/src/routes/mod.rs:28-36`
- 問題: `"ok\n"` を Prometheus Content-Type で返すだけ。CacheMetrics も QueueStats も接続されていない。
- 修正: 監視を入れるまでルートごと削除するのが正直 (嘘の 200 を返さない)。

### 4.10 config の `jwt_secret()` getter
- 場所: `config/src/lib.rs:95-97`
- 問題: pub フィールドに対する意味のない getter (呼び出し側も `config.jwt_secret` と混在)。
- 修正: 削除してフィールド直参照に統一。

---

## 5. 🔵 コンパイラ警告 (clippy 91 件) — frontend-web に集中

`rtk cargo clippy --workspace --all-targets` の内訳。バックエンドは `api` の unused import 2 件のみで、残りはほぼ frontend-web。

### 5.1 未使用 API クライアント関数 (実装済みだが UI 未接続)
- `api/notes.rs`: `fetch_note`, `delete_note`, `renote`, `add_reaction`, `remove_reaction`, `fetch_replies`, `fetch_quotes`
- `api/dm.rs`: `fetch_conversations`, `fetch_messages`, `send_message`, `create_conversation`, `mark_read` (+ `Conversation`, `DirectMessage` 構造体)
- `api/auth.rs`: `refresh`, `logout`, `activate_2fa` (+ `TwoFactorActivateRequest`, `qr_code_url`, `refresh_token` フィールド)
- `api/client.rs`: `request_with_retry`
- → **判断が必要**: DM・2FA はバックエンドにルートが無い (2.1) ので、画面ごと未完成。接続する予定が近いなら残し、そうでなければ削除して復活は git に任せる (Ponytail 原則)。

### 5.2 未使用コンポーネント/フィールド (props が読まれていない = 表示が壊れている可能性)
- `FollowButton` (`is_following` 等 4 fields 未読 + mod.rs で unused import) — フォローボタンがどこにも置かれていない
- `NoteMenu` (`is_open`/`on_action`/`on_close` 未読、`Unpin` variant 未構築)
- `RenotePicker` (`current`/`on_change` 未読)、`ReactionPicker` (`on_select` 等未読)
- `Toast` (`ToastStore::new`/`push` 未使用、`Info/Success/Warning/Error` variant 未構築) — トースト通知が発火されていない
- `Autocomplete` (全 props 未読)、`EmptyState`/`ErrorState`/`LoadingSpinner`/`MediaList` (unused import)
- `store/compose.rs`: `InlineTop`/`FullscreenWriting` variant、`store/stream.rs`: `StreamEvent` enum 未使用、`SeenNoteBuffer` 未構築
- → これらは「作ったが配線していない」UI。各画面への組み込みを完了するか、コンポーネントごと削除。

### 5.3 機械的に潰せる警告
- `passing a unit value to a function` ×9 / `unneeded unit expression` ×8 / `unnecessary parentheses around closure body` ×4 (post_card.rs, shell.rs, pages/mod.rs, protected.rs)
- `unused import: AppError` (`misskey/oauth.rs:8`, `misskey/push.rs:5`)
- `Iterator::last` on DoubleEndedIterator → `next_back()` に
- `useless use of format!`、`redundant closure`、`let-binding has unit value`、`large size difference between variants` (Box 化)
- 修正: `cargo clippy --fix --workspace --allow-dirty` で大半自動修正 → 残りを手当て → CI に `-D warnings` を追加して再発防止。

---

## 6. その他 (設計・ドキュメント整合)

- [ ] **CLAUDE.md と実態の乖離**: Tailwind は 3.x 表記だが Trunk.toml は `tailwindcss = "4.1.14"`。「トークン保管: IndexedDB」とあるが実装は `gloo_storage::LocalStorage` (`store/auth.rs`)。ディレクトリ構成の `db/queries` 一覧等も現状とズレ。ドキュメントを現実に合わせて更新する。
- [ ] **`routes/mastodon` の命名** (2.2 参照): Mastodon API v1 互換は実装されていない。README/CLAUDE.md の「Mastodon API v1 準拠」記述も現状は誤り。
- [ ] **AP 配送の `content` が生 Markdown**: `build_create_activity` (`api/src/services/note.rs:245`) は `note.text` をそのまま `content` に入れる。AP の `content` は HTML 想定なので、`shared::markdown::render_markdown` を通す (comrak デフォルトは生 HTML をエスケープするので XSS 面は安全) + `source` フィールドに原文を入れる。
- [ ] **ノート本文の長さ制限なし**: `CreateNoteRequest.text` に上限バリデーションがない (DB とタイムラインを巨大テキストで汚染可能)。3000 文字程度で Validation エラーに。
- [ ] **`should_persist_note` が常に true** (`federation/src/service.rs:335-342`): リモートノート全保存はディスク爆発の元。フォロー関係・メンションに限定する実装を入れるまで inbox の Create 受理 (Phase F3) を始めないこと。
- [ ] **`AppState::new` の storage 引数名の紛らわしさ**: `storage: RedisStorage` (キュー) と `let storage = create_storage_client` (オブジェクトストレージ) が同名でシャドーイング。`queue_storage` / `object_storage` にリネーム。
- [ ] **update_email がメール検証・重複チェックなし** (`i.rs:136`): 確認メールフローは無くても、最低限フォーマット検証と既存メール重複チェックを。
- [ ] **followers/following/outbox コレクションが totalItems: 0 固定** (`activitypub.rs:230-272`): 他実装からはフォロワー 0 に見える。実カウントを返す (ページングは後回しで可)。
- [ ] **Dockerfile / docker-compose.release.yml 削除の後始末**: `Dockerfile.release`・`docker-compose.release.yml`・`.github/workflows/ci-deploy.yml` が削除されている。README のデプロイ手順が現存ファイルと一致しているか確認。

---

## 推奨着手順

1. **§1 セキュリティ** — 特に 1.1 (署名検証)、1.2 (インジェクション)、1.4 (admin)、1.7 (XSS)、1.9 (JWT secret) は連合を有効にする前に必須。
2. **§2.1–2.5** — フロントの認証不一致と OGP パニックはユーザー影響が即時。
3. **§5.3** — clippy --fix と CI の `-D warnings` 化 (30 分で終わり、以後の腐敗を防ぐ)。
4. **§4** — 重複統合はセキュリティ修正のついでに該当ファイルを触るとき一緒に。
5. **§3 / §5.1–5.2** — 機能接続の方針 (DM・2FA・トーストを作り切るか消すか) を決めてから。
