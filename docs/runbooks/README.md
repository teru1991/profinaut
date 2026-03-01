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

## 2. ドメイン別Runbook（カテゴリ索引）
### 2.1 Safety / Execution
- docs/runbooks/ucel_decimal_policy_incidents.md
- docs/runbooks/safety_operations.md
- docs/runbooks/execution_failures_playbook.md
- docs/runbooks/execution_reconciliation_divergence.md

### 2.2 Collector / Market Data
- docs/runbooks/collector_recovery_playbook.md
- docs/runbooks/marketdata-gold-api-errors.md
- docs/runbooks/marketdata-local.md
- docs/runbooks/marketdata-ohlcv-backfill.md
- docs/runbooks/marketdata-registry-onboarding.md
- docs/runbooks/marketdata-replay.md
- docs/runbooks/marketdata_stale_gap_thin.md

### 2.3 Storage / Integrity / Data Platform
- docs/runbooks/backup_restore_drill.md
- docs/runbooks/bronze-writer-howto.md
- docs/runbooks/data-platform-backend-smoke.md
- docs/runbooks/data-platform-env-and-ports.md
- docs/runbooks/data-platform-howto.md
- docs/runbooks/data-platform-local.md
- docs/runbooks/data-platform-ready-check.md
- docs/runbooks/data-platform-samples.md
- docs/runbooks/data-platform-troubleshooting.md
- docs/runbooks/dataplat-ci-triage-notes.md
- docs/runbooks/reconcile-mismatch-repair.md
- docs/runbooks/silver-recompute-howto.md
- docs/runbooks/serving-apis-howto.md

### 2.4 On-chain / IR / Paper
- docs/runbooks/onchain_finality_reorg_playbook.md
- docs/runbooks/ir_ingestion_revision_playbook.md
- docs/runbooks/paper_e2e.md

### 2.5 Security / Incident
- docs/runbooks/incident_response.md
- docs/runbooks/key_rotation.md
- docs/runbooks/supply-chain-security.md
- docs/runbooks/troubleshooting/bots-502.md

### 2.6 Release / Governance
- docs/runbooks/e2e-smoke-runbook.md
- docs/runbooks/pr-preflight.md

### 2.7 Support Bundle
- docs/runbooks/support_bundle_generation.md

## 3. 参照（正本）
- Runbook Format Spec（固定）: docs/specs/system/runbook_index.md
- Safety固定仕様: docs/specs/crosscut/safety_interlock_spec.md
- Audit/Replay固定仕様: docs/specs/crosscut/audit_replay_spec.md
- Support Bundle固定仕様: docs/specs/crosscut/support_bundle_spec.md
- Observability固定仕様: docs/specs/observability/observability_slo_diagnostics_spec.md
