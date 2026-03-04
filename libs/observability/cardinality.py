from __future__ import annotations

from dataclasses import dataclass, field

from libs.observability import budget


@dataclass
class CardinalityTracker:
    per_metric: dict[str, set[tuple[tuple[str, str], ...]]] = field(default_factory=dict)
    total: set[tuple[str, tuple[tuple[str, str], ...]]]= field(default_factory=set)

    def observe(self, metric_name: str, label_kv: dict[str, str]) -> tuple[bool, dict[str, str]]:
        cfg = budget.cfg()
        metric_set = self.per_metric.setdefault(metric_name, set())
        key = tuple(sorted((str(k), str(v)) for k, v in label_kv.items()))
        total_key = (metric_name, key)

        if key in metric_set:
            return True, label_kv

        if len(self.total) >= cfg.max_total_unique_series:
            budget.mark_metrics_exceeded()
            return self._on_exceed(metric_name, label_kv)

        if len(metric_set) >= cfg.max_unique_series_per_metric:
            budget.mark_metrics_exceeded()
            return self._on_exceed(metric_name, label_kv)

        metric_set.add(key)
        self.total.add(total_key)
        budget.state().metrics_total_unique_series = len(self.total)
        return True, label_kv

    def _on_exceed(self, metric_name: str, label_kv: dict[str, str]) -> tuple[bool, dict[str, str]]:
        mode = budget.cfg().metrics_on_exceed
        if mode == "aggregate":
            coarse = {"service": label_kv.get("service", "unknown"), "result": "other"}
            return True, coarse
        if mode == "sample":
            if hash((metric_name, tuple(sorted(label_kv.items())))) % 10 == 0:
                return True, label_kv
            return False, label_kv
        return False, label_kv

    def snapshot(self) -> dict[str, int]:
        return {
            "metrics": len(self.per_metric),
            "total_unique": len(self.total),
        }


_TRACKER = CardinalityTracker()


def tracker() -> CardinalityTracker:
    return _TRACKER


def reset_for_tests() -> None:
    _TRACKER.per_metric.clear()
    _TRACKER.total.clear()
