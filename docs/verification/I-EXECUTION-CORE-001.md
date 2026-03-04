# Verification: I-EXECUTION-CORE-001

## Changed files
- services/execution/app/storage.py
- services/execution/app/main.py
- services/execution/app/exchange_gateway.py
- services/execution/app/exchange_send_adapter.py
- services/execution/app/execution_worker.py
- services/execution/app/i_types.py
- services/execution/app/i_events.py
- services/execution/app/i_inbox.py
- services/execution/app/i_outbox.py
- services/execution/app/i_scheduler.py
- services/execution/app/i_state_machine.py
- services/execution/app/i_reconcile.py
- services/execution/app/i_gate.py
- services/execution/tests/test_i_outbox_inbox.py
- services/execution/tests/test_i_lane0_priority.py
- services/execution/tests/test_i_gate_deny_blocks_send.py
- services/execution/tests/test_i_reconcile_unknown.py
- docs/specs/domains/I_execution.md
- docs/verification/I-EXECUTION-CORE-001.md

## What / Why
- Execution の durable outbox/inbox/eventlog を SQLite に追加し、再送/重複に耐える骨格を実装。
- Lane0（cancel/flatten）優先を `i_scheduler.py` と `i_outbox.py` の優先 dequeue で固定。
- Unknown を許容しつつ reconcile で収束させる `i_state_machine.py` / `i_reconcile.py` を追加。
- J policy を送信直前に評価する `i_gate.py` と worker (`execution_worker.py`) を追加。
- `main.py` に single-egress helper (`_single_egress_send`) を追加し、GMO new/cancel を outbox+gateway 経由へ統一。

## Self-check results
- Allowed-path check OK: pass（docs/services のみ）
- Tests added/updated:
  - services/execution/tests/test_i_outbox_inbox.py
  - services/execution/tests/test_i_lane0_priority.py
  - services/execution/tests/test_i_gate_deny_blocks_send.py
  - services/execution/tests/test_i_reconcile_unknown.py
- Commands:
  - `PYTHONPATH=services/execution pytest -q services/execution/tests/test_i_outbox_inbox.py services/execution/tests/test_i_lane0_priority.py services/execution/tests/test_i_gate_deny_blocks_send.py services/execution/tests/test_i_reconcile_unknown.py` => 4 passed
  - `PYTHONPATH=services/execution pytest -q services/execution/tests` => 2 failed / 37 passed（既存 `test_api.py` の healthz/capabilities 期待値乖離）
- Secrets scan:
  - `grep -RInE '(API_KEY|SECRET|TOKEN|Authorization:|Bearer )' services/execution/app docs/specs/domains/I_execution.md scripts || true`
  - 既存コード中の既知ヒットのみ。新規に秘密値は追加していない。

## ★履歴確認の証拠（必須）
- git log --oneline -n 50:
  - HEAD起点 `a1f6294`（J policy gate導入）。直近の #450〜#453 merge 系列と競合しないよう、I実装を新規 `i_*.py` 中心で追加。
- git log --merges --oneline -n 30:
  - merge履歴は主に diagnostics/observability 系。execution コア追加と直接競合する変更は限定的。
- git merge-base HEAD origin/<default-branch>:
  - 実行不可（この環境は `origin` remote 未設定）。
- storage.py / main.py blame summary:
  - storage は既存で SQLite idempotency_map のみ保持。今回 outbox/inbox/events を追記し既存 API 署名を維持。
  - main は既存 SAFE_MODE/LIVE gating の意図を維持しつつ、送信の出口を `_single_egress_send` に集約。

