# Mithic 新規 TODO リスト (New_Todo.md)

**作成日**: 2026-06-13
**概要**: 古くなった `TODO.md`, `HowTo.md`, `docs` 以下のドキュメントから、すでに実装・修正が完了した項目を除外し、**現在も未実装または対応が必要な項目のみ**を抽出・整理した最新のタスク管理ドキュメントです。

---

## 0. ドキュメントの現状と古い情報の指摘
直近（2026-06-11）の大規模な実装およびE2Eテスト合格により、既存の `HowTo.md` や `docs/performance-optimization-plan.md` 等の一部の記述が実態と乖離して古くなっています。

* **すでに完了しており、古い注意点となっていた項目**:
  * コネクションプール（SurrealDB / Dragonfly）は実装済みです。
  * HTTP署名の生成（RSA-SHA256）および鍵キャッシュは実装済みです。
  * `/api/streaming` の接続・配信、および WebSocket サーバの実装は完了しています。
  * `fetch_remote_actor` の JSON-LD パースおよびアクター永続化は実装済みです。

---

## 1. パフォーマンス・アーキテクチャ関連の未完了タスク
`docs/feature-gap-analysis.md` および `docs/performance-optimization-plan.md` に基づく未解決の課題です。

- **[ ] キャッシュのホットパス統合 (P-G6)**
  - Dragonfly キャッシュヘルパ (`db/src/cache.rs`) は定義されていますが、各 API ルートや書き込みパスなどの実処理（タイムライン、ノート、ユーザー情報）への組み込みは未完了です。
  - `note:{id}` のボディキャッシュ（MGET）や、`noteresp:{id}` のプリレンダJSONキャッシュ、block/muteのセットキャッシュを統合する必要があります。
- **[ ] 配送処理の並列化の高度化 (P-G8 / P-G9)**
  - 現在4並列ワーカーは動作していますが、**ホスト単位のセマフォ（同時接続制限）** や **prefetch（ジョブのまとめて取得）** は未実装です。
  - **Dead Inbox Circuit Breaker (`dead_inbox:{host}`)** : 失敗回数の記録および閾値超過時の一時停止処理が未実装です。
- **[ ] ホスト単位のバッチ配送 (P-G10)**
  - 同一 `sharedInbox` 宛てのジョブをキュー投入前にグループ化・バッチ化する仕組みが未実装です（現状は inbox ごとに 1 ジョブ）。
- **[ ] リレー（Relay）連携と重複排除 (P-G13)**
  - `relay` / `activity` テーブルの定義と `activity.uri` による dedup 処理が未実装です。
  - リレー購読フロー（Subscribe/Unsubscribe）および `should_persist_note`（自インスタンスに関係あるノートのみ保存し、無関係なものは破棄する）ロジックが未実装です。
  - ノート作成時の `fanout_to_relays` が未実装です。
- **[ ] シリアライズとランタイム最適化 (P-G15)**
  - REST JSON での `simd-json` の適用、および WebSocket ペイロードでの MessagePack（`rmp-serde`）の適用が未着手です。
- **[ ] 可観測性 (P-G16)**
  - `metrics` + `metrics-exporter-prometheus` による各レイテンシやキャッシュヒット率の計測、および `tokio-console` の導入が未着手です。

---

## 2. 機能面での未完了タスク (Phase別)

### Phase 1 — 認証・共通処理
- **[ ] APIエラー共通ハンドリング** (401/400/422/500/ネットワーク) のフロント側実装。
- **[ ] 共通UIコンポーネント** (ローディング / 空状態 / エラー状態) の整備。
- **[~] ComposeModal 実投稿の UI 補完**:
  - 本文・公開範囲・CW・NSFW は送信されますが、添付ファイルID・投票・予約日時・返信先IDのUIが未実装です。

### Phase 2 — タイムライン＆データベース
- **[ ] データベーススキーマとインデックスの追加**:
  - `note.host` の非正規化および複合インデックス `(visibility,host,id)` の追加。
  - `follow` の双方向インデックス `(out,in)` / `(in,out)` の追加。
- **[ ] タイムライン Fan-out**:
  - `core/services/timeline.rs` における Fan-out on Write（フォロワー数が10,000未満はPush、10,000以上はPullのハイブリッド方式）の実装。
- **[~] フロントタイムライン (F-2) の高度化**:
  - 無限スクロールの自動化、および WebSocket 経由で差し込まれるイベントの重複排除が未実装です。
  - フロントの `api/timeline.rs` の整理（現状 `notes.rs` 内）。

### Phase 3 — ソーシャルグラフ＆通知
- **[ ] フォローリクエスト（鍵アカウント）の管理機能**:
  - `following/requests/{accept,reject,cancel,list}` の実装。
- **[ ] ブロック・ミュート一覧 API**:
  - `blocking/list` / `muting/list` の実装。
- **[ ] ユーザー設定 API**:
  - プロフィール更新（`i/update`）、パスワード変更（`i/change-password`）、トークン再生成（`i/regenerate-token`）、メールアドレス更新（`i/update-email`）の実装、およびフロント `api/i.rs` へのバインド。
- **[ ] ノートのピン留め**:
  - `i/pin` / `i/unpin` API および featured collection の公開。
- **[ ] サービス層の分離**:
  - フォロー/ブロック/ミュートのビジネスロジックを `core/services/user.rs` に分離・集約。
