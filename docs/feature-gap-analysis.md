# 機能ギャップ分析 (Feature Gap Analysis)

**検証日**: 2026-06-03
**参照**: `old-src/`（旧 Misskey/Dolphin 実装）との機能比較、クレート実装調査、パフォーマンス調査
**関連**: `docs/performance-optimization-plan.md`, `TODO.md`

> 本ファイルは CLAUDE.md §7 が常時更新を求めるギャップ台帳である。機能を実装したとき・新たな不足を発見したときに更新する。
> 今回はパフォーマンス調査で判明した「性能ギャップ」を中心に新規作成した。

---

## 1. パフォーマンス関連ギャップ（2026-05-29 調査で判明）

性能の主要ボトルネックは「遅い実装」ではなく**未実装のホットパス**であることが判明した。詳細と設計は
`docs/performance-optimization-plan.md` を参照。

| # | ギャップ | 根拠 | 区分 | 設計参照 |
|---|---|---|---|---|
| P-G1 | `init_schema()` が起動時に呼ばれず、スキーマ/21 インデックスが未適用の可能性 | `db/src/surreal.rs:53`, `server/src/main.rs` | 重大 | 計画 §3-A |
| P-G2 | ホームタイムライン（フォローグラフ）不在。`HomeTimelineChannel` は NOOP | `stream/src/channels.rs` | 重大 | 計画 §3-D |
| P-G3 | TL クエリが行ごとレコードリンク deref（DB 内 N+1） | `db/src/queries/timeline.rs:13-46` | 高 | 計画 §3-A/C |
| P-G4 | API 層で著者を 1 件ずつ取得（アプリ層 N+1） | `api/src/routes/timeline.rs` | 高 | 計画 §3-C |
| P-G5 | SurrealDB/Dragonfly ともにコネクションプール無し | `db/src/lib.rs:13-34`, `api/src/state.rs` | 高 | 計画 §3-B |
| P-G6 | Dragonfly キャッシュヘルパ未使用（TL/note/応答キャッシュ無し） | `db/src/dragonfly.rs` | 高 | 計画 §3-D |
| P-G7 | ワーカーが配送ループを起動しない（キューが溜まるだけ） | `worker/src/main.rs` | 重大 | 計画 §3-E |
| P-G8 | 配送が単一ループ・並列度 1。prefetch/バッチ無し | `federation/src/service.rs` | 重大 | 計画 §3-E |
| P-G9 | retry キューを誰も drain しない。DLQ・サーキットブレーカ無し | `federation/src/service.rs` | 高 | 計画 §3-E |
| P-G10 | ホスト単位バッチ配送無し（inbox ごとに 1 ジョブ） | `federation/src/service.rs:29-48,154-201` | 高 | 計画 §3-F |
| P-G11 | HTTP 署名が `"placeholder"`、検証は常に `Ok(true)` | `federation/src/service.rs:50-115`, `api/src/middleware/http_signature.rs:188-203` | 重大 | 計画 §3-F |
| P-G12 | `FederationService` がプール未設定の `Client::new()` を二重生成 | `api/src/state.rs:30-34`, `federation/src/service.rs:24` | 中 | 計画 §3-F |
| P-G13 | リレー購読/取り込み/dedup/`should_persist`・`relay`/`activity` テーブル無し | `core/src/models/relay.rs`, `db/src/surreal.rs` | 重大 | 計画 §3-A/F |
| P-G14 | `fetch_remote_actor` がスタブ（常に `None`） | `federation/src/service.rs:317-332` | 高 | 計画 §3-F |
| P-G15 | アロケータ/リリースビルド最適化・simd-json/MessagePack 未適用 | ルート `Cargo.toml` | 中 | 計画 §3-G |
| P-G16 | メトリクス（Prometheus）/`tokio-console` 未導入。性能の定量比較基盤が無い | 全体 | 中 | 計画 §3-H |

---

## 2. 機能ギャップ（old-src 比較・主要項目）

詳細は `TODO.md` の B-1〜B-6, F-1〜 を参照。性能観点と重複しない主な未実装:

- [ ] MFM の全機能（カスタム絵文字、位置指定アニメーション等）
- [ ] WebFinger / Actor JSON-LD / inbox・outbox エンドポイント（連合受信の前提）
- [ ] 通知サービス（mention/reply/reaction 生成・配送）
- [ ] 全文検索（SurrealDB 任せ → Meilisearch/Tantivy 専門化を検討）
- [ ] モデレーション機能・管理者機能
- [ ] プラグインシステム
- [ ] API の完全互換性（Mastodon/Misskey クライアント）

---

## 3. 更新ルール（CLAUDE.md §7）

- 機能を実装・完了したら、本ファイルの該当項目を削除またはチェック済みに変更し、`TODO.md` の対応ボックスを更新する。
- 新たな不足を発見したら本ファイルと `TODO.md` に追記する。
- 更新時は冒頭の「検証日」を更新日に書き換える。
