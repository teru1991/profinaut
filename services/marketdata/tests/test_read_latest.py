from __future__ import annotations

import sqlite3
from pathlib import Path

from fastapi.testclient import TestClient

from services.marketdata.app import main
from services.marketdata.app.db.schema import apply_migrations


async def _idle_poller() -> None:
    return None


def _prep_db(db_file: Path) -> sqlite3.Connection:
    conn = sqlite3.connect(db_file)
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn)
    return conn


def test_ticker_latest_returns_trade_fallback_and_stale(monkeypatch, tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    conn = _prep_db(db_file)
    conn.execute(
        """
        INSERT INTO md_trades (
            raw_msg_id, venue_id, market_id, instrument_id, source_msg_key,
            price, qty, side, occurred_at, received_ts, extra_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "gmo",
            "spot",
            "btc_jpy",
            "k-1",
            123.4,
            1.0,
            "buy",
            "2026-02-16T00:00:00Z",
            "2026-02-16T00:00:00Z",
            "{}",
        ),
    )
    conn.commit()

    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("READ_STALE_THRESHOLD_SECONDS", "0")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        resp = client.get("/ticker/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy")

    assert resp.status_code == 200
    body = resp.json()
    assert body["found"] is True
    assert body["price"] == 123.4
    assert body["bid"] is None
    assert body["ask"] is None
    assert body["stale"] is True


def test_ticker_latest_prefers_bba(monkeypatch, tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    conn = _prep_db(db_file)
    conn.execute(
        """
        INSERT INTO md_best_bid_ask (
            raw_msg_id, venue_id, market_id, instrument_id,
            bid_px, bid_qty, ask_px, ask_qty, event_ts, received_ts, extra_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "gmo",
            "spot",
            "btc_jpy",
            100.0,
            2.0,
            101.0,
            3.0,
            "2026-02-16T00:00:00Z",
            "2099-01-01T00:00:00Z",
            "{}",
        ),
    )
    conn.commit()

    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        resp = client.get("/ticker/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy")

    assert resp.status_code == 200
    body = resp.json()
    assert body["found"] is True
    assert body["bid"] == 100.0
    assert body["ask"] == 101.0
    assert body["price"] == 100.5


def test_ticker_latest_returns_503_when_db_missing(monkeypatch) -> None:
    monkeypatch.delenv("DB_DSN", raising=False)
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        resp = client.get("/ticker/latest?venue_id=gmo&market_id=spot&instrument_id=btc_jpy")

    assert resp.status_code == 503
    assert resp.json()["code"] == "READ_MODEL_UNAVAILABLE"


def test_ohlcv_latest_and_range_with_stale_semantics(monkeypatch, tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    conn = _prep_db(db_file)
    conn.executemany(
        """
        INSERT INTO md_ohlcv (
            raw_msg_id, venue_id, market_id, instrument_id, timeframe,
            open_ts, open, high, low, close, volume, is_final, extra_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        [
            (
                "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                "gmo",
                "spot",
                "btc_jpy",
                "1m",
                "2026-02-16T00:00:00Z",
                1.0,
                2.0,
                0.5,
                1.5,
                42.0,
                1,
                "{}",
            ),
            (
                "01ARZ3NDEKTSV4RRFFQ69G5FAW",
                "gmo",
                "spot",
                "btc_jpy",
                "1m",
                "2026-02-16T00:01:00Z",
                1.5,
                2.1,
                1.4,
                1.9,
                41.0,
                1,
                "{}",
            ),
        ],
    )
    conn.commit()

    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("LATEST_STALE_MS", "0")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        latest = client.get("/ohlcv/latest?venue_id=gmo&market_id=spot&tf=1m")
        rng = client.get("/ohlcv/range?venue_id=gmo&market_id=spot&tf=1m&from=2026-02-16T00:00:00Z&to=2026-02-16T00:02:00Z&limit=10")
        missing = client.get("/ohlcv/latest?venue_id=gmo&market_id=missing&tf=1m")

    assert latest.status_code == 200
    body = latest.json()
    assert body["found"] is True
    assert body["stale"] is True
    assert body["tf"] == "1m"
    assert len(body["candles"]) == 1
    assert body["candles"][0]["close"] == 1.9

    assert rng.status_code == 200
    rbody = rng.json()
    assert rbody["found"] is True
    assert rbody["tf"] == "1m"
    assert len(rbody["candles"]) == 2

    assert missing.status_code == 200
    assert missing.json()["found"] is False


def test_ohlcv_endpoints_return_explicit_error_when_db_unavailable(monkeypatch) -> None:
    monkeypatch.delenv("DB_DSN", raising=False)
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        latest = client.get("/ohlcv/latest?venue_id=gmo&market_id=spot&tf=1m")
        rng = client.get("/ohlcv/range?venue_id=gmo&market_id=spot&tf=1m&from=2026-02-16T00:00:00Z&to=2026-02-16T00:02:00Z")

    assert latest.status_code == 503
    assert latest.json()["code"] == "READ_MODEL_UNAVAILABLE"

    assert rng.status_code == 503
    assert rng.json()["code"] == "READ_MODEL_UNAVAILABLE"

def test_orderbook_bbo_latest_reports_stale_and_degraded(monkeypatch, tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    conn = _prep_db(db_file)
    conn.execute(
        """
        INSERT INTO md_orderbook_state (
            venue_id, market_id, bid_px, bid_qty, ask_px, ask_qty,
            as_of, last_update_ts, last_seq, degraded, reason
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            "gmo",
            "spot",
            100.0,
            1.0,
            101.0,
            2.0,
            "2026-02-16T00:00:00Z",
            "2026-02-16T00:00:00Z",
            "10",
            0,
            None,
        ),
    )
    conn.commit()

    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setenv("LATEST_STALE_MS", "0")

    with TestClient(main.app) as client:
        resp = client.get("/orderbook/bbo/latest?venue_id=gmo&market_id=spot")

    assert resp.status_code == 200
    body = resp.json()
    assert body["found"] is True
    assert body["stale"] is True
    assert body["degraded"] is True
    assert body["reason"] == "ORDERBOOK_GAP"


def test_gold_api_bad_input_returns_400_with_error_codes(monkeypatch, tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    _prep_db(db_file)

    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        missing_orderbook = client.get("/orderbook/bbo/latest?market_id=spot")
        missing_state = client.get("/orderbook/state?venue_id=gmo")
        invalid_tf = client.get("/ohlcv/latest?venue_id=gmo&market_id=spot&tf=bogus")
        invalid_ts = client.get("/ohlcv/range?venue_id=gmo&market_id=spot&tf=1m&from=bad-ts&to=2026-02-16T00:02:00Z")
        invalid_range = client.get("/ohlcv/range?venue_id=gmo&market_id=spot&tf=1m&from=2026-02-16T00:03:00Z&to=2026-02-16T00:02:00Z")
        partial_ticker = client.get("/ticker/latest?venue_id=gmo&market_id=spot")

    assert missing_orderbook.status_code == 400
    assert missing_orderbook.json()["code"] == "MISSING_REQUIRED_QUERY"

    assert missing_state.status_code == 400
    assert missing_state.json()["code"] == "MISSING_REQUIRED_QUERY"

    assert invalid_tf.status_code == 400
    assert invalid_tf.json()["code"] == "INVALID_TIMEFRAME"

    assert invalid_ts.status_code == 400
    assert invalid_ts.json()["code"] == "INVALID_TIMESTAMP"

    assert invalid_range.status_code == 400
    assert invalid_range.json()["code"] == "INVALID_TIME_RANGE"

    assert partial_ticker.status_code == 400
    assert partial_ticker.json()["code"] == "MISSING_REQUIRED_QUERY"
