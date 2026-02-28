# UCEL-DECIMAL-A2-001 Verification

## 1) Changed files (`git diff --name-only`)
- docs/README.md
- docs/specs/ucel/decimal_policy/decimal_policy_spec.md
- docs/specs/ucel/decimal_policy/A2_serde_guard.md
- docs/status/trace-index.json
- docs/verification/ucel-decimal-a2-001.md

## 2) What / Why
- UCEL Decimal Policy（A-2）として、全体仕様（丸め/比較/tick-step/拒否条件/例外/推奨運用）を SSOT 文書に固定しました。
- serde deserialize 時点で guard により不正値を拒否する方針を、`serde.rs` の Rust 雛形（全コード）として docs に固定しました。
- docs の入口から到達可能にするため、`docs/README.md` に Decimal Policy Spec への最小リンクを追記しました。
- 追跡可能性を維持するため、`trace-index` に `UCEL-DECIMAL-A2-001` の branch/artifacts/verification_evidence を追加しました。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照だけ）OK（既存未解決参照は下記 MISSING に記録）
- MISSING:
  - docs/specs/UCEL_v1.1.4.md
  - docs/ARCH_UCEL_CRATE_GRAPH.md
  - docs/USAGE_HUB.md
  - docs/ARCH_UCEL_INVOKER.md
  - docs/USAGE_INVOKER.md
  - docs/system-common-templates-001
  - docs/doc-ucel-lib-hardening-plan-001
  - docs/ucel-decimal-a1-001
  - docs/ucel-decimal-a2-001
  - docs/verification/ucel-decimal-a2-001.md
