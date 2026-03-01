# UCEL-DECIMAL-APPLY-003 Verification

## 1) Changed files（git diff --name-only）
- docs/runbooks/README.md
- docs/runbooks/ucel_decimal_apply_step3_ordergate_and_final_verification.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-apply-003.md

## 2) What/Why
- OrderGate を発注経路へ最終的に組み込む Step3 実行手順SSOT を新規作成した。
- Kraken への適用とテストパッチ適用を起点に、他CEXへの水平展開手順を固定した。
- 最終監査（f64残存 / 無検証parse / OrderGate導入 / value-class serde）を一本道で実行できるよう明文化した。
- runbooks index へ最小リンクを追加して既存導線から到達可能にした。
- trace-index では TASK-ID `UCEL-DECIMAL-APPLY-003` のみを追加し、artifact/evidence を記録した。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の "docs/" 参照だけ）
  - MISSING:
    - docs/patches/ucel/UCEL-DECIMAL-REMAIN-3-001.order-gate-kraken.patch
    - docs/patches/ucel/UCEL-DECIMAL-REMAIN-3-001.tests.patch
