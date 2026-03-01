# Verification — UCEL-DECIMAL-REMAIN-4-001

## 1) Changed files（git diff --name-only）
- docs/README.md
- docs/runbooks/README.md
- docs/specs/ucel/decimal_policy_spec.md
- docs/specs/ucel/order_gate_spec.md
- docs/runbooks/ucel_decimal_policy_incidents.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-remain-4-001.md

## 2) What/Why
- Appended DoD and SSOT entry links to `decimal_policy_spec.md` so completion criteria and operational entry points are explicit.
- Added dedicated `order_gate_spec.md` as the fixed SSOT for final pre-order tick/step enforcement responsibilities.
- Added a minimal incident runbook for Decimal Policy / OrderGate failures to standardize triage and recurrence prevention.
- Added minimal index links in `docs/README.md` and `docs/runbooks/README.md` so Domain入口から必ず辿れる導線を確保.
- Updated trace index only for task `UCEL-DECIMAL-REMAIN-4-001` and recorded this verification file.

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の "docs/" 参照だけ）
  - MISSING: none
