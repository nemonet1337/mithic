# TODO — コードベース監査結果 (2026-07-26)

**進捗**: 監査項目のほぼすべて完了。ネイティブ API 統合 (api_plan Phase 1–5) 完了。ディレクトリ再編 Phase 1 完了。

## 完了済み (主要)

### セキュリティ §1
- [x] 1.1–1.12 すべて

### バグ §2
- [x] 2.1–2.14 すべて (2FA 本実装は未・UI は準備中)

### パフォーマンス §3
- [x] 3.1 search N+1
- [x] 3.2 serve_upload ストリーム配信 (`Body::from_stream`)
- [x] 3.3 I18n
- [x] 3.4 通知/カウンタ spawn + メンション一括
- [x] 3.5 federation キャッシュ janitor
- [x] 3.6 broadcast コメント
- [ ] 3.7 frontend comrak メモ化 (許容)

### 重複 §4
- [x] 主要完了
- [ ] 4.7 actor デシリアライズ共通化 (任意)
- [ ] 4.8 cache block/mute (任意)

### 警告 §5 / UI 配線
- [x] バックエンド clippy クリーン
- [x] Toast / FollowButton / NoteMenu / ReactionPicker を画面に接続
- [x] 投稿: リアクション・リノート・削除・リンク共有
- [x] プロフィール: FollowButton + トースト
- [x] ログイン成功トースト

### その他 §6
- [x] ノート長制限 / AP HTML content / should_persist_note
- [x] AppState 引数名 / email 検証 / totalItems
- [x] CLAUDE.md / README / .env.example
- [x] ネイティブ API 統合 (`routes/v1/` へ集約、misskey 互換層削除)
- [ ] Dockerfile 後始末の細部 (任意)

## API リプレース (api_plan.md)

### Phase 1–3: v1 統合 + 互換層削除
- [x] `routes/frontend_api` + `routes/misskey` → `routes/v1/` に統合
- [x] REST 化: drive / push / streaming / follow-requests / block / mute / admin / relays
- [x] `GET /api/v1/instance` (メタ + カスタム絵文字)
- [x] local/global タイムラインを認証不要に
- [x] フロント: streaming `/api/v1/streaming`、drive REST、`until_id` / `emoji` 修正
- [x] Misskey クライアント API / MiAuth / OAuth ルート削除

### Phase 4: ActivityPub Misskey 拡張
- [x] `@context` に Misskey 拡張宣言 (`_misskey_reaction`, `quoteUrl` 等)
- [x] Create 配送に quoteUrl / `_misskey_quote` / FEP-e232 `quote` 併記
- [x] `build_like_activity` / `build_undo_like_activity` / `deliver_reaction`
- [x] inbox: Like → リアクション保存 (content / `_misskey_reaction` / デフォルト ⭐)
- [x] inbox: Create(Note) → quoteUrl 解決 + Emoji tag を remote_emoji キャッシュ
- [x] inbox: Create(Question) + name のみ Note による投票
- [x] inbox: Announce → リノート
- [x] inbox: Undo(Like) / Undo(Follow)
- [x] ローカル リアクション API から Like / Undo(Like) を連合配送

### Phase 5: 高性能化
- [x] タイムライン GET の Dragonfly キャッシュ (local/global JSON 15s、home Sorted Set、投稿時 invalidate)
- [x] 公開リソース ETag / Cache-Control (instance / note / local·global TL / trending)
- [x] streaming ワイヤフォーマット統一 (`shared::StreamEvent` = `{type, body}`)

## ディレクトリ再編 (shrink_plan.md)

- [x] Phase 1: `backend/` + `frontend/` への配置移動 (クレート統合なし)
- [ ] Phase 2: backend クレート統合 (任意・保留)

## 起動必須

```bash
JWT_SECRET=<十分な長さのランダム文字列>
```

## 任意の次ステップ

1. Clipboard API feature でリンクを本当にコピー
2. ミュート/ブロック UI を v1 API クライアントへ接続
3. 2FA 本実装
4. comrak メモ化 / タイムライン仮想化
5. Delete アクティビティの永続化反映 / リモートノート HTML→プレーン正規化
