import asyncio
import json
import logging

from fastapi.testclient import TestClient

from services.marketdata.app.main import _poller, app, MarketDataPoller, PollerConfig, TickerSnapshot


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

    status_code, payload = run(poller.latest_payload())

    assert status_code == 200

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

    status_code, payload = run(poller.latest_payload())

    assert status_code == 200

    assert payload["stale"] is False
    assert payload["degraded_reason"] is None
    assert payload["quality"]["status"] == "OK"
    assert payload["mid"] == 101.0


def test_logs_state_transitions_on_failure_and_recovery(caplog) -> None:
    poller = make_poller()
    caplog.set_level(logging.INFO, logger="observability")

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


def test_error_envelope_includes_request_id_header() -> None:
    with TestClient(app) as client:
        response = client.get("/ticker/latest", headers={"x-request-id": "req-123"})

    assert response.status_code == 503
    body = response.json()
    assert body["error"]["code"] == "TICKER_NOT_READY"
    assert body["degraded_reason"] == "UPSTREAM_ERROR"
    assert body["request_id"] == "req-123"
    assert response.headers.get("x-request-id") == "req-123"


def test_latest_ticker_rejects_unsupported_symbol_with_stable_shape() -> None:
    _poller._record_success(
        TickerSnapshot(
            symbol="BTC_JPY",
            ts="2026-02-14T00:00:00Z",
            bid=1.0,
            ask=2.0,
            last=1.5,
            source="gmo",
        )
    )
    with TestClient(app) as client:
        response = client.get("/ticker/latest?symbol=ETH_JPY")

    assert response.status_code == 400
    body = response.json()
    assert body["symbol"] == "ETH_JPY"
    assert body["degraded_reason"] == "UNSUPPORTED_SYMBOL"
    assert body["error"]["code"] == "UNSUPPORTED_SYMBOL"
