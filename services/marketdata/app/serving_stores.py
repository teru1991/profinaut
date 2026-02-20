from __future__ import annotations

import json
import os
import sqlite3
import threading
import time
from dataclasses import dataclass
from typing import Any

from services.marketdata.app.gold_cache import HotCache


@dataclass(frozen=True)
class ReadQueryResult:
    payload: dict[str, Any] | list[dict[str, Any]] | None
    backend: str


class ClickHouseLikeStore:
    """Lightweight serving store with ClickHouse-like schema semantics.

    For local/tests we back this with sqlite (`CLICKHOUSE_DSN=sqlite:///...`).
    """

    def __init__(self, dsn: str | None = None):
        self._dsn = dsn or os.getenv("CLICKHOUSE_DSN") or ""
        self._conn: sqlite3.Connection | None = None
        self._lock = threading.Lock()
        if self._dsn.startswith("sqlite:///"):
            db_path = self._dsn.removeprefix("sqlite:///")
            self._conn = sqlite3.connect(db_path, check_same_thread=False)
            self._conn.execute("PRAGMA foreign_keys = ON")
            self.ensure_schema()

    @property
    def available(self) -> bool:
        return self._conn is not None

    def ensure_schema(self) -> None:
        if self._conn is None:
            return
        self._conn.execute(
            """
            CREATE TABLE IF NOT EXISTS ch_ticker_latest (
                venue_id TEXT NOT NULL,
                market_id TEXT NOT NULL,
                instrument_id TEXT NOT NULL,
                price REAL NOT NULL,
                bid_px REAL,
                ask_px REAL,
                bid_qty REAL,
                ask_qty REAL,
                ts_event TEXT,
                ts_recv TEXT NOT NULL,
                dt TEXT NOT NULL,
                lineage_ref TEXT NOT NULL,
                PRIMARY KEY (venue_id, market_id, instrument_id)
            )
            """
        )
        self._conn.execute(
            """
            CREATE TABLE IF NOT EXISTS ch_best_bid_ask (
                venue_id TEXT NOT NULL,
                market_id TEXT NOT NULL,
                instrument_id TEXT NOT NULL,
                bid_px REAL NOT NULL,
                bid_qty REAL NOT NULL,
                ask_px REAL NOT NULL,
                ask_qty REAL NOT NULL,
                ts_event TEXT,
                ts_recv TEXT NOT NULL,
                dt TEXT NOT NULL,
                lineage_ref TEXT NOT NULL,
                PRIMARY KEY (venue_id, market_id, instrument_id)
            )
            """
        )
        self._conn.execute(
            """
            CREATE TABLE IF NOT EXISTS ch_ohlcv_1m (
                venue_id TEXT NOT NULL,
                market_id TEXT NOT NULL,
                instrument_id TEXT NOT NULL,
                ts_bucket TEXT NOT NULL,
                open REAL NOT NULL,
                high REAL NOT NULL,
                low REAL NOT NULL,
                close REAL NOT NULL,
                volume REAL NOT NULL,
                is_final INTEGER NOT NULL,
                dt TEXT NOT NULL,
                lineage_ref TEXT NOT NULL,
                PRIMARY KEY (venue_id, market_id, instrument_id, ts_bucket)
            )
            """
        )
        self._conn.commit()

    def sync_from_sqlite_gold(self, conn: sqlite3.Connection) -> int:
        if self._conn is None:
            return 0
        copied = 0
        with self._lock:
            for row in conn.execute("SELECT venue_id, market_id, instrument_id, price, bid_px, ask_px, bid_qty, ask_qty, ts_event, ts_recv, dt, raw_refs FROM gold_ticker_latest").fetchall():
                self._conn.execute(
                    """
                    INSERT INTO ch_ticker_latest(venue_id, market_id, instrument_id, price, bid_px, ask_px, bid_qty, ask_qty, ts_event, ts_recv, dt, lineage_ref)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(venue_id, market_id, instrument_id) DO UPDATE SET
                        price=excluded.price, bid_px=excluded.bid_px, ask_px=excluded.ask_px,
                        bid_qty=excluded.bid_qty, ask_qty=excluded.ask_qty, ts_event=excluded.ts_event,
                        ts_recv=excluded.ts_recv, dt=excluded.dt, lineage_ref=excluded.lineage_ref
                    """,
                    row,
                )
                copied += 1
            for row in conn.execute("SELECT venue_id, market_id, instrument_id, bid_px, bid_qty, ask_px, ask_qty, ts_event, ts_recv, dt, raw_refs FROM gold_best_bid_ask").fetchall():
                self._conn.execute(
                    """
                    INSERT INTO ch_best_bid_ask(venue_id, market_id, instrument_id, bid_px, bid_qty, ask_px, ask_qty, ts_event, ts_recv, dt, lineage_ref)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(venue_id, market_id, instrument_id) DO UPDATE SET
                        bid_px=excluded.bid_px, bid_qty=excluded.bid_qty, ask_px=excluded.ask_px,
                        ask_qty=excluded.ask_qty, ts_event=excluded.ts_event, ts_recv=excluded.ts_recv,
                        dt=excluded.dt, lineage_ref=excluded.lineage_ref
                    """,
                    row,
                )
                copied += 1
            for row in conn.execute("SELECT venue_id, market_id, instrument_id, ts_bucket, open, high, low, close, volume, is_final, dt, raw_refs FROM gold_ohlcv_1m").fetchall():
                self._conn.execute(
                    """
                    INSERT INTO ch_ohlcv_1m(venue_id, market_id, instrument_id, ts_bucket, open, high, low, close, volume, is_final, dt, lineage_ref)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(venue_id, market_id, instrument_id, ts_bucket) DO UPDATE SET
                        open=excluded.open, high=excluded.high, low=excluded.low, close=excluded.close,
                        volume=excluded.volume, is_final=excluded.is_final, dt=excluded.dt, lineage_ref=excluded.lineage_ref
                    """,
                    row,
                )
                copied += 1
            self._conn.commit()
        return copied

    def query_ticker_latest(self, *, venue: str, market_id: str, symbol: str) -> dict[str, Any] | None:
        if self._conn is None:
            return None
        row = self._conn.execute(
            "SELECT price, bid_px, ask_px, bid_qty, ask_qty, ts_recv, lineage_ref FROM ch_ticker_latest WHERE venue_id=? AND market_id=? AND instrument_id=?",
            (venue, market_id, symbol),
        ).fetchone()
        if row is None:
            return None
        return {"price": row[0], "bid_px": row[1], "ask_px": row[2], "bid_qty": row[3], "ask_qty": row[4], "ts_recv": row[5], "raw_refs": json.loads(row[6] or "[]")}

    def query_bba_latest(self, *, venue: str, market_id: str, symbol: str) -> dict[str, Any] | None:
        if self._conn is None:
            return None
        row = self._conn.execute(
            "SELECT bid_px, bid_qty, ask_px, ask_qty, ts_recv, lineage_ref FROM ch_best_bid_ask WHERE venue_id=? AND market_id=? AND instrument_id=?",
            (venue, market_id, symbol),
        ).fetchone()
        if row is None:
            return None
        return {"bid_px": row[0], "bid_qty": row[1], "ask_px": row[2], "ask_qty": row[3], "ts_recv": row[4], "raw_refs": json.loads(row[5] or "[]")}

    def query_ohlcv_range(self, *, venue: str, market_id: str, symbol: str, from_ts: str, to_ts: str, limit: int) -> list[dict[str, Any]]:
        if self._conn is None:
            return []
        rows = self._conn.execute(
            "SELECT ts_bucket, open, high, low, close, volume, is_final, lineage_ref FROM ch_ohlcv_1m WHERE venue_id=? AND market_id=? AND instrument_id=? AND ts_bucket>=? AND ts_bucket<=? ORDER BY ts_bucket ASC LIMIT ?",
            (venue, market_id, symbol, from_ts, to_ts, limit),
        ).fetchall()
        return [{"ts_bucket": r[0], "open": r[1], "high": r[2], "low": r[3], "close": r[4], "volume": r[5], "is_final": bool(r[6]), "raw_refs": json.loads(r[7] or "[]")} for r in rows]


