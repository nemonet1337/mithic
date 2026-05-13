# Mithic フロントエンド実装指示書

**Webフロントエンド — Leptos + TailwindCSS 構成**

---

## 1. 技術スタック

| クレート | バージョン | 用途 |
|---|---|---|
| Leptos | 0.7 | UIフレームワーク (Rust/WASM) |
| Leptos Signals | 0.7 | リアクティブ状態管理 |
| Leptos Router | 0.7 | クライアントサイドルーティング |
| Reqwest | 0.12 | HTTPクライアント (WASM対応) |
| gloo-storage | 0.3 | JWTトークン保管 (LocalStorage) |
| Leptos Icons | 最新 | アイコン |
| TailwindCSS | 3.x | スタイリング |
| cargo-leptos | 最新 | ビルドツール / 開発サーバー |

---

## 2. リポジトリ配置

Cargoワークスペース内の `crates/frontend-web/` に配置します。

```text
mithic/
└── crates/
    ├── frontend-web/        # このドキュメントのスコープ
    │   ├── src/
    │   │   ├── main.rs
    │   │   ├── app.rs       # ルートコンポーネント・ルーティング定義
    │   │   ├── api/         # Reqwestを使ったAPIクライアント
    │   │   ├── components/  # 共通UIコンポーネント
    │   │   ├── pages/       # ページ単位のコンポーネント
    │   │   ├── store/       # グローバルSignal (認証状態等)
    │   │   └── models/      # sharedクレートの型をラップ
    │   ├── style/
    │   │   └── main.css     # TailwindCSSエントリポイント
    │   ├── Cargo.toml
    │   └── Trunk.toml       # or cargo-leptos.toml
    └── shared/              # バックエンドと共有する型定義
```

---

## 3. Cargo.toml

```toml
[dependencies]
leptos = { version = "0.7", features = ["csr"] }
leptos_router = { version = "0.7", features = ["csr"] }
reqwest = { version = "0.12", features = ["json", "wasm"] }
gloo-storage = "0.3"
leptos-icons = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
shared = { path = "../shared" }

[features]
default = ["csr"]
csr = ["leptos/csr", "leptos_router/csr"]
ssr = ["leptos/ssr", "leptos_router/ssr"]
```

---

## 4. ルーティング定義

`Leptos Router` を使い、以下のルートを定義します。

| パス | ページ | 認証必須 |
|---|---|---|
| `/` | タイムライン (ホーム) | ✅ |
| `/local` | ローカルタイムライン | ✅ |
| `/global` | グローバルタイムライン | ✅ |
| `/notifications` | 通知 | ✅ |
| `/search` | 検索 / 発見 | — |
| `/messages` | DM 受信箱 | ✅ |
| `/messages/:id` | DM 会話 | ✅ |
| `/:handle` | プロフィール | — |
| `/settings` | 設定 (トップ) | ✅ |
| `/settings/:section` | 設定 (各セクション) | ✅ |
| `/login` | ログイン | — |

```rust
// app.rs (概略)
#[component]
pub fn App() -> impl IntoView {
    provide_context(AuthStore::new());
    view! {
        <Router>
            <Routes>
                <ProtectedRoute path="/" view=HomePage />
                <ProtectedRoute path="/notifications" view=NotificationsPage />
                <ProtectedRoute path="/messages" view=DmPage />
                <ProtectedRoute path="/messages/:id" view=DmConversationPage />
                <ProtectedRoute path="/settings" view=SettingsPage />
                <ProtectedRoute path="/settings/:section" view=SettingsPage />
                <Route path="/search" view=SearchPage />
                <Route path="/:handle" view=ProfilePage />
                <Route path="/login" view=LoginPage />
            </Routes>
        </Router>
    }
}
```

---

## 5. グローバル状態管理 (Signals)

`store/` にグローバルなSignalをまとめ、`provide_context` で注入します。

```rust
// store/auth.rs
#[derive(Clone)]
pub struct AuthStore {
    pub token: RwSignal<Option<String>>,
    pub me: RwSignal<Option<User>>,
}

impl AuthStore {
    pub fn new() -> Self {
        // gloo-storageからJWTを復元
        let token = LocalStorage::get("token").ok();
        Self {
            token: RwSignal::new(token),
            me: RwSignal::new(None),
        }
    }

    pub fn login(&self, token: String, user: User) {
        LocalStorage::set("token", &token).unwrap();
        self.token.set(Some(token));
        self.me.set(Some(user));
    }

    pub fn logout(&self) {
        LocalStorage::delete("token");
        self.token.set(None);
        self.me.set(None);
    }
}
```

---

## 6. APIクライアント

`api/` 配下にエンドポイントごとのモジュールを置きます。Authヘッダーは共通関数で付与します。

