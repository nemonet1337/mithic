# 機能ギャップ分析 (Feature Gap Analysis)

**検証日**: 2026-06-11
**参照**: `old-src/`（旧 Misskey/Dolphin 実装）との機能比較、全クレート実装インベントリ調査 (2026-06-04)、パフォーマンス調査
**関連**: `docs/performance-optimization-plan.md`, `TODO.md`（Phase 単位ロードマップ統合済）

> 本ファイルは CLAUDE.md §7 が常時更新を求めるギャップ台帳である。機能を実装したとき・新たな不足を発見したときに更新する。
> 2026-06-04 の実コードインベントリ調査で、いくつかの性能ギャップが既に解消済みであることが判明したため同期した。
> 2026-06-11: Phase 0/2/5/F1/F2 の主要項目を実装し同期。併せて DB 層の重大バグ
> （SurrealDB 3 レコードID正規化・`type::thing` 廃止対応・`$token` 予約変数・RELATE 構文・
> SCHEMAFULL の FLEXIBLE オブジェクト）を修正し、API を E2E スモークテストで検証した。

---

## 1. パフォーマンス関連ギャップ（2026-06-04 実測で同期）

性能の主要ボトルネックは「遅い実装」ではなく**未実装のホットパス**である。詳細と設計は
`docs/performance-optimization-plan.md` を参照。状態列: ✅解消 / 🔶部分 / ⬜未着手。

| # | ギャップ | 状態 | 根拠 | 区分 | 設計参照 |
|---|---|---|---|---|---|
| P-G1 | `init_schema()` を起動時に呼ぶ | ✅ | server/worker とも起動時に呼出 | 重大 | 計画 §3-A |
| P-G2 | ホームタイムライン（フォローグラフ） | ✅ | `db/src/queries/timeline.rs:get_home_timeline`、`notes::home_timeline` 配線済 | 重大 | 計画 §3-D |
| P-G3 | TL クエリの行ごと deref（DB 内 N+1） | ✅ | `actor_id.* AS author` で著者同梱 (`db/src/queries/timeline.rs`) | 高 | 計画 §3-A/C |
| P-G4 | API 層で著者を 1 件ずつ取得（アプリ層 N+1） | ✅ | `NoteWithAuthor` を直接 DTO 化 | 高 | 計画 §3-C |
| P-G5 | SurrealDB/Dragonfly コネクションプール無し | ✅ | ラウンドロビン `SurrealClient` プール + `ConnectionManager` (`db/src/lib.rs`) | 高 | 計画 §3-B |
| P-G6 | Dragonfly キャッシュヘルパ未使用（TL/note/応答キャッシュ無し） | 🔶 | ヘルパ実装済 (`db/src/cache.rs`)。ホットパスへの組込みは未 | 高 | 計画 §3-D |
| P-G7 | ワーカーが配送ループを起動 | ✅ | `worker/src/main.rs` が `run_delivery_worker()` を spawn | 重大 | 計画 §3-E |
| P-G8 | 配送が単一ループ・並列度 1。prefetch/バッチ無し | ✅ | 並列4ワーカー + 専用 BRPOP 接続 (`run_delivery_workers`) | 重大 | 計画 §3-E |
| P-G9 | retry キューを誰も drain しない。DLQ・サーキットブレーカ無し | 🔶 | ZSET スケジューラ + 指数バックオフ + DLQ 実装。サーキットブレーカは未 | 高 | 計画 §3-E |
| P-G10 | ホスト単位バッチ配送無し（inbox ごとに 1 ジョブ） | ⬜ | `federation/src/service.rs` | 高 | 計画 §3-F |
| P-G11 | HTTP 署名 — 検証は実装、**生成**が `"placeholder"` | ✅ | RSA-SHA256 実署名 + 鍵キャッシュ (`federation/src/service.rs`)。signup 時に鍵ペア生成 | 重大 | 計画 §3-F |
| P-G12 | `FederationService` がプール未設定の `Client::new()` を二重生成 | ✅ | AppState のプール済 `reqwest::Client` を共有 | 中 | 計画 §3-F |
| P-G13 | リレー購読/取り込み/dedup/`should_persist`・`relay`/`activity` テーブル無し | ⬜ | `core/src/models/relay.rs`, `db/src/surreal.rs` | 重大 | 計画 §3-A/F |
| P-G14 | `fetch_remote_actor` — HTTP 取得は実装、JSON-LD パースが未（`None` 返却） | ✅ | `parse_remote_actor` で Actor 変換 + inbox 受信時に永続化 | 高 | 計画 §3-F |
| P-G15 | アロケータ/リリースビルド最適化・simd-json/MessagePack 未適用 | 🔶 | mimalloc + fat LTO/codegen-units=1/strip 適用済。simd-json/MessagePack は未 | 中 | 計画 §3-G |
| P-G16 | メトリクス（Prometheus）/`tokio-console` 未導入 | ⬜ | 全体 | 中 | 計画 §3-H |

各項目の実装計画は `TODO.md` の Phase 0 / Phase 2 / Phase F2 / Phase F3 / Phase 9 を参照。

---

## 2. 機能ギャップ（old-src 比較・主要項目）

詳細は `TODO.md` の各 Phase を参照。性能観点と重複しない主な未実装:

- [ ] MFM の全機能（カスタム絵文字、位置指定アニメーション等） → Phase 8 / F-11
- [x] WebFinger / Actor JSON-LD / NodeInfo / inbox（Follow・Undo 処理）実装済。outbox/followers コレクションは未 → Phase F1
- [x] 通知生成（reply/reaction/renote/follow）+ WebSocket リアルタイム配信実装済。mention 通知は未 → Phase 3
- [ ] 全文検索（SurrealDB 任せ → Meilisearch/Tantivy 専門化を検討） → Phase 6 / Phase 9
- [ ] モデレーション機能・管理者機能 → Phase 7
- [ ] プラグインシステム → 未計画（将来）
- [ ] API の完全互換性（Mastodon/Misskey クライアント） → Phase 6 全般

---

## 3. 更新ルール（CLAUDE.md §7）

- 機能を実装・完了したら、本ファイルの該当項目を削除またはチェック済みに変更し、`TODO.md` の対応ボックスを更新する。
- 新たな不足を発見したら本ファイルと `TODO.md` に追記する。
- 更新時は冒頭の「検証日」を更新日に書き換える。
