from __future__ import annotations

import json
import sqlite3
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class RawIngestMeta:
    raw_msg_id: str
    tenant_id: str
    source_type: str | None
    venue_id: str | None
    market_id: str | None
    stream_name: str | None
    endpoint: str | None
    event_ts: str | None
    received_ts: str
    seq: str | None
    source_msg_key: str | None
    payload_hash: str | None
    payload_size: int | None
    object_key: str | None
    quality_json: dict[str, Any]
    parser_version: str | None


class MarketDataMetaRepository:
    def __init__(self, conn: sqlite3.Connection):
        self._conn = conn

    def insert_raw_ingest_meta(self, row: RawIngestMeta) -> None:
        self._conn.execute(
            """
            INSERT INTO raw_ingest_meta (
                raw_msg_id, tenant_id, source_type, venue_id, market_id, stream_name, endpoint,
                event_ts, received_ts, seq, source_msg_key, payload_hash, payload_size, object_key,
                quality_json, parser_version
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                row.raw_msg_id,
                row.tenant_id,
                row.source_type,
                row.venue_id,
                row.market_id,
                row.stream_name,
                row.endpoint,
                row.event_ts,
                row.received_ts,
                row.seq,
                row.source_msg_key,
                row.payload_hash,
                row.payload_size,
                row.object_key,
                json.dumps(row.quality_json, separators=(",", ":"), ensure_ascii=False),
                row.parser_version,
            ),
        )
        self._conn.commit()

    def get_raw_ingest_meta(self, raw_msg_id: str) -> RawIngestMeta | None:
        cursor = self._conn.execute(
            """
            SELECT raw_msg_id, tenant_id, source_type, venue_id, market_id, stream_name, endpoint,
                   event_ts, received_ts, seq, source_msg_key, payload_hash, payload_size, object_key,
                   quality_json, parser_version
            FROM raw_ingest_meta
            WHERE raw_msg_id = ?
            """,
            (raw_msg_id,),
        )
        row = cursor.fetchone()
        if row is None:
            return None

        return RawIngestMeta(
            raw_msg_id=row[0],
            tenant_id=row[1],
            source_type=row[2],
            venue_id=row[3],
            market_id=row[4],
            stream_name=row[5],
            endpoint=row[6],
            event_ts=row[7],
            received_ts=row[8],
            seq=row[9],
            source_msg_key=row[10],
            payload_hash=row[11],
            payload_size=row[12],
            object_key=row[13],
            quality_json=json.loads(row[14] or "{}"),
            parser_version=row[15],
        )


    def count_payload_hash_since(self, payload_hash: str, *, since_ts: str) -> int:
        row = self._conn.execute(
            """
            SELECT COUNT(*)
            FROM raw_ingest_meta
            WHERE payload_hash = ? AND received_ts >= ?
            """,
            (payload_hash, since_ts),
        ).fetchone()
        return int(row[0] if row is not None else 0)

    def exists_source_msg_key_since(
        self,
        *,
        venue_id: str | None,
        market_id: str | None,
        source_msg_key: str,
        since_ts: str,
    ) -> bool:
        row = self._conn.execute(
            """
            SELECT 1
            FROM raw_ingest_meta
            WHERE source_msg_key = ?
              AND COALESCE(venue_id, '') = COALESCE(?, '')
              AND COALESCE(market_id, '') = COALESCE(?, '')
              AND received_ts >= ?
            LIMIT 1
            """,
            (source_msg_key, venue_id, market_id, since_ts),
        ).fetchone()
        return row is not None

    def insert_ws_session(
        self,
        *,
        session_id: str,
        venue_id: str | None,
        market_id: str | None,
        started_at: str,
        ended_at: str | None,
        close_reason: str | None,
        recv_count: int,
        dup_suspect_count: int,
        gap_suspect_count: int,
        lag_stats_json: dict[str, Any],
    ) -> bool:
        cur = self._conn.execute(
            """
            INSERT INTO ws_sessions (
                session_id, venue_id, market_id, started_at, ended_at, close_reason,
                recv_count, dup_suspect_count, gap_suspect_count, lag_stats_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                session_id,
                venue_id,
                market_id,
                started_at,
                ended_at,
                close_reason,
                recv_count,
                dup_suspect_count,
                gap_suspect_count,
                json.dumps(lag_stats_json, separators=(",", ":"), ensure_ascii=False),
            ),
        )
        self._conn.commit()

    def insert_ws_subscription(
        self,
        *,
        session_id: str,
        stream_name: str,
        subscribed_at: str,
        unsubscribed_at: str | None,
        meta_json: dict[str, Any],
    ) -> None:
        self._conn.execute(
            """
            INSERT INTO ws_subscriptions (
                session_id, stream_name, subscribed_at, unsubscribed_at, meta_json
            ) VALUES (?, ?, ?, ?, ?)
            """,
            (
                session_id,
                stream_name,
                subscribed_at,
                unsubscribed_at,
                json.dumps(meta_json, separators=(",", ":"), ensure_ascii=False),
            ),
        )
        self._conn.commit()

    def update_ws_session_end(
        self,
        *,
        session_id: str,
        ended_at: str,
        close_reason: str | None,
        recv_count: int,
        dup_suspect_count: int,
        gap_suspect_count: int,
        lag_stats_json: dict[str, Any],
    ) -> None:
        self._conn.execute(
            """
            UPDATE ws_sessions
            SET ended_at = ?,
                close_reason = ?,
                recv_count = ?,
                dup_suspect_count = ?,
                gap_suspect_count = ?,
                lag_stats_json = ?
            WHERE session_id = ?
            """,
            (
                ended_at,
                close_reason,
                recv_count,
                dup_suspect_count,
                gap_suspect_count,
                json.dumps(lag_stats_json, separators=(",", ":"), ensure_ascii=False),
                session_id,
            ),
        )
        self._conn.commit()

    def update_ws_subscription_end(
        self,
        *,
        session_id: str,
        stream_name: str,
        subscribed_at: str,
        unsubscribed_at: str,
    ) -> None:
        self._conn.execute(
            """
            UPDATE ws_subscriptions
            SET unsubscribed_at = ?
            WHERE session_id = ?
              AND stream_name = ?
              AND subscribed_at = ?
            """,
            (unsubscribed_at, session_id, stream_name, subscribed_at),
        )
        self._conn.commit()

    def insert_md_trade(
        self,
        *,
        raw_msg_id: str,
        venue_id: str | None,
        market_id: str | None,
        instrument_id: str | None,
        source_msg_key: str | None,
        price: float,
        qty: float,
        side: str,
        occurred_at: str,
        received_ts: str,
        extra_json: dict[str, Any],
    ) -> bool:
        cur = self._conn.execute(
            """
            INSERT OR IGNORE INTO md_trades (
                raw_msg_id, venue_id, market_id, instrument_id, source_msg_key,
                price, qty, side, occurred_at, received_ts, extra_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                raw_msg_id,
                venue_id,
                market_id,
                instrument_id,
                source_msg_key,
                price,
                qty,
                side,
                occurred_at,
                received_ts,
                json.dumps(extra_json, separators=(",", ":"), ensure_ascii=False),
            ),
        )
        self._conn.commit()
        return bool(cur.rowcount)

    def insert_md_ohlcv(
        self,
        *,
        raw_msg_id: str,
        venue_id: str | None,
        market_id: str | None,
        instrument_id: str | None,
        timeframe: str,
        open_ts: str,
        open_price: float,
        high: float,
        low: float,
        close: float,
        volume: float,
        is_final: bool,
        extra_json: dict[str, Any],
    ) -> None:
        self._conn.execute(
            """
            INSERT OR IGNORE INTO md_ohlcv (
                raw_msg_id, venue_id, market_id, instrument_id, timeframe,
                open_ts, open, high, low, close, volume, is_final, extra_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                raw_msg_id,
                venue_id,
                market_id,
                instrument_id,
                timeframe,
                open_ts,
                open_price,
                high,
                low,
                close,
                volume,
                1 if is_final else 0,
                json.dumps(extra_json, separators=(",", ":"), ensure_ascii=False),
            ),
        )
        self._conn.commit()

    def insert_md_best_bid_ask(
        self,
        *,
        raw_msg_id: str,
        venue_id: str | None,
        market_id: str | None,
        instrument_id: str | None,
        bid_px: float,
        bid_qty: float,
        ask_px: float,
        ask_qty: float,
        event_ts: str,
        received_ts: str,
        extra_json: dict[str, Any],
    ) -> None:
        self._conn.execute(
            """
            INSERT INTO md_best_bid_ask (
                raw_msg_id, venue_id, market_id, instrument_id,
                bid_px, bid_qty, ask_px, ask_qty, event_ts, received_ts, extra_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                raw_msg_id,
                venue_id,
                market_id,
                instrument_id,
                bid_px,
                bid_qty,
                ask_px,
                ask_qty,
                event_ts,
                received_ts,
                json.dumps(extra_json, separators=(",", ":"), ensure_ascii=False),
            ),
        )
        self._conn.commit()

    def insert_md_events_json(
        self,
        *,
        raw_msg_id: str,
        tenant_id: str,
        event_type: str,
        event_ts: str | None,
        received_ts: str,
        payload_jsonb: dict[str, Any],
        payload_schema_ref: str,
        parser_version: str,
        extra_json: dict[str, Any],
    ) -> None:
        self._conn.execute(
            """
            INSERT INTO md_events_json (
                raw_msg_id, tenant_id, event_type, event_ts, received_ts,
                payload_jsonb, payload_schema_ref, parser_version, extra_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                raw_msg_id,
                tenant_id,
                event_type,
                event_ts,
                received_ts,
                json.dumps(payload_jsonb, separators=(",", ":"), ensure_ascii=False),
                payload_schema_ref,
                parser_version,
                json.dumps(extra_json, separators=(",", ":"), ensure_ascii=False),
            ),
        )
        self._conn.commit()

    def get_latest_best_bid_ask(
        self,
        *,
        venue_id: str,
        market_id: str,
        instrument_id: str,
    ) -> dict[str, Any] | None:
        row = self._conn.execute(
            """
            SELECT bid_px, bid_qty, ask_px, ask_qty, event_ts, received_ts
            FROM md_best_bid_ask
            WHERE venue_id = ? AND market_id = ? AND instrument_id = ?
            ORDER BY received_ts DESC, id DESC
            LIMIT 1
            """,
            (venue_id, market_id, instrument_id),
        ).fetchone()
        if row is None:
            return None
        return {
            "bid_px": row[0],
            "bid_qty": row[1],
            "ask_px": row[2],
            "ask_qty": row[3],
            "event_ts": row[4],
            "received_ts": row[5],
        }

    def get_latest_trade(
        self,
        *,
        venue_id: str,
        market_id: str,
        instrument_id: str,
    ) -> dict[str, Any] | None:
        row = self._conn.execute(
            """
            SELECT price, qty, side, occurred_at, received_ts
            FROM md_trades
            WHERE venue_id = ? AND market_id = ? AND instrument_id = ?
            ORDER BY received_ts DESC, id DESC
            LIMIT 1
            """,
            (venue_id, market_id, instrument_id),
        ).fetchone()
        if row is None:
            return None
        return {
            "price": row[0],
            "qty": row[1],
            "side": row[2],
            "occurred_at": row[3],
            "received_ts": row[4],
        }

    def get_latest_ohlcv(
        self,
        *,
        venue_id: str,
        market_id: str,
        instrument_id: str,
        timeframe: str,
    ) -> dict[str, Any] | None:
        row = self._conn.execute(
            """
            SELECT open_ts, open, high, low, close, volume, is_final
            FROM md_ohlcv
            WHERE venue_id = ? AND market_id = ? AND instrument_id = ? AND timeframe = ?
            ORDER BY open_ts DESC, id DESC
            LIMIT 1
            """,
            (venue_id, market_id, instrument_id, timeframe),
        ).fetchone()
        if row is None:
            return None
        return {
            "open_ts": row[0],
            "open": row[1],
            "high": row[2],
            "low": row[3],
            "close": row[4],
            "volume": row[5],
            "is_final": bool(row[6]),
            "received_ts": row[0],
        }

    def upsert_orderbook_state(
        self,
        *,
        venue_id: str,
        market_id: str,
        bid_px: float | None,
        bid_qty: float | None,
        ask_px: float | None,
        ask_qty: float | None,
        as_of: str | None,
        last_update_ts: str,
        last_seq: str | None,
        degraded: bool,
        reason: str | None,
    ) -> None:
        self._conn.execute(
            """
            INSERT INTO md_orderbook_state (
                venue_id, market_id, bid_px, bid_qty, ask_px, ask_qty,
                as_of, last_update_ts, last_seq, degraded, reason
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(venue_id, market_id) DO UPDATE SET
                bid_px = excluded.bid_px,
                bid_qty = excluded.bid_qty,
                ask_px = excluded.ask_px,
                ask_qty = excluded.ask_qty,
                as_of = excluded.as_of,
                last_update_ts = excluded.last_update_ts,
                last_seq = excluded.last_seq,
                degraded = excluded.degraded,
                reason = excluded.reason
            """,
            (
                venue_id,
                market_id,
                bid_px,
                bid_qty,
                ask_px,
                ask_qty,
                as_of,
                last_update_ts,
                last_seq,
                1 if degraded else 0,
                reason,
            ),
        )
        self._conn.commit()

    def get_orderbook_state(self, *, venue_id: str, market_id: str) -> dict[str, Any] | None:
        row = self._conn.execute(
            """
            SELECT bid_px, bid_qty, ask_px, ask_qty, as_of, last_update_ts, last_seq, degraded, reason
            FROM md_orderbook_state
            WHERE venue_id = ? AND market_id = ?
            """,
            (venue_id, market_id),
        ).fetchone()
        if row is None:
            return None
        return {
            "bid_px": row[0],
            "bid_qty": row[1],
            "ask_px": row[2],
            "ask_qty": row[3],
            "as_of": row[4],
            "last_update_ts": row[5],
            "last_seq": row[6],
            "degraded": bool(row[7]),
            "reason": row[8],
        }
