# UCEL-DECIMAL-APPLY-001 Verification

## 1) Changed files（git diff --name-only）
- docs/README.md
- docs/runbooks/README.md
- docs/runbooks/ucel_decimal_apply_step1_core_and_symbol.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-apply-001.md

## 2) What/Why
- Decimal/OrderGate 系 patch を実コードへ反映するための SSOT runbook を新規作成した。
- 適用順序・コンパイル時の典型エラー・修正方針を一本道で実行できる形に固定した。
- docs/runbooks/README.md に最小の索引リンクを追加し、既存運用導線から到達可能にした。
- docs/README.md に入口リンクを追加し、docs のトップ導線から Step1 runbook へ遷移可能にした。
- trace-index には TASK-ID `UCEL-DECIMAL-APPLY-001` の entry のみ追加し、artifact と verification evidence を記録した。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の "docs/" 参照だけ）
  - MISSING: none
