# Antigravity CLI 協業ブリッジ — 環境構築

Claude Code から Antigravity CLI（`agy` / Gemini 3.5 Flash (High)）へ実装を委譲する協業環境のセットアップ手順。
**実機検証日時（2026-06-13 / agy 1.0.8 / Windows 11）。**

> **Claude が読んで実行する想定**。各手順はコマンドベース。Windows / PowerShell 前提。

---

## 前提条件

| 項目 | 確認コマンド | 備考 |
|---|---|---|
| Antigravity CLI | `agy --version` | 未導入なら手順0でインストール。AI Ultra/Pro で **OAuth ログイン済み**であること |
| Python 3.10+ | `python --version` | 未導入環境では導入方法を案内すること |
| git | `git --version` | ブリッジの clone に使用 |

### ⚠️ 最重要：従量課金を避ける
`GEMINI_API_KEY` / `ANTIGRAVITY_API_KEY` を**環境変数に設定しないこと**（設定すると OAuth サブスク枠をバイパスして従量課金になる）。すべて `(未設定)` を確認:
```powershell
foreach ($n in 'GEMINI_API_KEY','ANTIGRAVITY_API_KEY','GOOGLE_API_KEY') {
  foreach ($s in 'Process','User','Machine') {
    $v = [Environment]::GetEnvironmentVariable($n,$s)
    "{0,-22}{1,-8}: {2}" -f $n,$s,$(if($v){'★設定あり'}else{'(未設定)'})
  }
}
```

---

## 0. Antigravity CLI（`agy`）のインストール（未導入の場合）

導入済みなら読み飛ばす。未導入なら **公式（Google 所有ドメイン）** から:
```powershell
irm https://antigravity.google/cli/install.ps1 | iex
```
- インストール先: `%LOCALAPPDATA%\agy\bin\agy.exe`（= `C:\Users\<ユーザー名>\AppData\Local\agy\bin`）
- PATH が変わるのでターミナル（および Claude Code）を再起動
- 初回に `agy` を起動して認証 ── **Google OAuth** を選び **AI Ultra/Pro アカウント**でログイン
- 確認: `agy --version`

