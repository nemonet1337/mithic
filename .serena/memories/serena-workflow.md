# Serena の使い方 (mithic)

- コード探索: `get_symbols_overview` → `find_symbol` (`include_body`) → 必要なら `find_referencing_symbols`
- 編集: `replace_content` / `replace_symbol_body` / `insert_after_symbol` を優先。リファクタは `rename_symbol` / `safe_delete_symbol`
- 並列 tool call を最大限バッチ
- オンボーディング未実施 (2026-08-25)
- Headroom MCP あり: 大きな出力は `headroom_compress` で圧縮可
- 記憶の参照は `mem:name` （例: `mem:project-overview`）
