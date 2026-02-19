# EXCH-AUDIT-POSTB-000 Post-TaskB 監査レポート

- Task ID: `EXCH-AUDIT-POSTB-000`
- Scope: `postb-audit-and-dedup-plan`
- Execution mode: `SINGLE-RUN`
- Locks: `LOCK:shared-docs`（コード変更は未実施）

## 0. Gate確認（Task B merged）

- Task B 相当の `MD-DESC-B-005` は完了済みで、trace/status 上の追跡PR `#156` は `merged`。
- 旧PR `#140` は `closed(no-merge)` 継続。

## 1. Repo 実態スキャン（services/marketdata）

実行コマンド（機械列挙）:

- `python - <<'PY' ...`（`services/marketdata` 配下で adapter/facade/transport/policy/descriptor/dsl/engine/gmo を含むファイル名列挙）
- `rg --files services/marketdata`
- `rg -n "adapter|facade|transport|policy|descriptor|dsl|engine|gmo" services/marketdata/app services/marketdata/tests`

### 1.1 “UCELっぽい”層の重複有無

- `descriptor/dsl/engine` 実装本体は **`app/descriptor_dsl.py` 単一**。
- “adapter/facade/policy” 命名の別モジュールは検出なし。
- `engine` 命名は `OrderbookEngine`（`silver/orderbook.py`）と descriptor 実行（`execute_descriptor`）で**用途が異なる**ため、同目的重複とは判定しない。

### 1.2 gmo_adapter 相当の重複有無

- `gmo_adapter*` 命名ファイルは検出なし。
- GMO導線は以下2つで構成（役割分離）:
  - `main.py` 内 REST poller (`_fetch_gmo_ticker`, `_fetch_gmo_ohlcv`)
  - `gmo_ws_connector.py` の WS ingest
- 現状は「同一アダプタの二重実装」ではなく「transport導線分離」。

### 1.3 descriptor / dsl / engine 重複（Task B との重複）

- Task B で追加/統合された `execute_descriptor(...)` は `descriptor_dsl.py` に単一存在。
- テストは parser/validator系 (`test_descriptor_dsl.py`) と統合E2E系 (`test_descriptor_engine_e2e.py`) で責務分離され、同一目的の二重テストとは判定しない。

### 1.4 同一目的テストの二重存在

- 明確な同一目的の完全重複テストは未検出。
- ただし重複疑い（運用上の二重導線）:
  - health/capabilities: `app/main.py` 実装と `app/routes/health.py` 実装の併存。
  - object store abstraction: `app/object_store.py` と `app/storage/object_store.py` の2系統。

## 2. Open PR / branch 整理方針

- `#140`: close-without-merge を継続（direct merge 不可）。
- `#156`: merged（Task B 完了の根拠PR）。
- 類似PRが将来残存した場合の基準:
  - 旧境界実装の再流入になるPRは close。
  - 必須差分は `UCEL-CORE-POSTB-001` 以降の新PRへ re-implement。
  - 部分再利用は「仕様/責務がUCEL基盤に一致する最小要素のみ cherry-pick 相当」で扱う（PR取り込みはしない）。

## 3. 次コミットでの修正方針（確定）

1. 参照先一本化（additive-only）を優先し、即削除はしない。
2. `descriptor_dsl.py` を descriptor 実行の唯一SSOTとして拡張。
3. GMO transport は op記述SSOTを先に固定してから統合。
4. health/object-store 二重導線は「main側を基準」に寄せる計画を `UCEL-CORE-POSTB-001` で実施。

## 4. DoD反映

- 重複/二重実装の疑い一覧化: 完了。
- 採用/廃止/移行方針を decisions に固定: 完了。
- 次タスク `UCEL-CORE-POSTB-001` へ進行可能な状態: 完了。
