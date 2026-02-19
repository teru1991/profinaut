from __future__ import annotations

import json
import sqlite3
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any


@dataclass(frozen=True)
class GoldMaterializeResult:
    ticker_latest_rows: int
    bba_rows: int
    ohlcv_rows: int


def _dt(ts: str | None) -> str:
    if not ts:
        return datetime.now(UTC).date().isoformat()
    return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC).date().isoformat()


def _bucket_1m(ts: str) -> str:
    dt = datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC).replace(second=0, microsecond=0)
    return dt.isoformat().replace("+00:00", "Z")


def materialize_gold(conn: sqlite3.Connection, *, watermark_ts: str | None = None) -> GoldMaterializeResult:
    conn.execute("PRAGMA foreign_keys = ON")

    bba_rows = conn.execute(
        """
        SELECT venue_id, market_id, instrument_id, bid_px, bid_qty, ask_px, ask_qty,
               COALESCE(event_ts, received_ts) AS ts_event, received_ts, raw_msg_id
        FROM md_best_bid_ask
        WHERE (? IS NULL OR received_ts <= ?)
        ORDER BY received_ts DESC, id DESC
        """,
        (watermark_ts, watermark_ts),
    ).fetchall()
    seen: set[tuple[str, str, str]] = set()
    inserted_bba = 0
    inserted_ticker = 0
    for row in bba_rows:
        venue, market, instrument = row[0], row[1], row[2]
        if not venue or not market or not instrument:
            continue
        key = (venue, market, instrument)
        if key in seen:
            continue
        seen.add(key)
        lineage = json.dumps([row[9]])
        conn.execute(
            """
            INSERT INTO gold_best_bid_ask(
                venue_id, market_id, instrument_id, bid_px, bid_qty, ask_px, ask_qty,
                ts_event, ts_recv, dt, raw_refs
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(venue_id, market_id, instrument_id) DO UPDATE SET
                bid_px=excluded.bid_px,
                bid_qty=excluded.bid_qty,
                ask_px=excluded.ask_px,
                ask_qty=excluded.ask_qty,
                ts_event=excluded.ts_event,
                ts_recv=excluded.ts_recv,
                dt=excluded.dt,
                raw_refs=excluded.raw_refs
            """,
            (venue, market, instrument, row[3], row[4], row[5], row[6], row[7], row[8], _dt(row[8]), lineage),
        )
        inserted_bba += 1
        conn.execute(
            """
            INSERT INTO gold_ticker_latest(
                venue_id, market_id, instrument_id, price, bid_px, ask_px, bid_qty, ask_qty,
                ts_event, ts_recv, dt, raw_refs
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(venue_id, market_id, instrument_id) DO UPDATE SET
                price=excluded.price,
                bid_px=excluded.bid_px,
                ask_px=excluded.ask_px,
                bid_qty=excluded.bid_qty,
                ask_qty=excluded.ask_qty,
                ts_event=excluded.ts_event,
                ts_recv=excluded.ts_recv,
                dt=excluded.dt,
                raw_refs=excluded.raw_refs
            """,
            (venue, market, instrument, (float(row[3]) + float(row[5])) / 2, row[3], row[5], row[4], row[6], row[7], row[8], _dt(row[8]), lineage),
        )
        inserted_ticker += 1

    trade_rows = conn.execute(
        """
        SELECT venue_id, market_id, instrument_id, price, occurred_at, received_ts, raw_msg_id
        FROM md_trades
        WHERE (? IS NULL OR received_ts <= ?)
        ORDER BY received_ts DESC, id DESC
        """,
        (watermark_ts, watermark_ts),
    ).fetchall()
    for row in trade_rows:
        venue, market, instrument = row[0], row[1], row[2]
        if not venue or not market or not instrument:
            continue
        key = (venue, market, instrument)
        if key in seen:
            continue
        seen.add(key)
        lineage = json.dumps([row[6]])
        conn.execute(
            """
            INSERT INTO gold_ticker_latest(
                venue_id, market_id, instrument_id, price, bid_px, ask_px, bid_qty, ask_qty,
                ts_event, ts_recv, dt, raw_refs
            ) VALUES (?, ?, ?, ?, NULL, NULL, NULL, NULL, ?, ?, ?, ?)
            ON CONFLICT(venue_id, market_id, instrument_id) DO UPDATE SET
                price=excluded.price,
                bid_px=excluded.bid_px,
                ask_px=excluded.ask_px,
                bid_qty=excluded.bid_qty,
                ask_qty=excluded.ask_qty,
                ts_event=excluded.ts_event,
                ts_recv=excluded.ts_recv,
                dt=excluded.dt,
                raw_refs=excluded.raw_refs
            """,
            (venue, market, instrument, row[3], row[4], row[5], _dt(row[5]), lineage),
        )
        inserted_ticker += 1

    ohlcv_rows = conn.execute(
        """
        SELECT venue_id, market_id, instrument_id, timeframe, open_ts, open, high, low, close, volume, is_final, raw_msg_id
        FROM md_ohlcv
        WHERE timeframe = '1m'
          AND (? IS NULL OR open_ts <= ?)
        ORDER BY open_ts ASC, id ASC
        """,
        (watermark_ts, watermark_ts),
    ).fetchall()
    inserted_ohlcv = 0
    for row in ohlcv_rows:
        if not row[0] or not row[1] or not row[2]:
            continue
        conn.execute(
            """
            INSERT INTO gold_ohlcv_1m(
                venue_id, market_id, instrument_id, ts_bucket, open, high, low, close, volume,
                is_final, dt, raw_refs
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(venue_id, market_id, instrument_id, ts_bucket) DO UPDATE SET
                open=excluded.open,
                high=excluded.high,
                low=excluded.low,
                close=excluded.close,
                volume=excluded.volume,
                is_final=excluded.is_final,
                dt=excluded.dt,
                raw_refs=excluded.raw_refs
            """,
            (row[0], row[1], row[2], _bucket_1m(row[4]), row[5], row[6], row[7], row[8], row[9], row[10], _dt(row[4]), json.dumps([row[11]])),
        )
        inserted_ohlcv += 1

    conn.commit()
    return GoldMaterializeResult(ticker_latest_rows=inserted_ticker, bba_rows=inserted_bba, ohlcv_rows=inserted_ohlcv)
