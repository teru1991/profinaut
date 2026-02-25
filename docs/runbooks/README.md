# Runbooks Index v1.0（可変）
Status: Canonical Index (Runbooks)
Rule: ここは手順（可変）。固定仕様（Core）は docs/specs/** にある。

## 0. 目的
障害時に迷わず「観測→自動復旧→手動介入→証拠（bundle）→再発防止」へ進むための索引。

## 1. 共通テンプレ（必ずこの順で書く）
1) Symptoms（何が起きた）
2) Observations（何を見て確認する）
3) Auto-recovery（自動で何が起きる/起きない）
4) Manual actions（手動介入）
5) Evidence（audit/integrity/gate/replay/support bundle）
6) Rollback/Recovery（戻す・復旧）
7) Postmortem（再発防止：decision/plan/policy更新）

## 2. ドメイン別Runbook（推奨の入口）
### 2.1 Safety / Execution
- 実行拒否 / killswitch / safe mode 遷移
- 429 / auth failure / order cancel failure

### 2.2 Collector / Market Data
- reconnect storm / resubscribe / gap増加 / quarantine
- snapshot+delta再同期

### 2.3 Storage / Integrity
- backlog / IO stall / disk near-full
- integrity FAIL/UNKNOWN の扱い

### 2.4 On-chain
- finality遅延 / reorg / RPC不一致
- backfill / resync

### 2.5 Security / Incident
- forbidden-key scan 検知
- 認証失敗急増 / 侵害疑い / 封じ込め

### 2.6 Release / Governance
- gate FAIL/UNKNOWN
- rollout停止 / rollback

### 2.7 Support Bundle
- 生成条件 / 作り方 / 提出方法（secret-free）

## 3. 参照（正本）
- Safety固定仕様: docs/specs/crosscut/safety_interlock_spec.md
- Audit/Replay固定仕様: docs/specs/crosscut/audit_replay_spec.md
- Support Bundle固定仕様: docs/specs/crosscut/support_bundle_spec.md
- Observability固定仕様: docs/specs/observability/observability_slo_diagnostics_spec.md
