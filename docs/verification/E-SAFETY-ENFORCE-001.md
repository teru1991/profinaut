# Verification: E-SAFETY-ENFORCE-001

## Changed files
- services/execution/app/e_lease.py
- services/execution/app/e_audit_chain.py
- services/execution/app/e_interlocks.py
- services/execution/app/e_safety_adapter.py
- services/execution/app/execution_worker.py
- services/execution/app/i_gate.py
- docs/specs/domains/E_safety.md
- services/execution/tests/test_e_lease_failclose.py
- services/execution/tests/test_e_audit_failclose.py
- services/execution/tests/test_e_interlock_missing_inputs.py
- services/execution/tests/test_e_execution_send_hard_enforce.py
- docs/verification/E-SAFETY-ENFORCE-001.md

## What / Why
- Execution の最終送信点（`execution_worker.worker_step`）で E lease + audit chain verify + interlock を先に評価し、fail-close を物理強制。
- J gate の判定が ALLOW でも、E or audit がNGなら送信前に OUTBOX_BLOCKED へ収束する経路を追加。
- 送信直前に audit chain append を必須化し、append失敗時は送信しない（fail-close）。
- Interlock で required inputs 欠損を HALT 相当に固定し、lease切れ/監査不整合を安全側へ収束。

## Self-check results
- Allowed-path check OK: pass（docs/services のみ変更）
- Tests added/updated:
  - services/execution/tests/test_e_lease_failclose.py
  - services/execution/tests/test_e_audit_failclose.py
  - services/execution/tests/test_e_interlock_missing_inputs.py
  - services/execution/tests/test_e_execution_send_hard_enforce.py
- Commands:
  - `PYTHONPATH=services/execution pytest -q services/execution/tests/test_e_lease_failclose.py services/execution/tests/test_e_audit_failclose.py services/execution/tests/test_e_interlock_missing_inputs.py services/execution/tests/test_e_execution_send_hard_enforce.py` => 4 passed
  - `PYTHONPATH=services/execution pytest -q services/execution/tests` => 2 failed / 41 passed（既存 `test_api.py` の healthz/capabilities 期待値乖離）
- Secrets scan:
  - `grep -RInE '(API_KEY|SECRET|TOKEN|Authorization:|Bearer )' services/execution/app docs/specs/domains/E_safety.md scripts || true`
  - 既存コード中の既知ヒットのみ。新規に秘密値は追加していない。

## ★履歴確認の証拠（必須）
- git log --oneline -n 50:
  - HEAD起点は `a1f6294`（I/J導入済みコミット）。E enforcement はその send点強化に集中。
- git log --merges --oneline -n 30:
  - #450〜#453 系列の merge を確認。E対応は execution app 局所変更で衝突最小化。
- git merge-base HEAD origin/<default-branch>:
  - 実行不可（この環境では `origin` remote 未設定）。
- safety_lease_renewer / local_kill_runner / execution_worker blame summary:
  - `worker/safety_lease_renewer.py` は renew(20s TTL, 5s tick)・失敗時 block 通知の意図。
  - `worker/local_kill_runner.py` は push型 halt/kill 実行経路。
  - `execution_worker.py` は Iで追加された single send点のため、ここへ E物理強制を集中実装。
