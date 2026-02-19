from __future__ import annotations

from datetime import UTC, datetime
from typing import Any

from services.marketdata.app.ucel_core import (
    Envelope,
    Meta,
    OrderBookDelta,
    OrderBookLevel,
    OrderBookSnapshot,
    Quality,
    TickerSnapshot,
    TradeEvent,
)


def parse_ts(value: str | None) -> datetime:
    if not value:
        return datetime.now(UTC)
    return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(UTC)


def _quality(*, missing_fields: bool = False) -> Quality:
    return Quality(is_partial=missing_fields)


def make_meta(*, symbol: str, ts_event: datetime, trace_id: str | None, request_id: str | None, run_id: str | None, missing_fields: bool = False) -> Meta:
    return Meta(
        venue="gmocoin",
        symbol=symbol,
        venue_symbol=symbol,
        ts_event=ts_event,
        ts_recv=datetime.now(UTC),
        trace_id=trace_id,
        request_id=request_id,
        run_id=run_id,
        quality=_quality(missing_fields=missing_fields),
    )


def to_ticker(*, symbol: str, item: dict[str, Any], trace_id: str | None, request_id: str | None, run_id: str | None) -> Envelope[TickerSnapshot]:
    missing = any(item.get(k) is None for k in ("bid", "ask"))
    ts_event = parse_ts(item.get("timestamp"))
    return Envelope(
        meta=make_meta(symbol=symbol, ts_event=ts_event, trace_id=trace_id, request_id=request_id, run_id=run_id, missing_fields=missing),
        payload=TickerSnapshot(
            bid=float(item["bid"]) if item.get("bid") is not None else None,
            ask=float(item["ask"]) if item.get("ask") is not None else None,
            last=float(item["last"]) if item.get("last") is not None else (float(item["price"]) if item.get("price") is not None else None),
        ),
    )


def to_trades(*, symbol: str, rows: list[dict[str, Any]], trace_id: str | None, request_id: str | None, run_id: str | None) -> Envelope[tuple[TradeEvent, ...]]:
    events: list[TradeEvent] = []
    ts = datetime.now(UTC)
    for idx, row in enumerate(rows):
        ts = parse_ts(row.get("timestamp") or row.get("executed_at"))
        events.append(
            TradeEvent(
                trade_id=str(row.get("id") or idx),
                side=str(row.get("side") or "unknown").lower(),
                price=float(row["price"]),
                amount=float(row.get("size") or row.get("amount") or 0),
                ts_event=ts,
            )
        )
    return Envelope(
        meta=make_meta(symbol=symbol, ts_event=ts, trace_id=trace_id, request_id=request_id, run_id=run_id, missing_fields=not events),
        payload=tuple(events),
    )


def _levels(rows: list[dict[str, Any]]) -> tuple[OrderBookLevel, ...]:
    return tuple(OrderBookLevel(price=float(x["price"]), amount=float(x.get("size") or x.get("amount") or 0)) for x in rows)


def to_snapshot(*, symbol: str, item: dict[str, Any], depth: int, trace_id: str | None, request_id: str | None, run_id: str | None) -> Envelope[OrderBookSnapshot]:
    ts_event = parse_ts(item.get("timestamp"))
    return Envelope(
        meta=make_meta(symbol=symbol, ts_event=ts_event, trace_id=trace_id, request_id=request_id, run_id=run_id),
        payload=OrderBookSnapshot(
            bids=_levels((item.get("bids") or [])[:depth]),
            asks=_levels((item.get("asks") or [])[:depth]),
            sequence=int(item["sequence"]) if item.get("sequence") is not None else None,
        ),
    )


def to_delta(*, symbol: str, payload: dict[str, Any], trace_id: str | None, request_id: str | None, run_id: str | None) -> Envelope[OrderBookDelta]:
    body = payload.get("changes") if isinstance(payload.get("changes"), dict) else payload
    ts_event = parse_ts(payload.get("timestamp"))
    return Envelope(
        meta=make_meta(symbol=symbol, ts_event=ts_event, trace_id=trace_id, request_id=request_id, run_id=run_id),
        payload=OrderBookDelta(
            bids=_levels(body.get("bids") or []),
            asks=_levels(body.get("asks") or []),
            sequence=int(payload["sequence"]) if payload.get("sequence") is not None else None,
        ),
    )
