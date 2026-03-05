from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Literal

Mode = Literal["SAFE", "WARN", "DEGRADED", "CANCEL_ONLY", "HALT"]


@dataclass(frozen=True)
class InterlockInputs:
    metrics_ok: bool | None
    clock_ok: bool | None
    deps_ok: bool | None
    lease_ok: bool | None
    audit_ok: bool | None
    reconcile_ok: bool | None = None


@dataclass(frozen=True)
class InterlockDecision:
    mode: Mode
    reason: str
    evidence: dict[str, Any]


def decide(inputs: InterlockInputs) -> InterlockDecision:
    required = {
        "metrics_ok": inputs.metrics_ok,
        "clock_ok": inputs.clock_ok,
        "deps_ok": inputs.deps_ok,
        "lease_ok": inputs.lease_ok,
        "audit_ok": inputs.audit_ok,
    }
    missing = [k for k, v in required.items() if v is None]
    if missing:
        return InterlockDecision("HALT", "missing_required_inputs", {"missing": missing})

    if inputs.audit_ok is False:
        return InterlockDecision("HALT", "audit_chain_broken", {})
    if inputs.lease_ok is False:
        return InterlockDecision("CANCEL_ONLY", "lease_missing", {})
    if inputs.deps_ok is False or inputs.metrics_ok is False:
        return InterlockDecision("CANCEL_ONLY", "deps_or_metrics_degraded", {})
    if inputs.clock_ok is False:
        return InterlockDecision("WARN", "clock_drift", {})
    if inputs.reconcile_ok is False:
        return InterlockDecision("CANCEL_ONLY", "reconcile_mismatch", {})
    return InterlockDecision("SAFE", "ok", {})
