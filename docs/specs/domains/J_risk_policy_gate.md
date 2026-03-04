# J: Risk / Policy Gate (SSOT)

## Purpose
J は「安全側へ確実に倒す」ための意思決定ルールを SSOT として固定する。
- 目的: 事故（誤発注/想定外挙動/観測不能/監査不能/依存障害）を “許さない”
- 原則: unknown / missing / 破損 は fail-close（DENY/CANCEL_ONLY/HALT のいずれか）

## Decision model (high-level)
- Inputs（例）:
  - safety_lease_ok (E)
  - audit_chain_ok (E)
  - deps_health (SLO)
  - metrics_health (required metrics)
  - clock_health (skew/drift)
  - current_mode (SAFE/WARN/DEGRADED/CANCEL_ONLY/HALT)
  - actor/op/scope (RBAC/forbidden/quiet hours)
- Outputs:
  - ALLOW / DENY / CANCEL_ONLY / HALT
- Latch:
  - CRITICAL は latch 可能（解除は break-glass / suppress に限定。監査必須）

## Files (SSOT)
- boundaries.yml
- reason_codes.yml
- mode_machine.yml
- exception_templates.yml
- observability_contract.yml
- rbac_matrix.yml
- quiet_hours.yml
- forbidden_ops.yml
- failure_modes.md
- dependency_slo.yml
- degraded_levels.yml
- retention_redaction.md
- bootstrap.md

## YAML Schema (contract)
### boundaries.yml
Top-level keys:
- schema_version: int (required)
- fail_close:
  - on_unknown_input: bool (required)
  - on_missing_required_input: bool (required)
- required_inputs: list[str] (required)
  - examples: ["metrics", "clock", "audit", "lease", "deps"]
- execution:
  - single_egress_required: bool (required)
  - lane0_priority_required: bool (required)
- portfolio:
  - tamper_evident_ledger_required: bool (required)

### reason_codes.yml
Top-level keys:
- schema_version: int (required)
- reasons: list[Reason] (required)
Reason:
- code: str (required, unique)
- severity: one of ["INFO","WARN","ERROR","CRITICAL"] (required)
- default_action: one of ["ALLOW","DENY","CANCEL_ONLY","HALT"] (required)
- description: str (required)
- doc: str (optional)  # link/reference note

### mode_machine.yml
Top-level keys:
- schema_version: int (required)
- states: list[str] (required)  # must include SAFE,WARN,DEGRADED,CANCEL_ONLY,HALT
- transitions: list[Transition] (required)
Transition:
- from: str (required)
- to: str (required)
- when_all: list[str] (required)  # condition keys; semantics are defined by implementation (next task)
- latch: bool (required)          # true => requires explicit break-glass/suppress to exit
- reason_code: str (required)     # must exist in reason_codes.yml

### exception_templates.yml
Top-level keys:
- schema_version: int (required)
- break_glass:
  - max_ttl_sec: int (required)
  - require_actor: bool (required)
  - require_reason: bool (required)
  - require_evidence: bool (required)
- suppress:
  - max_ttl_sec: int (required)
  - require_actor: bool (required)
  - require_reason: bool (required)
  - require_evidence: bool (required)

### observability_contract.yml
Top-level keys:
- schema_version: int (required)
- required_log_keys: list[str] (required)
- required_metrics: list[str] (required)
- missing_metrics_policy: one of ["FAIL_CLOSE","DEGRADED","WARN_ONLY"] (required)

### rbac_matrix.yml
Top-level keys:
- schema_version: int (required)
- roles: list[str] (required)
- ops: list[str] (required)
- rules: list[Rule] (required)
Rule:
- role: str (required)
- op: str (required)
- allow: bool (required)

### quiet_hours.yml
Top-level keys:
- schema_version: int (required)
- timezone: str (required) # IANA timezone, e.g. "Asia/Tokyo"
- windows: list[Window] (required)
Window:
- name: str (required)
- start: "HH:MM" (required)
- end: "HH:MM" (required)
- days: list[int] (required) # 1=Mon ... 7=Sun

### forbidden_ops.yml
Top-level keys:
- schema_version: int (required)
- forbidden: list[Forbidden] (required)
Forbidden:
- op: str (required)
- reason_code: str (required)
- note: str (optional)

### dependency_slo.yml
Top-level keys:
- schema_version: int (required)
- dependencies: list[Dep] (required)
Dep:
- name: str (required)
- kind: one of ["exchange_api","ws_feed","db","object_store","vault","dns","time_source"] (required)
- slo:
  - max_error_rate: float (required)
  - max_p95_ms: int (required)
  - max_consecutive_failures: int (required)
- on_breach:
  - action: one of ["DEGRADED","CANCEL_ONLY","HALT"] (required)
  - reason_code: str (required)

### degraded_levels.yml
Top-level keys:
- schema_version: int (required)
- levels: list[Level] (required)
Level:
- name: str (required)  # e.g. "DEGRADED", "CANCEL_ONLY", "HALT"
- triggers_all: list[str] (required) # condition keys
- reason_code: str (required)

## Invariants (must be proven by tests in next task)
1) Unknown input/missing required input => fail-close (DENY/CANCEL_ONLY/HALT; boundaries.ymlで指定)
2) audit_chain_ok == false => HALT (latch)
3) safety_lease_ok == false AND op is live_send/new_order => DENY or CANCEL_ONLY (policy)
4) In CANCEL_ONLY: allow cancel/flatten (Lane0), deny new/replace
5) quiet hours + forbidden op => DENY
6) SSOT parse/validate failure => fail-close (implementation must not allow live)
