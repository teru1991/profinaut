from __future__ import annotations

import json

from services.marketdata.app.e2e_harness import HarnessConfig, run_harness


def test_e2e_harness_deterministic_for_same_seed() -> None:
    first = run_harness(HarnessConfig(seed=11, rate=30, duration_s=2))
    second = run_harness(HarnessConfig(seed=11, rate=30, duration_s=2))

    assert first.pass_all
    assert second.pass_all
    assert first.deterministic_digest == second.deterministic_digest


def test_e2e_harness_reports_gold_serving_and_perf() -> None:
    summary = run_harness(HarnessConfig(seed=5, rate=40, duration_s=2))
    dump = json.dumps(summary.__dict__)

    assert summary.pass_all
    assert summary.bronze_lines >= summary.accepted - summary.dedupe_dropped
    assert summary.silver_trades > 0
    assert summary.silver_bba > 0
    assert summary.restart_no_growth
    assert summary.objectstore_degraded
    assert summary.objectstore_spool_bounded
    assert summary.clickhouse_degraded_safe
    assert summary.valkey_degraded_safe
    assert summary.queue_depth_stable
    assert summary.bronze_p95_ms > 0
    assert summary.api_hit_p95_ms > 0
    assert summary.api_miss_p95_ms > 0
    assert '"api_unavailable_status": 503' in dump