- **[~] 通知機能の補完**:
  - mention（メンション）通知の生成処理が未実装です。
  - ワードミュート/フィルター（`word_mute.rs`）、およびユーザー凍結（`suspend_user.rs`）が未実装です。

### Phase 4 — ドライブ＆メディア
- **[ ] ドライブファイル・フォルダ関連 API / クエリ**:
  - ファイル検索・URLからのアップロード・添付ノート取得（`drive/files/{find,upload-from-url,attached-notes}`）。
  - フォルダ作成・取得・削除（`drive/folders/{create,show,delete}`）。
- **[ ] サムネイル・メディア処理 (B-7)**:
  - 画像の WebP 変換およびサムネイル生成（`core/services/drive.rs`）。
  - 動画のサムネイル生成。
- **[ ] フロントエンド ドライブ機能 (F-5 / F-12)**:
  - ドラッグ＆ドロップ、アップロード進捗UI、プレビュー、ALTテキスト、ファイルマネージャー画面。
  - ライトボックス表示（`MediaImage`）、`MediaVideo`、`MediaList` などの表示コンポーネント。
  - フロント `api/drive.rs` の実装。

### Phase F1 — ActivityPub 受信基盤
- **[ ] Webルート・エイリアス対応**:
  - `/@:username` エイリアスでの Actor アクセス対応。
- **[ ] ActivityPub コレクション API**:
  - `GET /users/:id/{outbox,followers,following,collections/featured}` の実装。
- **[~] ActivityPub 受信アクションの補完**:
  - `POST /users/:username/inbox` + `POST /inbox` で、Create / Like / Announce などのアクティビティを実際にパースしてDBに反映する処理（現在は受理のみ）。
- **[ ] 署名検証の適用**:
  - inbox エンドポイントに HTTP 署名検証ミドルウェアを正しく適用する。

### Phase 6 — 二次機能（検索 / タグ / リスト / クリップ / チャンネル / アンテナ / 投票）
- **[ ] 検索 API / フロント (F-8)**:
  - `notes/{search,search-by-tag,mentions,children,replies,renotes,conversation,state}` のクエリおよびサービスの実装。
  - フロント `SearchPage` の実API化。
- **[ ] ハッシュタグ**:
  - トレンド、ハッシュタグタイムライン等のAPI実装。
- **[ ] ユーザーリスト**:
  - リスト CRUD API、リストタイムラインの実装、およびフロント側の接続。
- **[ ] クリップ (F-8b)**:
  - クリップ CRUD、ノート追加・削除APIの実装、およびフロント側の接続。
- **[ ] チャンネル (F-8c)**:
  - チャンネル CRUD、フォロー、チャンネルタイムラインの実装、およびフロント側の接続。
- **[ ] アンテナ**:
  - アンテナ CRUD、アンテナタイムラインの実装。
- **[ ] 投票 (F-14)**:
  - 投票API（`notes/polls/vote`）、集計処理、およびフロント `PollView`/`PollEditor` コンポーネントの実装。

### Phase 7 — 管理者機能・OAuth・Push
- **[ ] メタ・統計情報 API**:
  - インスタンス情報（`meta`）、詳細統計（`stats`）、Web Push 登録（`sw/{register,unregister}`）の実装。
- **[ ] チャート集計・取得**:
  - インスタンス、ノート、ユーザー、ドライブ、連合の時系列チャート取得API。
- **[ ] 管理者用管理 API (Admin)**:
  - アカウントの削除・凍結、カスタム絵文字管理、連合インスタンス管理、キュー管理、リレー管理、DBメンテナンス。
- **[ ] OAuth / 外部連携**:
  - OAuthアプリケーション管理、ログインセッション管理。
- **[ ] Web Push 配送サービス**:
  - VAPIDキーによるブラウザ宛 Web Push 送信処理。
- **[ ] フロント管理者画面 (F-9)**。

### Phase 8 — フロント機能拡張＆共通UI
- **[ ] 絵文字・リアクションピッカー (F-4)**:
  - リアクションピッカー、カスタム絵文字表示、キーボードナビゲーション。
- **[ ] ダイレクトメッセージ (F-6)**:
  - 会話一覧・詳細・送信フォーム（非公開ノート）の実API化、未読バッジのリアルタイム同期。
- **[ ] 設定画面の充実 (F-7 / 2FA)**:
  - プロフィール、プライバシー、2要素認証（2FA）等の設定画面およびバックエンド API。
- **[ ] オンボーディング (F-10)**:
  - ウェルカム画面（`/welcome`）とサインアップステップの実装。
- **[ ] MFM（マークアップ）高度機能 (F-11)**:
  - カスタム絵文字、位置指定アニメーション、KaTeX数式、URLプレビューのパースおよびレンダリング。
- **[ ] フロントUIコンポーネント補完 (F-13 / F-14)**:
  - `Autocomplete`, `Toast`, `RenotePicker`, `FollowButton`, `NoteMenu`, `VisibilityChooser` 等の各種コンポーネント。
- **[ ] 国際化 (i18n)**:
  - `leptos-i18n` の導入、`ja.ftl` / `en.ftl` の整備。

---

## 3. 品質ゲートと検証
開発を行う際は、以下の品質ゲートを必ず通してください。

```bash
cargo fmt --all
cargo clippy --all -- -D warnings
cargo check --all
cd frontend-web && trunk build
```
