from __future__ import annotations

from dataclasses import dataclass, field

from libs.observability import budget
from libs.observability.audit import emit_audit_event
from libs.observability.cardinality import tracker

FORBIDDEN_LABEL_KEYS = {"symbol", "order_id", "price", "trace_id", "run_id", "instance_id"}


@dataclass
class MetricStore:
    counts: dict[str, int] = field(default_factory=dict)
    violations_total: dict[tuple[str, str], int] = field(default_factory=dict)
    budget_exceeded_gauge: dict[tuple[str, str], int] = field(default_factory=dict)


_STORE = MetricStore()


def observe_metric(*, service: str, metric: str, labels: dict[str, str], value: int = 1) -> bool:
    forbidden = {k for k in labels if k in FORBIDDEN_LABEL_KEYS}
    if forbidden:
        if budget.is_strict_mode():
            raise ValueError(f"forbidden metric label keys: {sorted(forbidden)}")
        key = (service, metric)
        _STORE.violations_total[key] = _STORE.violations_total.get(key, 0) + 1
        _STORE.budget_exceeded_gauge[(service, "metrics")] = 1
        budget.mark_metrics_exceeded()
        emit_audit_event(
            "cardinality_violation",
            service=service,
            details={"metric": metric, "reason": "forbidden_labels"},
        )
        labels = {k: v for k, v in labels.items() if k not in forbidden}

    allowed, effective_labels = tracker().observe(metric, labels)
    if not allowed:
        key = (service, metric)
        _STORE.violations_total[key] = _STORE.violations_total.get(key, 0) + 1
        _STORE.budget_exceeded_gauge[(service, "metrics")] = 1
        emit_audit_event(
            "cardinality_violation",
            service=service,
            details={"metric": metric, "policy": budget.cfg().metrics_on_exceed},
        )
        return False

    label_sig = "|".join(f"{k}:{v}" for k, v in sorted(effective_labels.items()))
    series_key = f"{metric}#{label_sig}"
    _STORE.counts[series_key] = _STORE.counts.get(series_key, 0) + value
    if budget.state().metrics_exceeded:
        _STORE.budget_exceeded_gauge[(service, "metrics")] = 1
    return True


def observe_http_request(*, service: str, path: str, method: str, status: str) -> bool:
    return observe_metric(
        service=service,
        metric="profinaut_http_requests_total",
        labels={"service": service, "path": path, "method": method, "status": status},
        value=1,
    )


def metrics_snapshot() -> dict[str, object]:
    return {
        "series": len(_STORE.counts),
        "violations": dict(_STORE.violations_total),
        "budget_gauge": dict(_STORE.budget_exceeded_gauge),
    }


def reset_for_tests() -> None:
    _STORE.counts.clear()
    _STORE.violations_total.clear()
    _STORE.budget_exceeded_gauge.clear()