> ⚠️ 必ず **公式ドキュメント**（[antigravity.google/docs/cli-install](https://antigravity.google/docs/cli-install)）で最新を確認。macOS / Linux 版も公式に記載。
> 📝 日本語の詳しい解説（スクショ・PATH 設定・課金体系）: [Antigravity CLI セットアップ解説（note）](https://note.com/hantani/n/nee2e9179c67b)

---

## 手順

### 1. agy のモデルを High に固定
`%USERPROFILE%\.gemini\antigravity-cli\settings.json` を読み、`"model"` キーを追加する（既存キーは保持）:
```json
{
  "model": "Gemini 3.5 Flash (High)"
}
```
→ 素の `agy -p` でも High が適用される。`--model` フラグは `-p` と順序依存で不安定なので使わない。

### 2. ブリッジを clone
```powershell
$dest = "$env:USERPROFILE\tools\agy-mcp-bridge"
git clone https://github.com/SinanTufekci/Claude-Code-Antigravity-CLI-MCP-Server.git $dest
```

### 3. server.py を確認し、AGY_BIN 対応に改修
clone 後、`server.py` を必ず目視確認し精査すること（標準ライブラリ + fastmcp のみ / subprocess は引数リスト渡し / `~/.gemini` 配下の読み取り中心 / 不審な外部送信なし）。

**続けて、PATH 問題を予防する改修を入れる**（Windows では MCP サーバーが起動時の古い PATH を継承し agy を見失うため）:
1. import 群の近くにモジュール変数を追加: `_AGY_BIN = os.environ.get("AGY_BIN") or "agy"`
2. agy 呼び出し2箇所を置換: `["agy", …]` → `[_AGY_BIN, …]`（`_get_agy_version` と `_run_agy` 内）

> ⚠️ 注意: `agy -p` は承認ゲート無しでファイル書込み・コマンド実行・ネット送信する自律エージェント。運用は SKILL.md（テキスト提案／コミット前に `git diff`／未信頼テキストを流さない）。

### 4. venv 作成 + fastmcp
```powershell
python -m venv "$dest\.venv"
& "$dest\.venv\Scripts\python.exe" -m pip install fastmcp
```

### 5. スモークテスト（AGY_BIN 設定下で。agy を1〜2回呼ぶ＝クレジット消費）
末尾【付録】の `agy_smoke_min.py` を `%TEMP%` に作成し、**AGY_BIN を設定して**実行:
```powershell
$env:AGY_BIN = "$env:LOCALAPPDATA\agy\bin\agy.exe"
& "$dest\.venv\Scripts\python.exe" "$env:TEMP\agy_smoke_min.py"
```
`agy_ask`→`'PONG'`、`agy_continue`→`'PING'` が数秒で返れば成功（= server.py の AGY_BIN 改修も効いている確証）。

### 6. Claude Code に登録（AGY_BIN 付き・user scope）
```powershell
# 設定をバックアップ
Copy-Item "$env:USERPROFILE\.claude.json" "$env:USERPROFILE\.claude.json.bak" -Force
# AGY_BIN 付きで登録（PATH 継承問題を予防）
claude mcp add agy -s user -e AGY_BIN="$env:LOCALAPPDATA\agy\bin\agy.exe" -- "$dest\.venv\Scripts\python.exe" "$dest\server.py"
# 確認（"agy: ... √ Connected"）
claude mcp list
```
※ このプロジェクト限定にするなら `-s user` を `-s project` に。

### 7. Claude Code を再起動
再起動後、`mcp__agy__agy_ask` / `agy_continue` / `agy_image` / `agy_status` が使える。まず `mcp__agy__agy_status` で `agy CLI [ok]` を確認（疎通診断・quota 消費なし）。

---

## ハマりどころ（背景・検証で判明したこと）
- `agy -p` は **stdout に応答を出さない**（公式 issue #76）→ ブリッジが transcript（`brain/<id>/.system_generated/logs/transcript.jsonl`）の `PLANNER_RESPONSE` を読む。
- 非対話実行は **stdin 閉じ必須**（ブリッジは `subprocess.DEVNULL` 実装済み。手動なら `$null | agy -p "..."`）。
- High は **settings.json** で固定（`--model` フラグは順序依存で不安定）。
- **PATH 継承問題**（手順3・6 の AGY_BIN で予防済み）: 再起動後、MCP サーバーが古い PATH を継承し agy を見失う。User PATH に `agy\bin` があっても起きる。

## トラブルシュート
- **`agy_status` で `agy CLI [!!] not found on PATH`** → AGY_BIN が効いていない。手順3の改修（`_AGY_BIN`）と手順6の `-e AGY_BIN` を確認し、`agy.exe` の実パス（`%LOCALAPPDATA%\agy\bin\agy.exe`）が正しいか確認 → 再登録 → 再起動。
- `mcp__agy__*` が出ない → Claude Code を再起動したか / `claude mcp list` で Connected か。
- 応答が空 → `mcp__agy__agy_status` で診断。settings.json・brain dir を確認。
- **応答が別実行のものに見える／混在する** → agy を IDE 等で並行起動していないか確認。ブリッジは「最新の brain フォルダ＝自分の実行」前提のため、同時起動で transcript が混在する。検証中は他の agy を閉じる。
- agy 更新で壊れた → server.py は agy 1.0.7 の state-file 前提。将来 SQLite 形式へ完全移行すると transcript 読みが壊れる（server.py docstring 参照）。

---

## 【付録】最小スモークスクリプト（`%TEMP%\agy_smoke_min.py`）

```python
"""最小スモーク: ブリッジ経由で agy_ask / agy_continue を実行（画像はスキップ）。"""
import os, sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")
BRIDGE = os.path.expanduser(r"~\tools\agy-mcp-bridge")
sys.path.insert(0, BRIDGE)
from server import agy_ask, agy_continue

print("smoke1 agy_ask     :", agy_ask(prompt="Reply with exactly one word: PONG", workspace=BRIDGE))
print("smoke2 agy_continue:", agy_continue(prompt="Now reply with exactly one word: PING", workspace=BRIDGE))
print("ALL PASS")
```
