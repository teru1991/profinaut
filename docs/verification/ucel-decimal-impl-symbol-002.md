# Verification — UCEL-DECIMAL-IMPL-SYMBOL-002

## 1) Changed files (`git diff --name-only`)
- docs/patches/ucel/UCEL-DECIMAL-IMPL-SYMBOL-002.md
- docs/patches/ucel/UCEL-DECIMAL-IMPL-SYMBOL-002.patch
- docs/status/trace-index.json
- docs/verification/ucel-decimal-impl-symbol-002.md

## 2) What/Why
- ucel-symbol-core を ucel-core Decimal SSOT（policy/tick-step）へ委譲するための完全パッチを docs に固定しました。
- 既存の独自丸めを policy参照へ置換し、tick/step validate/quantize の追加APIを提供する方針を明確化しています。
- symbol層での guard 過剰拒否を避けるため、policy_relaxed の採用理由と互換性上の扱いを文書に明記しました。
- trace-index は `UCEL-DECIMAL-IMPL-SYMBOL-002` エントリのみ更新して成果物と検証証跡を紐付けました。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照のみ）OK
- MISSING: none
