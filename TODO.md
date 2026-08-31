# Mithic TODO

最終更新: 2026-08-31（UI: Misskey 左バー廃止、Deck シェル、DM/QR/高度な設定を削除）

凡例:

- **Done** — コードがあり、意図した経路で動く想定
- **Partial** — 入口はあるが、未配線・スタブ・未検証が残る
- **Missing** — 製品としてまだ無い / 意図的に後回し

---

## 1. いま動いているもの（Done）

### バックエンド基盤

- [x] Axum サーバ + 連合配送ワーカー同一プロセス (`mithic-server`)
- [x] SurrealDB スキーマ初期化 / Dragonfly 接続
- [x] JWT 認証 + トークン失効（actor.token 突合）+ サスペンド拒否
- [x] CORS / 圧縮 / Trace（許可オリジンに `localhost:3000` / `127.0.0.1:3000` / Trunk `:1420`。CorsLayer は最外層）
- [x] レート制限: Dragonfly `INCR` + `EXPIRE`（auth 系）
- [x] SSRF ガード（サーバー側 fetch）
- [x] 公開 GET の ETag / Cache-Control (`http_cache`)
- [x] HTTP Signature **sign/verify 共通化** (`mithic_federation::http_sig`)
- [x] プロセス内ストリーム (`events` broadcast) + `/api/v1/streaming`
- [x] YAGNI 掃除: 未配線 middleware、Misskey 風 stream クレート、推測モデル群、未使用 deps
- [x] クレート統合: backend 7 + db → `mithic-server` 1 クレート。`shared` は DTO + markdown のみ。`common/` は作らない
- [x] CI: rustfmt / clippy+test (`mithic-server` + `shared`) / frontend WASM check
- [x] TL DTO enrich の N+1 解消（添付/リノート/閲覧者リアクションをバッチ）
- [x] バックエンド i18n FTL を実際に使う error キーのみに縮小
- [x] DB: 未使用テーブル削除 (`word_mute` / `chart` / `meta` / `hashtag`)、重複インデックス削減
- [x] DB: ホーム TL の壊れた Sorted Set fan-out を削除。ホームは SQL、公開 TL は JSON キャッシュ
- [x] DB: `note.host` に合わせてローカル TL をインデックス可能に。ユーザーは `(username_lower, host)` UNIQUE。`follow`/`block`/`mute` は `(in, out)` UNIQUE + `is_accepted`

### REST API (`/api/v1/*`)

- [x] 認証: register / login / refresh / logout
- [x] ユーザー: me, 更新, パスワード変更, show, search, suggested, follow/block/mute, フォローリクエスト
- [x] ノート: 作成・表示・削除・返信/引用一覧・リアクション・リノート・お気に入り・ピン・投票・検索
- [x] タイムライン: home / local / global / hashtag / trending
- [x] 通知: 一覧・既読
- [x] ドライブ: アップロード / URL 取込 / 一覧 / 削除 / 配信 (`/uploads/{hash}`)
- [x] 画像サムネイル: WebP 最大 400px → オブジェクト `{hash}.thumb` + `thumbnail_url`
- [x] インスタンスメタ + 公開絵文字一覧
- [x] 管理: アカウント停止/解除/削除、リレー CRUD

### Web Push（コード上は Done）

- [x] DB: `push_subscription` テーブル + upsert/list/delete
- [x] API: `POST/GET/DELETE /api/v1/push/subscription`
- [x] `InstanceInfo.vapidPublicKey`（`VAPID_PRIVATE_KEY` 設定時）
- [x] 通知発行時に `deliver_web_push`（`publish_notification` から spawn）
- [x] Service Worker: `push` / `notificationclick`
- [x] 設定 UI: 「通知」→ 有効化 / 解除

> **運用前提**: `.env` に `VAPID_PRIVATE_KEY`（URL-safe base64 生鍵）と任意で `VAPID_CONTACT` が必要。未設定なら購読 API は動くが **配送は no-op**。

### ActivityPub（受信・配送の骨格）

