from libs.observability import budget
from libs.observability.metrics import metrics_snapshot, observe_http_request, reset_for_tests


def test_metrics_drop_after_budget_exceeded(monkeypatch):
    monkeypatch.setenv("PROFINAUT_OBS_BUDGET_STRICT", "0")
    budget.reset_for_tests()
    reset_for_tests()
    try:
        cfg = budget.cfg()
        cfg.max_unique_series_per_metric = 3
        cfg.max_total_unique_series = 10
        cfg.metrics_on_exceed = "drop"

        for idx in range(20):
            observe_http_request(service="svc", path=f"/p/{idx}", method="GET", status="200")

        snap = metrics_snapshot()
        assert snap["series"] <= 3
        assert snap["violations"]
    finally:
        reset_for_tests()
        budget.reset_for_tests()
