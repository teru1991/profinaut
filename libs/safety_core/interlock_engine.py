from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime

from libs.safety_core.engine import compute_decision
from libs.safety_core.interlock_catalog import INTERLOCK_RULES, InterlockRule
from libs.safety_core.models import ScopeKind, SafetyDecision, SafetyDirective


@dataclass(slots=True)
class InterlockTrigger:
    rule_id: str
    reason: str
    mode: str
    latched: bool
    evaluated_at: str


class InterlockEngine:
    def __init__(self, rules: tuple[InterlockRule, ...] = INTERLOCK_RULES) -> None:
        self._rules = rules
        self._last_triggers: list[InterlockTrigger] = []

    def evaluate(self, snapshot: dict[str, float | int], now: datetime | None = None) -> list[SafetyDirective]:
        ts = now or datetime.now(UTC)
        directives: list[SafetyDirective] = []
        triggers: list[InterlockTrigger] = []

        for rule in self._rules:
            if self._is_triggered(rule, snapshot):
                reason = f"interlock:{rule.rule_id} triggered"
                directive = SafetyDirective(
                    scope_kind=ScopeKind.GLOBAL,
                    selector="*",
                    mode=rule.recommended_mode,
                    ttl_seconds=300,
                    reason=reason,
                    actor="interlock_daemon",
                    evidence={"trace_id": f"interlock-{rule.rule_id}", "run_id": "interlock-daemon"},
                    issued_at=ts.isoformat(),
                )
                directives.append(directive)
                triggers.append(
                    InterlockTrigger(
                        rule_id=rule.rule_id,
                        reason=reason,
                        mode=rule.recommended_mode.value,
                        latched=rule.latch,
                        evaluated_at=ts.isoformat(),
                    )
                )

        self._last_triggers = triggers
        return directives

    def compose(self, directives: list[SafetyDirective], now: datetime | None = None) -> SafetyDecision:
        return compute_decision(directives, now or datetime.now(UTC))

    def last_triggers(self) -> list[InterlockTrigger]:
        return list(self._last_triggers)

    def _is_triggered(self, rule: InterlockRule, snapshot: dict[str, float | int]) -> bool:
        if rule.rule_id == "OBS_UNKNOWN":
            return float(snapshot.get("unknown_ratio", 0.0)) >= float(rule.default_threshold["unknown_ratio"])
        if rule.rule_id == "CLOCK_CRITICAL":
            return int(snapshot.get("clock_drift_ms", 0)) >= int(rule.default_threshold["drift_ms"])
        if rule.rule_id == "RECONCILE_MISMATCH":
            return int(snapshot.get("reconcile_mismatch_count", 0)) >= int(rule.default_threshold["mismatch_count"])
        if rule.rule_id == "ORDER_STORM":
            return int(snapshot.get("submit_rate_per_sec", 0)) >= int(rule.default_threshold["submit_rate_per_sec"])
        if rule.rule_id == "CANCEL_LOOP":
            return int(snapshot.get("cancel_rate_per_sec", 0)) >= int(rule.default_threshold["cancel_rate_per_sec"])
        if rule.rule_id == "UNKNOWN_SPIKE":
            return int(snapshot.get("unknown_events_per_min", 0)) >= int(rule.default_threshold["unknown_events_per_min"])
        if rule.rule_id == "DISK_WAL_DANGER":
            return int(snapshot.get("disk_free_percent", 100)) <= int(rule.default_threshold["disk_free_percent"])
        if rule.rule_id == "PNL_EXPOSURE_JUMP":
            return float(snapshot.get("exposure_jump_ratio", 0.0)) >= float(rule.default_threshold["exposure_jump_ratio"])
        return False
