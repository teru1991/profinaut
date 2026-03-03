from __future__ import annotations

from dataclasses import dataclass

from libs.safety_core.models import SafetyMode


@dataclass(frozen=True, slots=True)
class InterlockRule:
    rule_id: str
    category: str
    severity: str
    recommended_mode: SafetyMode
    latch: bool
    default_threshold: dict[str, int | float]
    description: str


INTERLOCK_RULES: tuple[InterlockRule, ...] = (
    InterlockRule(
        rule_id="OBS_UNKNOWN",
        category="OBS",
        severity="WARN",
        recommended_mode=SafetyMode.SAFE,
        latch=False,
        default_threshold={"unknown_ratio": 0.2},
        description="Unknown observability classifications exceed threshold.",
    ),
    InterlockRule(
        rule_id="CLOCK_CRITICAL",
        category="CLOCK",
        severity="CRITICAL",
        recommended_mode=SafetyMode.EMERGENCY_STOP,
        latch=True,
        default_threshold={"drift_ms": 1500},
        description="Clock drift is beyond critical safety range.",
    ),
    InterlockRule(
        rule_id="RECONCILE_MISMATCH",
        category="RECONCILE",
        severity="CRITICAL",
        recommended_mode=SafetyMode.EMERGENCY_STOP,
        latch=True,
        default_threshold={"mismatch_count": 1},
        description="Reconciliation mismatch indicates ledger inconsistency.",
    ),
    InterlockRule(
        rule_id="ORDER_STORM",
        category="EXEC",
        severity="CRITICAL",
        recommended_mode=SafetyMode.EMERGENCY_STOP,
        latch=True,
        default_threshold={"submit_rate_per_sec": 120},
        description="Order submit throughput exceeds control envelope.",
    ),
    InterlockRule(
        rule_id="CANCEL_LOOP",
        category="EXEC",
        severity="WARN",
        recommended_mode=SafetyMode.SAFE,
        latch=False,
        default_threshold={"cancel_rate_per_sec": 80},
        description="Cancel loop suspected due to high cancellation churn.",
    ),
    InterlockRule(
        rule_id="UNKNOWN_SPIKE",
        category="UNKNOWN",
        severity="WARN",
        recommended_mode=SafetyMode.SAFE,
        latch=False,
        default_threshold={"unknown_events_per_min": 50},
        description="Unknown classified events spike beyond baseline.",
    ),
    InterlockRule(
        rule_id="DISK_WAL_DANGER",
        category="DATA",
        severity="CRITICAL",
        recommended_mode=SafetyMode.EMERGENCY_STOP,
        latch=True,
        default_threshold={"disk_free_percent": 5},
        description="WAL/disk safety margin exhausted.",
    ),
    InterlockRule(
        rule_id="PNL_EXPOSURE_JUMP",
        category="RISK",
        severity="CRITICAL",
        recommended_mode=SafetyMode.EMERGENCY_STOP,
        latch=True,
        default_threshold={"exposure_jump_ratio": 0.35},
        description="Abrupt PnL/exposure jump breaches emergency threshold.",
    ),
)