class PostgresOpsLikeStore:
    def __init__(self, dsn: str | None = None):
        self._dsn = dsn or os.getenv("POSTGRES_OPS_DSN") or ""
        self._conn: sqlite3.Connection | None = None
        if self._dsn.startswith("sqlite:///"):
            self._conn = sqlite3.connect(self._dsn.removeprefix("sqlite:///"))
            self.ensure_schema()

    def ensure_schema(self) -> None:
        if self._conn is None:
            return
        ddls = [
            "CREATE TABLE IF NOT EXISTS venues(venue_id TEXT PRIMARY KEY, venue_name TEXT NOT NULL)",
            "CREATE TABLE IF NOT EXISTS accounts(account_id TEXT PRIMARY KEY, label TEXT NOT NULL)",
            "CREATE TABLE IF NOT EXISTS account_venue_binding(account_id TEXT NOT NULL, venue_id TEXT NOT NULL, external_account_ref TEXT, PRIMARY KEY(account_id, venue_id))",
            "CREATE TABLE IF NOT EXISTS balance_snapshots(account_id TEXT NOT NULL, venue_id TEXT NOT NULL, asset TEXT NOT NULL, ts_snapshot TEXT NOT NULL, free_qty REAL NOT NULL, locked_qty REAL NOT NULL, PRIMARY KEY(account_id, venue_id, asset, ts_snapshot))",
            "CREATE TABLE IF NOT EXISTS position_snapshots(account_id TEXT NOT NULL, venue_id TEXT NOT NULL, symbol TEXT NOT NULL, ts_snapshot TEXT NOT NULL, qty REAL NOT NULL, avg_price REAL, PRIMARY KEY(account_id, venue_id, symbol, ts_snapshot))",
            "CREATE TABLE IF NOT EXISTS orders(order_id TEXT PRIMARY KEY, account_id TEXT, venue_id TEXT, symbol TEXT, side TEXT, order_type TEXT, status TEXT, ts_event TEXT)",
            "CREATE TABLE IF NOT EXISTS fills(fill_id TEXT PRIMARY KEY, order_id TEXT, account_id TEXT, venue_id TEXT, symbol TEXT, price REAL, qty REAL, ts_event TEXT)",
        ]
        for ddl in ddls:
            self._conn.execute(ddl)
        self._conn.commit()


class ValkeyHotCache:
    def __init__(self, inmem: HotCache):
        self._cache = inmem

    def get_or_load(self, key: str, loader, *, ttl_seconds: float | None = None):
        return self._cache.get_or_load(key, loader, ttl_seconds=ttl_seconds)

    def invalidate(self, key: str) -> None:
        self._cache.invalidate(key)

    def stats(self) -> dict[str, int]:
        s = self._cache.stats()
        return {"hit": s.hit, "miss": s.miss}


def timed_query(timeout_s: float, fn):
    started = time.perf_counter()
    value = fn()
    elapsed = time.perf_counter() - started
    if elapsed > timeout_s:
        raise TimeoutError("READ_TIMEOUT")
    return value, elapsed
