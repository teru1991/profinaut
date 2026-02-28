# UCEL-DECIMAL-A1-001 Verification

## 1) Changed files (`git diff --name-only`)
- docs/README.md
- docs/specs/ucel/decimal_policy/A1_core_policy_and_tick_step.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-a1-001.md

## 2) What / Why
- UCEL Decimal Policy（A-1）の正本仕様を docs に新規固定し、丸め/比較/ガード/tick-step の責務を明文化しました。
- 後続タスクで Rust 実装へ転記可能なように、`mod/policy/guard/tick_step` の雛形コードを仕様文書内に完全収録しました。
- docs 入口から辿れるよう `docs/README.md` に最小限の 1 行リンクを追加しました。
- 進捗追跡のため `trace-index` に TASK-ID `UCEL-DECIMAL-A1-001` のエントリ（branch/artifacts/verification_evidence）を追加しました。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照だけ）OK
- MISSING:
  - docs/specs/UCEL_v1.1.4.md
  - docs/ARCH_UCEL_CRATE_GRAPH.md
  - docs/USAGE_HUB.md
  - docs/ARCH_UCEL_INVOKER.md
  - docs/USAGE_INVOKER.md
  - docs/system-common-templates-001
  - docs/doc-ucel-lib-hardening-plan-001
  - docs/ucel-decimal-a1-001
