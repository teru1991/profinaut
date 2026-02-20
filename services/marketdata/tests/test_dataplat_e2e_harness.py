from __future__ import annotations

import json

from services.marketdata.app.e2e_harness import HarnessConfig, run_harness


def test_e2e_harness_deterministic_for_same_seed() -> None:
    first = run_harness(HarnessConfig(seed=11, rate=30, duration_s=2))
    second = run_harness(HarnessConfig(seed=11, rate=30, duration_s=2))

    assert first.pass_all
    assert second.pass_all
    assert first.generated == second.generated
    assert first.accepted == second.accepted
    assert first.rejected == second.rejected
    assert first.dedupe_dropped == second.dedupe_dropped
    assert first.bronze_lines == second.bronze_lines
    assert first.silver_trades == second.silver_trades
    assert first.silver_bba == second.silver_bba
    assert first.silver_ohlcv == second.silver_ohlcv
    assert first.silver_events == second.silver_events
    assert first.anomalies == second.anomalies
    assert first.restart_no_growth == second.restart_no_growth
    assert first.api_unavailable_status == 503


def test_e2e_harness_reports_gold_serving_and_perf() -> None:
    summary = run_harness(HarnessConfig(seed=5, rate=40, duration_s=2))
    dump = json.dumps(summary.__dict__)

    assert summary.pass_all
    assert summary.bronze_lines >= summary.accepted - summary.dedupe_dropped
    assert summary.silver_trades > 0
    assert summary.silver_bba > 0
    assert summary.restart_no_growth
    assert summary.objectstore_degraded
    assert summary.bronze_p95_ms > 0
    assert summary.api_hit_p95_ms > 0
    assert summary.api_miss_p95_ms > 0
    assert '"api_unavailable_status": 503' in dump
