from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from typing import Any, Literal

from app.j_policy_ssot import JPolicySSOT


GateDecision = Literal["ALLOW", "DENY", "CANCEL_ONLY", "HALT"]


@dataclass(frozen=True)
class GateInput:
    op: str
    role: str
    current_mode: str
    safe_mode: str
    metrics_ok: bool | None = None
    clock_ok: bool | None = None
    audit_ok: bool | None = None
    lease_ok: bool | None = None
    deps_ok: bool | None = None
    backoff_active: bool | None = None
    now_utc: datetime | None = None


@dataclass(frozen=True)
class GateResult:
    decision: GateDecision
    reason_code: str
    evidence: dict[str, Any]


def _in_quiet_hours(ssot: JPolicySSOT, now_utc: datetime) -> bool:
    qh = ssot.quiet_hours
    tz = str(qh.get("timezone", "Asia/Tokyo"))
    windows = qh.get("windows", [])
    if tz != "Asia/Tokyo":
        return True

    jst = now_utc.astimezone(timezone.utc).replace(tzinfo=timezone.utc) + timedelta(hours=9)
    weekday = jst.isoweekday()
    hhmm = f"{jst.hour:02d}:{jst.minute:02d}"
    for w in windows:
        if not isinstance(w, dict):
            continue
        days = w.get("days", [])
        if isinstance(days, list) and weekday not in days:
            continue
        start = str(w.get("start", "00:00"))
        end = str(w.get("end", "00:00"))
        if start <= end:
            if start <= hhmm < end:
                return True
        else:
            if hhmm >= start or hhmm < end:
                return True
    return False


def _rbac_allows(ssot: JPolicySSOT, role: str, op: str) -> bool:
    rules = ssot.rbac.get("rules", [])
    for rule in rules:
        if isinstance(rule, dict) and rule.get("role") == role and rule.get("op") == op:
            return bool(rule.get("allow", False))
    return False


def _is_forbidden_op(ssot: JPolicySSOT, op: str) -> tuple[bool, str]:
    forbidden = ssot.forbidden_ops.get("forbidden", [])
    for item in forbidden:
        if isinstance(item, dict) and str(item.get("op", "")) == op:
            return True, str(item.get("reason_code", "J_POLICY_DENY_FORBIDDEN_OP"))
    return False, ""


def decide(ssot: JPolicySSOT, gate_input: GateInput) -> GateResult:
    now = gate_input.now_utc or datetime.now(timezone.utc)
    evidence: dict[str, Any] = {
        "op": gate_input.op,
        "role": gate_input.role,
        "current_mode": gate_input.current_mode,
        "safe_mode": gate_input.safe_mode,
        "now_utc": now.isoformat(),
    }

    required = set(ssot.boundaries.get("required_inputs", []))
    present: dict[str, bool | None] = {
        "metrics": gate_input.metrics_ok,
        "clock": gate_input.clock_ok,
        "audit": gate_input.audit_ok,
        "lease": gate_input.lease_ok,
        "deps": gate_input.deps_ok,
    }
    missing = [k for k in required if k not in present or present[k] is None]
    if missing:
        evidence["missing_required_inputs"] = sorted(missing)
        return GateResult(decision="HALT", reason_code="J_POLICY_DENY_MISSING_REQUIRED_INPUT", evidence=evidence)

    for k in required:
        if present[k] not in (True, False):
            evidence["unknown_input_key"] = k
            return GateResult(decision="HALT", reason_code="J_POLICY_DENY_UNKNOWN_INPUT", evidence=evidence)

    if gate_input.safe_mode in {"SAFE_MODE", "HALTED"}:
        evidence["condition"] = "safety.safe_mode"
        return GateResult(decision="HALT", reason_code="J_POLICY_HALT_SAFE_MODE", evidence=evidence)

    if gate_input.audit_ok is False:
        evidence["condition"] = "audit.chain_broken"
        return GateResult(decision="HALT", reason_code="J_POLICY_HALT_AUDIT_CHAIN_BROKEN", evidence=evidence)

    if gate_input.lease_ok is False and gate_input.op in {"live_send_new_order", "live_send_replace"}:
        evidence["condition"] = "lease.missing"
        return GateResult(decision="DENY", reason_code="J_POLICY_DENY_LEASE_MISSING", evidence=evidence)

    if gate_input.deps_ok is False or gate_input.metrics_ok is False or gate_input.backoff_active is True:
        evidence["condition"] = "deps.slo_breach_or_backoff"
        if gate_input.op in {"live_send_cancel", "live_send_flatten"}:
            return GateResult(decision="ALLOW", reason_code="J_POLICY_ALLOW", evidence=evidence)
        return GateResult(decision="CANCEL_ONLY", reason_code="J_POLICY_CANCEL_ONLY_DEGRADED", evidence=evidence)

    if gate_input.current_mode == "CANCEL_ONLY":
        if gate_input.op in {"live_send_cancel", "live_send_flatten"}:
            return GateResult(decision="ALLOW", reason_code="J_POLICY_ALLOW", evidence=evidence)
        return GateResult(decision="CANCEL_ONLY", reason_code="J_POLICY_CANCEL_ONLY_DEGRADED", evidence=evidence)

    forbidden, reason_code = _is_forbidden_op(ssot, gate_input.op)
    if forbidden and gate_input.role not in {"oncall", "admin"}:
        evidence["condition"] = "policy.forbidden_op"
        return GateResult(decision="DENY", reason_code=reason_code or "J_POLICY_DENY_FORBIDDEN_OP", evidence=evidence)

    if _in_quiet_hours(ssot, now) and gate_input.op not in {"view_status"}:
        evidence["condition"] = "policy.quiet_hours"
        return GateResult(decision="DENY", reason_code="J_POLICY_DENY_QUIET_HOURS", evidence=evidence)

    if not _rbac_allows(ssot, gate_input.role, gate_input.op):
        evidence["condition"] = "policy.rbac"
        return GateResult(decision="DENY", reason_code="J_POLICY_DENY_RBAC", evidence=evidence)

    return GateResult(decision="ALLOW", reason_code="J_POLICY_ALLOW", evidence=evidence)