- [x] WebFinger / NodeInfo / Actor / outbox / followers / following / featured
- [x] inbox（HTTP Signature 必須）: Follow, Undo, Like, Create, Announce, Question/投票系
- [x] 配送キュー (apalis-redis) + signed POST + ホスト並列制限 + 鍵キャッシュ

### フロント（画面はある）

- [x] 画面 UI（Deck ホーム / 詳細 / 検索 / プロフィール / 設定 / ログイン等）
- [x] 認証・投稿・タイムライン・通知・一部ユーザー操作が API 接続済み
- [x] WebSocket で note / notification / noteDeleted 受信
- [x] リノート前の注意ダイアログ（公開範囲の拡散注意 + プレビュー）
- [x] リアクションは投稿あたり1つ。別絵文字は置換、同じ絵文字の再押下は取り消し
- [x] 削除した投稿をタイムライン / 詳細 / プロフィールから即時除去（REST + AP Delete も broadcast）
- [x] プロフィール設定: バナー/アバター/名前/紹介/場所/誕生日/言語/追加情報/フォロー時メッセージ/リアクション受け入れ
- [x] Deck シェル: 上部バー + モバイル下部ドック。ホーム/ローカル/グローバル（+通知）を横並び。列の追加・削除・並び替えは localStorage
- [x] Service Worker: `/api/` は intercept しない（Workbox に載せると login/register が Failed to fetch になる）
- [x] 起動時は `/users/me` 成功まで未ログイン扱い。失効トークンで TL を叩かない
- [x] API クライアントは `RequestMode::SameOrigin`（CORS 不一致を Failed to fetch にしない）

---

## 2. 未完了・穴があるもの（ここが「できていない」）

### 2.1 コードはあるが未完成 / スタブ（Partial）

| 項目 | 現状 | 不足 |
|------|------|------|
| **AP Delete / Update / Accept / Reject / Block** | inbox で永続化 | リモート Delete は所有ノート削除 / Person は suspend。Update は Note・プロフィール。Accept/Reject は Follow のみ。Block は相互フォロー解除 + block 辺。object が URI のみの Accept/Update は未解決（fetch しない） |
| **AP fetch_remote_actor** | 実装あり。コメントに Phase F3 残り | SSRF 統一・キャッシュ・鍵取り回しの整理が甘い可能性 |
| **ノート作成時の poll** | `CreateNoteRequest.poll_choices` と `vote` API はある | **作成サービスが poll を DB に書かない**（リモート Create 経由の poll のみ強い） |
| **ピン留め** | バックエンド pin/unpin API + メニューから POST | プロフィールでのピン表示・解除 UI は未確認 |
| **ブロック / ミュート UI** | API + 投稿メニューから実行。設定に一覧 | 一覧からの解除ボタンは未接続 |
| **ドライブ「添付ノート」** | `GET .../files/{id}/notes` | **常に空配列**（逆引き未実装） |
| **Admin UI** | ルートはある | 画面は「管理機能は準備中」 |
| **DM** | **廃止** | ルート・ナビ・画面を削除済み。API も無し |
| **TOTP (2FA)** | core に generate/verify、DB にフィールド | **API ルートなし・設定 UI なし** |
| **フロント i18n** | なし（ハードコード日本語寄り） | leptos_i18n 等未導入 |
| **バックエンド locale middleware** | 削除済み | エラーは `DEFAULT_LOCALE` 固定。`Accept-Language` 未使用 |
| **DTO enrich** | 添付 `IN` / リノート著者 JOIN / 閲覧者リアクション一括。通知一覧も sender+note バッチ | 公開 TL JSON キャッシュと併用。マルチインスタンスは未着手 |
| **ストリーミング規模** | 全接続へ broadcast | 接続増時の per-user チャネル未着手 |
| **Web Push 実機検証** | 実装済み | 本環境での E2E（VAPID 発行〜ブラウザ通知）は未確認 |
| **サムネ実機検証** | 実装済み | 壊れた画像・巨大画像・透過・アニメ GIF の挙動は未確認 |
| **アイコン資産** | SW が `/icon-192.png` を参照 | public に無いと通知アイコン欠ける |

### 2.2 製品機能としてまだ無い（Missing / YAGNI 後回し）

優先して「今は作らない」もの。需要が出たら最小実装:

