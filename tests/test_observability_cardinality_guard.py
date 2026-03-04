from libs.observability import budget
from libs.observability.cardinality import CardinalityTracker
from libs.observability.metrics import metrics_snapshot, observe_metric, reset_for_tests


def test_tracker_blocks_when_limit_exceeded(monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_BUDGET_STRICT", "0")
    budget.reset_for_tests()
    try:
        cfg = budget.cfg()
        cfg.max_unique_series_per_metric = 2
        cfg.max_total_unique_series = 100
        cfg.metrics_on_exceed = "drop"

        tracker = CardinalityTracker()
        assert tracker.observe("m", {"service": "s", "result": "a"})[0] is True
        assert tracker.observe("m", {"service": "s", "result": "b"})[0] is True
        assert tracker.observe("m", {"service": "s", "result": "c"})[0] is False
    finally:
        budget.reset_for_tests()


def test_aggregate_mode_coarsens_labels(monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_BUDGET_STRICT", "0")
    budget.reset_for_tests()
    try:
        cfg = budget.cfg()
        cfg.max_unique_series_per_metric = 1
        cfg.metrics_on_exceed = "aggregate"

        tracker = CardinalityTracker()
        assert tracker.observe("m", {"service": "s", "result": "a"})[0] is True
        allowed, labels = tracker.observe("m", {"service": "s", "result": "b", "symbol": "BTC"})
        assert allowed is True
        assert labels["result"] == "other"
    finally:
        budget.reset_for_tests()


def test_violation_counter_increments(monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_BUDGET_STRICT", "0")
    budget.reset_for_tests()
    reset_for_tests()
    try:
        cfg = budget.cfg()
        cfg.max_unique_series_per_metric = 1
        cfg.metrics_on_exceed = "drop"

        assert observe_metric(service="svc", metric="m", labels={"service": "svc", "result": "ok"}) is True
        assert observe_metric(service="svc", metric="m", labels={"service": "svc", "result": "ng"}) is False
        snap = metrics_snapshot()
        assert snap["violations"]
    finally:
        reset_for_tests()
        budget.reset_for_tests()
