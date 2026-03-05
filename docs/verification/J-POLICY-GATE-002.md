# Verification: J-POLICY-GATE-002

## Changed files
- docs/specs/domains/J/reason_codes.yml
- docs/specs/domains/J/mode_machine.yml
- docs/specs/domains/J_risk_policy_gate.md
- services/execution/app/policy_gate.py
- services/execution/app/j_policy_yaml_min.py
- services/execution/app/j_policy_ssot.py
- services/execution/app/j_policy_decision.py
- services/execution/app/j_policy_selfcheck.py
- services/execution/tests/test_policy_gate.py
- services/execution/tests/test_j_policy_invariants.py
- scripts/ci/execution_tests.sh
- .github/workflows/ci.yml
- docs/verification/J-POLICY-GATE-002.md

## What / Why
- J Policy Gate を SSOT（docs/specs/domains/J/*.yml）駆動に移行し、SSOTロード失敗時の fail-close を実装。
- unknown/missing required input を HALT として処理する Invariant を `decide()` で固定。
- SAFE_MODE 条件を SSOT 側にも追加し、実装との整合を取った。
- runtime self-check (`j_policy_selfcheck.py`) を追加し、SSOT存在とロード検証を API/内部から呼べる形で提供。
- services/execution 向けの CI 実行入口として `scripts/ci/execution_tests.sh` を追加し、`ci.yml` に job を追加。

## Self-check results
- Allowed-path check OK: pass（docs/services/scripts/.github/workflows のみ変更）
- Tests added/updated:
  - services/execution/tests/test_policy_gate.py
  - services/execution/tests/test_j_policy_invariants.py
- Commands:
  - `PYTHONPATH=services/execution pytest -q services/execution/tests/test_policy_gate.py services/execution/tests/test_j_policy_invariants.py` => 7 passed
  - `PYTHONPATH=services/execution pytest -q services/execution/tests` => 2 failed / 33 passed（既存 `test_api.py` の healthz/capabilities 期待値乖離）
- CI wiring:
  - `.github/workflows/ci.yml` に `execution-tests` job を追加
- Secrets scan:
  - `grep -RInE '(API_KEY|SECRET|TOKEN|Authorization:|Bearer )' services/execution/app docs/specs/domains/J scripts/ci || true`
  - 既存ファイル（live.py/auth.py/main.py 等）で既知ヒット。今回追加ファイルに秘密値追加なし。
- docsリンク存在チェック:
  - N/A

## ★履歴確認の証拠（必須）
- git log --oneline -n 50:
  - HEAD基点は `f233ac03`（J-POLICY-SSOT-001）。直近 merge #450〜#453 系列と衝突しない局所実装で進行。
- git log --merges --oneline -n 30:
  - #450〜#453 の連続 merge。execution policy gate 実装に直接競合する同時編集は限定的。
- git merge-base HEAD origin/<default-branch>:
  - 実行不可（この環境で `origin` remote が未設定）。
- policy_gate.py blame summary:
  - 最終履歴 `376cca2`（UG-P0-110）で SAFE_MODE優先・GMOのみlive判定の意図を確認。
  - 本タスクではこの入口互換（SAFE_MODE/LIVE_DISABLED/DRY_RUN_ONLY/LIVE_DEGRADED）を維持しつつ、SSOT駆動評価を追加。

## Notes
- YAMLパーサはSSOTで使用しているサブセット（dict/list/scalar/inline-list）に限定し、未知構文は例外で `JPolicySSOTError` に集約。
- API統合系の既存テスト失敗は本タスク変更外の期待値乖離が含まれるため、別タスクでの調整が必要。