- [ ] ワードミュート / コンテンツフィルタ
- [ ] アンテナ
- [ ] クリップ
- [ ] ユーザーリスト
- [ ] OAuth（サードパーティ）— 方針上廃止
- [ ] エクスポート / インポート
- [ ] チャート / インスタンス統計 API
- [ ] ブックマーク（お気に入りと別概念が要るなら）
- [ ] 通報 (report)
- [ ] 引用ノート専用フロー（quote 通知型はあるが UI/作成経路が薄い）
- [ ] 予約投稿 (`scheduled_at` フィールドはあるが処理なし)
- [ ] NSFW / CW の一貫したフロント表示ルール
- [ ] フロント「設定」の残り（メール、連携アカウント、エクスポート、アカウント削除）
- [ ] Deck: ハッシュタグ列・列幅変更・サーバ保存（現状は 4 種 + localStorage）
- [ ] 設定 UI から外した猫 / Bot / QR（API フィールドは残っている）
- [ ] アバターデコレーション（素材カタログが要るので未着手）
- [ ] リノート確認からの引用フロー（CreateNoteRequest に renote_id が無い）

### 2.3 インフラ / 品質（Missing or Partial）

- [ ] 自動テスト: API 結合・AP 連合・WASM E2E がほぼ無い（`http_sig` 単体はあり）
- [ ] メトリクス / Prometheus（依存は落としたまま）
- [ ] 本番向け監視・バックアップ手順ドキュメント
- [x] MSRV / toolchain 文書の実態合わせ（`rust-version` 1.88。jsonwebtoken 11 が要求）
- [ ] 水平スケール時のストリーム: 現状 process-local broadcast のみ（複数 `mithic-server` では WS が共有されない）
- [x] Docker: `mithic-server` のコンパイル落ち修正。BuildKit cache を backend/frontend で分離、mold + cargo-chef バイナリ、コンテナ内 LTO オフで再ビルド短縮
- [x] SurrealDB 3: `user.fields` を `array<object> FLEXIBLE` に修正（`array FLEXIBLE` はパースエラー）
- [x] Caddy: `:3000` を `http://` + `bind 0.0.0.0` で IPv4 HTTP として listen。`/uploads/*` をバックエンドへ。hashed 以外の JS（`sw.js`）を immutable にしない。静的アセット欠落は SPA フォールバックしない
- [x] Windows+Podman: ホストの `127.0.0.1:3000` は `scripts/localhost_proxy.py` で WSL IP へ中継しないと login/register が Failed to fetch になる

---

## 3. 直近でやるなら（推奨順）

コードの穴を埋める順。新機能より **Partial の解消** 優先。

1. **ノート作成時の poll 永続化**  
   UI のアンケートと API のズレを解消。
2. **プロフィールでのピン表示**  
   ピン API はメニューから叩ける。プロフィール先頭への表示が残る。
3. **Web Push / サムネの手動 E2E**  
   `VAPID_PRIVATE_KEY` 設定 → 設定画面で有効化 → 通知発火。画像 UP → `.thumb` が返るか。
4. **TOTP**（必要なら）API + 設定 UI。
5. **ストリームの multi-instance**（スケールするとき）Dragonfly pub/sub 等。
6. **AP Update/Accept の URI-only object**  
   埋め込み無しの場合は現状スキップ。必要なら fetch を足す。

---

## 4. 意図的にやらないこと

- Misskey / Mastodon クライアント互換 API
- Misskey 風 WebSocket チャンネル抽象の復活
- 型だけ先に置く推測モデル（antenna/clip/oauth 等）— 機能実装時に最小から足す
- content negotiation ミドルウェアの再発明（AP はルートで Content-Type を付与）

---

## 5. 環境メモ

```bash
# Web Push を有効にする場合
# npx web-push generate-vapid-keys 等で private key (URL-safe base64) を用意
VAPID_PRIVATE_KEY=...
VAPID_CONTACT=mailto:admin@example.com
```

```bash
# ローカル確認の目安
cargo check -p mithic-server
cargo test -p mithic-server --lib
cargo check -p frontend --target wasm32-unknown-unknown
```