```rust
// api/client.rs
pub fn authed_client(token: &str) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", token).parse().unwrap(),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

// api/notes.rs
pub async fn fetch_timeline(token: &str, since_id: Option<&str>) -> Result<Vec<Note>, reqwest::Error> {
    authed_client(token)
        .get("/api/v1/timeline")
        .query(&[("since_id", since_id)])
        .send()
        .await?
        .json::<Vec<Note>>()
        .await
}

pub async fn create_note(token: &str, body: &CreateNoteRequest) -> Result<Note, reqwest::Error> {
    authed_client(token)
        .post("/api/v1/notes")
        .json(body)
        .send()
        .await?
        .json::<Note>()
        .await
}
```

---

## 7. 共通コンポーネント

### 7.1 レイアウト

全ページで共通するシェル。サイドバー・メインエリア・右レールで構成します。

```
+------------------+------------------------+------------------+
|   Sidebar        |   <slot />             |   RightRail      |
|   (固定)          |   (ページごとに変化)      |   (オプション)     |
+------------------+------------------------+------------------+
```

**Sidebar** — ナビゲーションアイコン + ラベル。デスクトップは常時表示、モバイルはボトムバーに変化。

アイコンと遷移先:
- ホーム (`/`)
- 検索 (`/search`)
- 通知 (`/notifications`) — 未読バッジ付き
- DM (`/messages`) — 未読バッジ付き
- プロフィール (`/:handle`)
- 設定 (`/settings`)
- 投稿ボタン (Composeモーダルを開く)

**RightRail** — トレンドタグ、フォロー推薦。768px以下では非表示。

### 7.2 PostCard

タイムラインや返信一覧で使う投稿カードです。

```rust
#[component]
pub fn PostCard(note: Note) -> impl IntoView {
    view! {
        <article class="flex gap-3 p-4 border-b border-line hover:bg-card/40 transition">
            <Avatar user=note.author.clone() />
            <div class="flex-1 min-w-0">
                <PostHeader user=note.author time=note.created_at />
                <PostBody content=note.content />   // MFMレンダリング
                <PostActions note=note />
            </div>
        </article>
    }
}
```

**PostActions** に含むボタン: 返信 / リノート / リアクション / 引用 / 共有

### 7.3 ComposeModal

新規投稿モーダル。サイドバーの投稿ボタンで開閉します。

**対応する入力項目:**

| 項目 | 仕様 |
|---|---|
| 本文 | 最大500文字、カウンター表示 |
| 公開範囲 | 公開 / ホーム / フォロワー / 指定ユーザー |
| CW (コンテンツ警告) | ON/OFFトグル + テキスト入力 |
| NSFW | トグル |
| 添付ファイル | 画像・動画 最大4ファイル・100MB。ドラッグ&ドロップ対応 |
| 投票 | オプション追加・期限設定 |
| 絵文字ピッカー | カスタム絵文字 + Unicode対応 |
| 予約投稿 | 日時選択 |
| 下書き | gloo-storageに自動保存 |

**バリアント:**
- `A · centered modal` — バックドロップつきモーダル（デフォルト）
- `B · inline at top` — タイムライン最上部にインライン展開
- `C · fullscreen writing` — 集中執筆モード（フルスクリーン）

### 7.4 Avatar

アバター画像。未設定時はイニシャルプレースホルダー。

```rust
#[component]
pub fn Avatar(user: User, #[prop(default = AvatarSize::Md)] size: AvatarSize) -> impl IntoView {
    // ...
}

pub enum AvatarSize { Sm, Md, Lg, Xl }
```

---

## 8. 画面仕様

### 8.1 ホーム (タイムライン)

**バリアントA — クラシック3カラム (デフォルト候補)**

- 左: Sidebar
- 中央: タイムラインフィード + TopBar
  - タブ: フォロー中 / ローカル / グローバル
  - 投稿カードを時系列で表示
- 右: RightRail

**バリアントB — マガジン風**

- コンパクトSidebar
- 2カラムグリッドのカードレイアウト
- ピン留め投稿をワイドカードで強調表示
- 日付ヘッダー付き

**バリアントC — タイムラインレール**

- 左端に時刻マーカー + 縦線を配置
- 投稿を時系列に並べる実験的レイアウト
- 時系列 / 注目順の切り替えボタン

**実装上の注意:**
- WebSocketで新着ノートをリアルタイム受信し、Signalでタイムラインに差し込む
- 無限スクロールは Intersection Observer (JS interop) で実装
- タイムラインはDragonflyキャッシュを経由するため、初回ロードは高速

---

### 8.2 投稿詳細

**バリアントA — クラシックスレッド**

- 元投稿をヘッダーとして大きく表示
- 統計 (リノート数・いいね数・引用数) を横並びで表示
- アクションボタン (返信 / リノート / リアクション / 共有)
- 返信一覧を下に表示

