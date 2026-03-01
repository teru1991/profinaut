# Verification — UCEL-DECIMAL-REMAIN-5-001

## 1) Changed files（git diff --name-only）
- docs/README.md
- docs/runbooks/README.md
- docs/runbooks/ucel_decimal_policy_verification.md
- docs/specs/ucel/decimal_policy_spec.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-remain-5-001.md

## 2) What/Why
- Added a dedicated SSOT verification runbook for Decimal Policy / OrderGate so completion can be judged uniformly in CI or manual review.
- Fixed audit viewpoints in one place: f64残存, 無検証parse/serde, and OrderGate未適用 checks with concrete command examples.
- Added minimal entry links from the decimal policy spec and docs indexes so operators can always reach the verification runbook from canonical entry points.
- Updated trace index only for `UCEL-DECIMAL-REMAIN-5-001` to register artifacts and evidence for this task.

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の "docs/" 参照だけ）
  - MISSING: none
