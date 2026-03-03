# E-PLAN-003 Verification

## 1) Changed files
- bots/simple_mm/main.py
- dashboard_api/main.py
- dashboard_api/safety_controller.py
- dashboard_api/safety_lease.py
- dashboard_api/safety_runtime.py
- libs/safety_core/__init__.py
- libs/safety_core/gate.py
- libs/safety_core/lease.py
- libs/safety_core/lease_client.py
- worker/safety_lease_renewer.py
- tests/test_safety_gate_failclose.py
- tests/test_safety_lease_lifecycle.py
- docs/status/trace-index.json
- docs/verification/E-PLAN-003.md

## 2) What/Why
- Added Execution Lease primitives and fail-close gate logic in `libs/safety_core` to force a final check immediately before external execution I/O.
- Added lease issue/renew/status API endpoints in dashboard API with idempotency, evidence validation, safety-mode deny behavior, and lease lifecycle handling.
- Added worker-side lease renewer for 5s renewal cadence and repeated-failure blocking signal.
- Inserted send-time safety gate call into `bots/simple_mm/main.py` at `submit_order_intent` (immediately before the POST to `/execution/order-intents`).
- Added tests that prove fail-close behavior blocks external send calls when lease/state are missing/expired/unreachable.

## 3) Self-check results
- Allowed-path check OK (no changes outside allowlist).
- Tests added/updated OK:
  - `tests/test_safety_gate_failclose.py`
  - `tests/test_safety_lease_lifecycle.py`
- Build/Test commands & results:
  - `python -m pytest -q tests/test_safety_gate_failclose.py tests/test_safety_lease_lifecycle.py bots/simple_mm/test_main.py tests/test_safety_core_engine.py tests/test_safety_core_store.py` => pass
  - `python -m pytest -q` => pass
- trace-index json.tool OK: `python -m json.tool docs/status/trace-index.json`.
- Secrets scan OK (no key-like literals introduced).
- docsリンク存在チェック: this verification doc references `docs/status/trace-index.json` and that file exists.

## 4) 履歴確認の証拠（必須）
### 実行コマンド
- `git log --oneline --decorate -n 50`
- `git log --graph --oneline --decorate --all -n 80`
- `git log --merges --oneline -n 30`
- `git show 5ef35d3f`
- `git merge-base HEAD work`
- `git branch -vv`
- `git reflog -n 30`
- `rg -n "send_order|place_order|submit_order|create_order|cancel_order|amend_order|transport|http|websocket|rest" -S bots libs worker dashboard_api`
- `rg -n "SAFE_MODE|safe_mode|SafetyState|safety_state|kill|interlock|lease" -S .`
- `git blame -w bots/simple_mm/main.py`

### SHA要点と結論
- 直近の前タスクコミットは `5ef35d3f`（E-PLAN-002）で、今回タスクはその直後に積み増し。
- merge-base(`HEAD`, `work`) は前提時点で同一系統（ローカル既定ブランチ `work`）で分岐矛盾なし。
- merge履歴上、Safety関連は E-PLAN-001/E-PLAN-002 が最新で、Execution Lease/send-time gate は未実装だったため本タスク実装は重複置換ではなく不足分の追加。

### “送信直前”箇所の特定結果
- `bots/simple_mm/main.py` の `submit_order_intent` が外部I/O直前の最終送信関数（`http_json("POST", .../execution/order-intents)`）であることを `rg` で確認。
- `git blame -w` から同ファイルは bot の実行制御ハブとして維持されていたため、当該関数にのみ局所差分で gate を追加し、抜け道を作らない形で最小変更とした。
- 共通送信ラッパは既存に無かったため、まず send-time gate を最終関数へ直差し（1箇所）して fail-close を成立。

## 5) 追加実装（不足分対応）
- 追加根拠: 送信経路探索で bot 側に共通I/Oラッパ不在、かつ外部送信点が `submit_order_intent` に収束していたため。
- 実装効果: lease missing/expired/safety unreachable/unknown safety state 時に `SafetyGateError` → `BotError(SAFETY_BLOCKED:...)` で送信前に停止。