**バリアントB — スレッドビュー**

- アバターの下に縦線を引いてスレッドを可視化
- 返信ごとにインデントせずに縦線のみで親子関係を表現
- 返信入力欄をスレッドの末尾に配置

**バリアントC — リアクションスプリット (デフォルト候補)**

- 左カラム: 元投稿 + 返信コンポーザー + 返信一覧
- 右カラム: リアクション一覧 + 引用一覧
- 引用には元投稿を埋め込み表示

**返信コンポーザーに含む要素:**
- 返信先ハンドル表示
- テキスト入力 (`@handle への返信`)
- 添付ボタン / 絵文字ボタン
- 返信送信ボタン

---

### 8.3 投稿作成 (Compose)

詳細は [7.3 ComposeModal](#73-composemodal) を参照。

**キーボードショートカット:**

| ショートカット | 動作 |
|---|---|
| `⌘ + Enter` | 投稿 |
| `⌘ + S` | 下書き保存 |
| `Esc` | モーダルを閉じる |
| `⌘ + K` | コマンドパレット (検索C) を開く |

---

### 8.4 通知

**バリアントA — クラシックリスト**

- タブ: すべて / @メンション / いいね / フォロー
- 未読は背景色で強調 + 左端に赤ドット
- 「すべて既読」ボタン

**バリアントB — タイプ別グループ**

- いいね・フォロー・返信/引用をグループカードにまとめて表示
- グループカード内にアバターを横並びで列挙
- 展開ボタンで詳細を開ける

**バリアントC — アクティビティストリーム (デフォルト候補)**

- 通知ごとにリッチプレビューカード
- いいね: 対象投稿のプレビューテキストを引用表示
- 返信: 返信本文を表示 + 「返信」「開く」ボタン
- フォロー: 共通フォロワー数 + 「フォローバック」ボタン

---

### 8.5 検索 / 発見

**バリアントA — 検索結果**

- 検索バー常時表示 (`⌘K` ショートカット)
- タブ: すべて / 投稿 / ユーザー / タグ / メディア
- 投稿カードと関連ユーザーを混在表示

**バリアントB — ディスカバーグリッド**

- 検索前のランディング画面
- 急上昇タグをピル形式で表示
- カテゴリグリッド (Art / Tech / Books / Music / Food / Photo)

**バリアントC — コマンドパレット (デフォルト候補)**

- `⌘K` で全画面からオーバーレイ表示
- ユーザー / タグ / アクションをリスト形式で表示
- キーボード (`↑↓` 移動、`↵` 開く、`Esc` 閉じる) で操作完結
- 検索ワードで新規投稿を直接作成するアクション

---

### 8.6 DM (ダイレクトメッセージ)

**バリアントA — 2ペイン (デフォルト候補)**

- 左ペイン: 会話一覧 (260px固定) + 検索バー
  - 未読は右端に赤ドット
  - グループDM対応
- 右ペイン: 会話ビュー
  - メッセージバブル (相手: 左、自分: 右)
  - 日付セパレーター
  - 入力欄 + 添付・絵文字ボタン

**バリアントB — 受信箱リスト**

- リスト形式で会話一覧を表示 (会話ペインなし)
- 未読は左ボーダーをアクセントカラーで強調
- タブ: すべて / 未読 / グループ / リクエスト

**バリアントC — フローティングウィンドウ**

- 画面右下にチャットウィンドウをフローティング表示
- 複数会話を同時に開ける
- 最小化/最大化対応

**グループDM:**
- スレッド名 + 参加人数を表示
- リクエスト (未承認DM) を別タブで管理

---

### 8.7 プロフィール

**バリアントA — バナー + タブ**

- カバー画像 (4:1) + アバター (重ね合わせ)
- 表示名・ハンドル・自己紹介・統計 (投稿数・フォロワー・フォロー)
- フォロー / フォロー中 / 編集 / その他 (`···`) ボタン
- タブ: 投稿 / 返信 / メディア / いいね

**バリアントB — カード + グリッド (Misskeyライク)**

- 左カラム: プロフィールカード (中央寄せ) + 統計カード
- 右カラム: 2カラムグリッドの投稿カード
- ロールタグ (designer / tokyo など)

**バリアントC — エディトリアル**

- 大きな氏名タイポグラフィ (64px)
- 3カラム: 自己紹介 / 最近の話題タグ / 統計
- アクセントカラーのアンダーライン装飾

**ActivityPub対応表示:**
- ローカルユーザー: `@hana`
- リモートユーザー: `@hana@remote.example` (フルハンドルを表示)

---

### 8.8 設定

**バリアントA — 2ペインサイドバー (デフォルト候補)**

- 左: 設定カテゴリナビゲーション (220px)
- 右: 選択中カテゴリのフォーム
- プロフィール設定: アバター変更・表示名・ハンドル・自己紹介

**バリアントB — カードグリッド**

- 設定カテゴリをカードグリッドで一覧表示
- Dangerゾーン (アカウント停止・削除) を破線カードで分離

**バリアントC — シングルページフォーム**

- 左: アンカーリンクジャンプナビ (160px)
- 右: 全設定をスクロールで表示

**設定カテゴリ:**

| カテゴリ | 項目 |
|---|---|
| アカウント | プロフィール、メール、パスワード、連携アカウント |
| プライバシー | 公開範囲、ブロック、ミュート |
| 通知 | プッシュ、メール、メンション |
| 表示 | テーマ (ライト/ダーク/自動)、言語、密度 (コンパクト/標準/ゆったり)、タイムゾーン |
| データ | エクスポート、削除 |
| 連携 | 他サービス連携、APIキー |
| 2段階認証 | SMS / TOTP |

---

## 9. WebSocketによるリアルタイム更新

バックエンドのWebSocketエンドポイント (`/ws`) に接続し、新着ノートをタイムラインSignalに差し込みます。

```rust
// store/stream.rs
pub fn connect_stream(token: String, set_notes: WriteSignal<Vec<Note>>) {
    spawn_local(async move {
        let url = format!("wss://{}/ws?token={}", HOST, token);
        // web_sys::WebSocket を使用 (gloo-net or 生JS interop)
        let ws = web_sys::WebSocket::new(&url).unwrap();

        let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if let Some(text) = e.data().as_string() {
                if let Ok(event) = serde_json::from_str::<StreamEvent>(&text) {
                    match event {
                        StreamEvent::Note(note) => {
                            set_notes.update(|notes| notes.insert(0, note));
                        }
                        StreamEvent::Notification(_) => { /* 通知バッジ更新 */ }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();
    });
}
```

---

## 10. MFMレンダリング

`shared` クレートの `pest` ベースのMFMパーサを呼び出し、Leptosのビューに変換します。

```rust
// components/mfm.rs
#[component]
pub fn MfmText(text: String) -> impl IntoView {
    let nodes = shared::mfm::parse(&text);
    nodes.into_iter().map(|node| match node {
        MfmNode::Text(t)     => view! { <span>{t}</span> }.into_any(),
        MfmNode::Hashtag(t)  => view! { <a class="text-accent hover:underline" href=format!("/search?tag={t}")>{"#"}{t}</a> }.into_any(),
        MfmNode::Mention(m)  => view! { <a class="text-accent hover:underline" href=format!("/{m}")>{"@"}{m}</a> }.into_any(),
        MfmNode::Url(u)      => view! { <a class="text-accent hover:underline break-all" href=u.clone() target="_blank">{u}</a> }.into_any(),
        MfmNode::Bold(inner) => view! { <strong><MfmText text=inner /></strong> }.into_any(),
        MfmNode::Emoji(e)    => view! { <span class="inline-emoji">{e}</span> }.into_any(),
        // ... 他のノードタイプ
    }).collect_view()
}
```

---

## 11. レスポンシブ対応

| ブレークポイント | レイアウト |
|---|---|
| `< 640px` (mobile) | ボトムナビバー表示、RightRail非表示、Sidebar非表示 |
| `640px〜1024px` (tablet) | コンパクトSidebar (アイコンのみ)、RightRail非表示 |
| `> 1024px` (desktop) | フル3カラムレイアウト |

TailwindCSSのブレークポイントプレフィックス (`sm:`, `md:`, `lg:`) をそのまま使用します。

---

## 12. パフォーマンス方針

- **初回ロード**: SSRで初期HTMLを返しFCPを高速化（cargo-leptosのSSR機能を使用）
- **タイムライン描画**: Leptosの粒度の細かいSignal更新により、カード単体のみ再描画
- **画像**: `loading="lazy"` + `aspect-ratio` で画像のガタツキを防止
- **無限スクロール**: Intersection Observer で最終カードを監視し、追加フェッチ
- **WASMサイズ**: `wasm-opt` で最適化、gzip後 500KB 前後を目標

---

## 13. 未解決事項 / 検討中

| 項目 | 状況 | 備考 |
|---|---|---|
| 各画面のバリアント選定 | ⬜ 未決定 | A/B/Cから選択または統合 |
| 仮想スクロール | ⬜ 未実装 | 大量ノート対策。JS interopか自前実装 |
| カスタム絵文字ピッカー | ⬜ 未設計 | サーバーごとに絵文字セットが異なる |
| i18n (多言語対応) | ⬜ 未設計 | `fluent` をWASM側でも使うか検討 |
| プッシュ通知 (Web Push) | ⬜ 未設計 | Service Worker 経由 |
| モバイルジェスチャー | ⬜ 未設計 | スワイプでタブ切り替え等 |
