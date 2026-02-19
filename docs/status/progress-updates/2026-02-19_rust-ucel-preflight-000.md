# RUST-UCEL-PREFLIGHT-000 Progress Update (2026-02-19)

## Summary
- 実行モード: SINGLE-RUN（調査→Docs OS反映のみ）
- 変更対象: `docs/**` のみ
- 結論: Rust UCEL 実装配置は **Option A（`ucel/` 独立 workspace 新設）** を採用。

## 1) Rust資産 監査結果（証拠）

### 実行コマンド
```bash
ls -la
find . -maxdepth 4 -name "Cargo.toml" -print
find . -maxdepth 4 -name "Cargo.lock" -print
find . -maxdepth 4 \( -type d -name "crates" -o -type d -name "rust" -o -type d -name "ucel" -o -type d -name "libs" \) -print
find .github/workflows -maxdepth 2 -type f -print
rg -n "\\bcargo\\b|\\brust\\b|marketdata-rs|Cargo.toml" .github/workflows/*.yml
find . -maxdepth 3 \( -name "Makefile" -o -name "justfile" -o -name "Taskfile.yml" -o -name "taskfile.yml" \) -print
find scripts -maxdepth 3 -type f 2>/dev/null | head -n 50
```

### 確定事項
- ルート `Cargo.toml`: **なし**。
- `Cargo.toml`: 2件。
  - `services/marketdata-rs/Cargo.toml`
  - `services/marketdata-rs/crypto-collector/Cargo.toml`
- `Cargo.lock`: 1件（`services/marketdata-rs/Cargo.lock`）。
- Rust候補ディレクトリ探索:
  - `./libs`（名称一致のみ。Rust確証なし）
  - `./services/marketdata/app/ucel`（Python側 `ucel` 名称）
- CI (`.github/workflows/*.yml`) への Rust/Cargo 記述: **検出なし**（`rg` exit code=1）。
- Make/Just/Taskfile: **検出なし**。
- `scripts/**` は存在するが、先頭50件の範囲で Rust/Cargo 導線は確認されず。

## 2) docs/exchanges 棚卸し（全件）

### 実行コマンド
```bash
ls -la docs/exchanges
find docs/exchanges -maxdepth 2 -type f -name "catalog.json" -print | sort
find docs/exchanges -maxdepth 2 -type f \( -name "sources.md" -o -name "rest-api.md" -o -name "websocket.md" -o -name "data.md" \) -print | sort
python - <<'PY'
import json, glob, os, sys
paths = sorted(glob.glob("docs/exchanges/*/catalog.json"))
print("catalog.json count:", len(paths))
bad = 0
for p in paths:
    try:
        d = json.load(open(p))
    except Exception as e:
        print("BAD_JSON", p, e); bad += 1; continue
    ids=[]
    for k in ("rest_endpoints","ws_channels","data_feeds"):
        for x in d.get(k,[]) or []:
            if "id" in x: ids.append(x["id"])
    if len(ids) != len(set(ids)):
        print("DUP_IDS", p, len(ids), "unique", len(set(ids))); bad += 1
print("DONE. bad=", bad)
sys.exit(1 if bad else 0)
PY
python - <<'PY'
from pathlib import Path
root=Path('docs/exchanges')
req=['catalog.json','sources.md','rest-api.md','websocket.md','data.md']
for d in sorted([p for p in root.iterdir() if p.is_dir()]):
    miss=[f for f in req if not (d/f).exists()]
    print(d.name, 'OK' if not miss else 'MISSING:'+','.join(miss))
PY
```

### 取引所ディレクトリ一覧（20件）
`binance`, `binance-coinm`, `binance-options`, `binance-usdm`, `bitbank`, `bitflyer`, `bitget`, `bithumb`, `bitmex`, `bittrade`, `bybit`, `coinbase`, `coincheck`, `deribit`, `gmocoin`, `htx`, `kraken`, `okx`, `sbivc`, `upbit`

### SSOTファイル充足状況
- `catalog.json`: 20/20 取引所で存在。
- `sources.md`: 20/20。
- `data.md`: 20/20。
- `rest-api.md`: 19/20（`deribit` のみ欠落）。
- `websocket.md`: 19/20（`deribit` のみ欠落）。

### `catalog.json` 最小検証
- `catalog.json count: 20`
- JSON parse: **全件成功**
- `id` 重複（`rest_endpoints/ws_channels/data_feeds` 横断）: **0件**
- 終了コード: 0（`DONE. bad= 0`）

## 3) Rust UCEL 配置方針（このタスクで確定）

### Decision
- 採用: **Option A — `ucel/` を repo ルートに新設し、Rust UCEL workspace を独立配置する。**

### Rationale
1. 現在の Rust 資産は `services/marketdata-rs/**` に閉じており、CI でも Rust導線が未定義。
2. 既存 Python `services/marketdata/**` と責務分離したほうが事故境界・review境界が明確。
3. 今回タスクの制約（`services/**` 非変更）とも整合し、後続タスクで additive に導入しやすい。

### 後続タスク（RUST-UCEL-IMPL-001）での実装ルート（迷いゼロ）
1. `ucel/` に root `Cargo.toml`（workspace）を作成。
2. 初期crateを `crates/ucel-core`, `crates/ucel-cex-common` として追加。
3. CEX adapter は `crates/ucel-cex-<exchange>` 命名で追加（`docs/exchanges/<exchange>/catalog.json` を唯一入力源にする）。
4. 取引所追加時フロー:
   - 先に `docs/exchanges/<exchange>/**`（catalog+sources+rest/websocket/data）を更新
   - 次に `ucel` adapter crate 追加
   - 最後に CI に `cargo fmt/clippy/test` を段階導入
5. DEX拡張前提:
   - `crates/ucel-chain-<chain_or_protocol>` を予約命名
   - CEX/DEX 共通 trait は `ucel-core` に集約し、I/O差分は各 adapter crate 側に閉じる

## 4) 次タスク発行条件
- 発行タスク: **RUST-UCEL-IMPL-001**
- 必須前提:
  1. `LOCK:shared-docs` が解放可能であること
  2. 実装タスクで必要な lock（例: `LOCK:services-marketdata`）を宣言すること
  3. まず `ucel/` 独立workspaceを scaffold し、既存 `services/**` へは非破壊連携で開始すること
