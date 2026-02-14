import asyncio
import json
import logging

from services.marketdata.app.main import MarketDataPoller, PollerConfig, TickerSnapshot


def run(coro):
    return asyncio.run(coro)


def make_poller() -> MarketDataPoller:
    return MarketDataPoller(
        PollerConfig(
            interval_seconds=0.01,
            stale_threshold_seconds=5,
            backoff_initial_seconds=1,
            backoff_max_seconds=4,
        )
    )


def test_backoff_exponential_with_cap() -> None:
    poller = make_poller()

    first = poller._record_failure(RuntimeError("boom-1"))
    second = poller._record_failure(RuntimeError("boom-2"))
    third = poller._record_failure(RuntimeError("boom-3"))
    fourth = poller._record_failure(RuntimeError("boom-4"))

    assert [first, second, third, fourth] == [1, 2, 4, 4]
    assert poller._degraded_reason == "UPSTREAM_ERROR"


def test_stale_detection_sets_reason_and_canonical_payload_shape() -> None:
    poller = make_poller()
    snapshot = TickerSnapshot(
        symbol="BTC_JPY",
        ts="2026-02-14T00:00:00Z",
        bid=1.0,
        ask=3.0,
        last=2.0,
        source="gmo",
    )
    poller._record_success(snapshot)
    poller._last_success_monotonic -= 10

    payload = run(poller.latest_payload())

    assert payload["stale"] is True
    assert payload["degraded_reason"] == "STALE_TICKER"
    assert payload["quality"]["status"] == "STALE"
    assert set(payload.keys()) == {
        "symbol",
        "ts",
        "bid",
        "ask",
        "last",
        "mid",
        "source",
        "quality",
        "stale",
        "degraded_reason",
    }


def test_degraded_clears_when_fresh_data_returns() -> None:
    poller = make_poller()

    poller._record_failure(RuntimeError("network"))
    fresh_snapshot = TickerSnapshot(
        symbol="BTC_JPY",
        ts="2026-02-14T00:00:00Z",
        bid=100.0,
        ask=102.0,
        last=101.0,
        source="gmo",
    )
    poller._record_success(fresh_snapshot)

    payload = run(poller.latest_payload())

    assert payload["stale"] is False
    assert payload["degraded_reason"] is None
    assert payload["quality"]["status"] == "OK"
    assert payload["mid"] == 101.0


def test_logs_state_transitions_on_failure_and_recovery(caplog) -> None:
    poller = make_poller()
    caplog.set_level(logging.INFO, logger="marketdata")

    poller._record_failure(RuntimeError("network"))
    poller._record_success(
        TickerSnapshot(
            symbol="BTC_JPY",
            ts="2026-02-14T00:00:00Z",
            bid=10,
            ask=20,
            last=15,
            source="gmo",
        )
    )

    transition_events = []
    for rec in caplog.records:
        try:
            payload = json.loads(rec.message)
        except json.JSONDecodeError:
            continue
        if payload.get("event") == "marketdata_state_transition":
            transition_events.append((payload.get("from_state"), payload.get("to_state")))

    assert ("healthy", "degraded") in transition_events
    assert ("degraded", "healthy") in transition_events
