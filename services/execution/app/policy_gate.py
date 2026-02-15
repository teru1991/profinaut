from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Literal

PolicyDecision = Literal["ALLOW", "BLOCK", "THROTTLE", "REDUCE_ONLY", "CLOSE_ONLY", "FLATTEN", "HALT"]
PolicyAction = Literal["ORDER_INTENT", "CANCEL", "REPLACE"]


@dataclass(frozen=True)
class PolicyGateInput:
    action: PolicyAction
    exchange: str
    safe_mode: str
    live_enabled: bool
    live_mode: str
    live_backoff_until_utc: datetime | None
    degraded_reason: str | None = None


@dataclass(frozen=True)
class PolicyGateResult:
    decision: PolicyDecision
    reason_code: str
    evidence: dict[str, Any] | None = None



def evaluate_policy_gate(gate_input: PolicyGateInput) -> PolicyGateResult:
    """Evaluate a centralized execution policy gate for all live/upstream entrypoints."""
    # 1) SAFE_MODE / kill-switch / hard safety flags
    if gate_input.safe_mode in {"SAFE_MODE", "HALTED"} and gate_input.action in {"ORDER_INTENT", "REPLACE"}:
        return PolicyGateResult(
            decision="BLOCK",
            reason_code="SAFE_MODE_BLOCKED",
            evidence={
                "safe_mode": gate_input.safe_mode,
                "action": gate_input.action,
            },
        )

    # 2) risk limits (not implemented in this service yet; reserved precedence slot)

    # 3) dry-run / live gating
    if gate_input.exchange == "gmo":
        if not gate_input.live_enabled:
            return PolicyGateResult(decision="BLOCK", reason_code="LIVE_DISABLED")
        if gate_input.live_mode.strip().lower() != "live":
            return PolicyGateResult(decision="BLOCK", reason_code="DRY_RUN_ONLY")

        now = datetime.now(timezone.utc)
        if gate_input.live_backoff_until_utc is not None and now < gate_input.live_backoff_until_utc:
            return PolicyGateResult(
                decision="THROTTLE",
                reason_code="LIVE_DEGRADED",
                evidence={"degraded_reason": gate_input.degraded_reason or "Execution service degraded"},
            )

    # 4) allow
    return PolicyGateResult(decision="ALLOW", reason_code="POLICY_ALLOW")
