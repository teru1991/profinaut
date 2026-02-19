from __future__ import annotations

import sqlite3
from pathlib import Path

from fastapi.testclient import TestClient

from services.marketdata.app import main
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.gold_cache import HotCache
from services.marketdata.app.gold_materializer import materialize_gold


async def _idle_poller() -> None:
    return None


def _prep(db_file: Path) -> sqlite3.Connection:
    conn = sqlite3.connect(db_file)
    apply_migrations(conn)
    return conn


def test_gold_materializer_builds_latest_and_ohlcv(tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    conn = _prep(db_file)
    conn.execute(
        """
        INSERT INTO md_best_bid_ask(raw_msg_id, venue_id, market_id, instrument_id, bid_px, bid_qty, ask_px, ask_qty, event_ts, received_ts, extra_json)
        VALUES ('r1','gmo','spot','btc_jpy',100,1,101,2,'2026-01-01T00:00:00Z','2026-01-01T00:00:01Z','{}')
        """
    )
    conn.execute(
        """
        INSERT INTO md_ohlcv(raw_msg_id, venue_id, market_id, instrument_id, timeframe, open_ts, open, high, low, close, volume, is_final, extra_json)
        VALUES ('o1','gmo','spot','btc_jpy','1m','2026-01-01T00:00:00Z',1,2,0.5,1.5,10,1,'{}')
        """
    )
    conn.commit()

    result = materialize_gold(conn)

    assert result.bba_rows == 1
    assert result.ticker_latest_rows == 1
    assert result.ohlcv_rows == 1


def test_markets_endpoints_use_gold_and_cache(monkeypatch, tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    conn = _prep(db_file)
    conn.execute(
        """
        INSERT INTO gold_ticker_latest(venue_id, market_id, instrument_id, price, bid_px, ask_px, bid_qty, ask_qty, ts_event, ts_recv, dt, raw_refs)
        VALUES ('gmo','spot','btc_jpy',100.5,100,101,1,2,'2026-01-01T00:00:00Z','2099-01-01T00:00:00Z','2099-01-01','["r1"]')
        """
    )
    conn.execute(
        """
        INSERT INTO gold_best_bid_ask(venue_id, market_id, instrument_id, bid_px, bid_qty, ask_px, ask_qty, ts_event, ts_recv, dt, raw_refs)
        VALUES ('gmo','spot','btc_jpy',100,1,101,2,'2026-01-01T00:00:00Z','2099-01-01T00:00:00Z','2099-01-01','["r1"]')
        """
    )
    conn.execute(
        """
        INSERT INTO gold_ohlcv_1m(venue_id, market_id, instrument_id, ts_bucket, open, high, low, close, volume, is_final, dt, raw_refs)
        VALUES ('gmo','spot','btc_jpy','2026-01-01T00:00:00Z',1,2,0.5,1.5,10,1,'2026-01-01','["o1"]')
        """
    )
    conn.commit()

    monkeypatch.setenv("DB_DSN", f"sqlite:///{db_file}")
    monkeypatch.setattr(main, "_gold_cache", HotCache(default_ttl_seconds=30.0, jitter_seconds=0.0))
    monkeypatch.setattr(main._poller, "run_forever", _idle_poller)

    with TestClient(main.app) as client:
        ticker = client.get("/markets/ticker/latest?venue=gmo&symbol=BTC_JPY")
        bba = client.get("/markets/bba/latest?venue=gmo&symbol=BTC_JPY")
        ohlcv = client.get("/markets/ohlcv?venue=gmo&symbol=BTC_JPY&tf=1m&from=2026-01-01T00:00:00Z&to=2026-01-01T00:01:00Z")

    assert ticker.status_code == 200
    assert ticker.json()["price"] == 100.5
    assert bba.status_code == 200
    assert bba.json()["bid"] == 100
    assert ohlcv.status_code == 200
    assert len(ohlcv.json()["candles"]) == 1


def test_cache_invalidation_behavior() -> None:
    cache = HotCache(default_ttl_seconds=60.0, jitter_seconds=0.0)
    cache.set("ticker_latest:gmo:BTC_JPY", {"price": 1})
    assert cache.get("ticker_latest:gmo:BTC_JPY") == {"price": 1}
    cache.invalidate("ticker_latest:gmo:BTC_JPY")
    assert cache.get("ticker_latest:gmo:BTC_JPY") is None
