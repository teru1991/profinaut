from __future__ import annotations

from dataclasses import asdict, dataclass
from datetime import UTC, datetime
import json
from pathlib import Path


@dataclass(slots=True)
class SloMetricEvent:
    metric: str
    value: float
    ts: str


class SloRecorder:
    def __init__(self, path: Path | None = None) -> None:
        self._path = path or Path("libs/safety_core/_audit_log/slo_metrics.jsonl")
        self._path.parent.mkdir(parents=True, exist_ok=True)

    def record_halt_to_block_ms(self, started_at: datetime, blocked_at: datetime) -> float:
        value = (blocked_at - started_at).total_seconds() * 1000.0
        self._write("halt_to_block_ms", value)
        return value

    def record_lease_expire_to_block_s(self, lease_expired_at: datetime, blocked_at: datetime) -> float:
        value = (blocked_at - lease_expired_at).total_seconds()
        self._write("lease_expire_to_block_s", value)
        return value

    def record_interlock_detect_to_escalate_ms(self, detected_at: datetime, escalated_at: datetime) -> float:
        value = (escalated_at - detected_at).total_seconds() * 1000.0
        self._write("interlock_detect_to_escalate_ms", value)
        return value

    def evaluate_alerts(self, metric: str, value: float) -> list[str]:
        alerts: list[str] = []
        if metric == "halt_to_block_ms" and value > 100:
            alerts.append("ALERT_HALT_BLOCK_SLO_BREACH")
        if metric == "lease_expire_to_block_s" and value > 30:
            alerts.append("ALERT_LEASE_BLOCK_CONVERGENCE_BREACH")
        if metric == "interlock_detect_to_escalate_ms" and value > 300:
            alerts.append("ALERT_INTERLOCK_ESCALATION_BREACH")
        return alerts

    def _write(self, metric: str, value: float) -> None:
        event = SloMetricEvent(metric=metric, value=value, ts=datetime.now(UTC).isoformat())
        with self._path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(asdict(event), ensure_ascii=False) + "\n")
            f.flush()
