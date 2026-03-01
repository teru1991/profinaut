# UCEL-DECIMAL-APPLY-002 Verification

## 1) Changed files（git diff --name-only）
- docs/runbooks/README.md
- docs/runbooks/ucel_decimal_apply_step2_cex_connectors.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-apply-002.md

## 2) What/Why
- CEXコネクタ群向けの Step2 実行手順SSOT を新規作成した。
- f64排除と guarded ingestion の適用順を固定し、実コード反映時の迷いをなくした。
- CEXごとの典型的な型不一致・変換エラーへの対処を runbook に明記した。
- runbooks index へ最小リンクを追加し、既存導線からアクセス可能にした。
- trace-index では TASK-ID `UCEL-DECIMAL-APPLY-002` のみを追加し、artifact/evidence を記録した。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の "docs/" 参照だけ）
  - MISSING: none
