from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from services.marketdata.app.db.repository import MarketDataMetaRepository


@dataclass(frozen=True)
class NormalizeResult:
    target: str
    event_type: str | None


def _as_float(value: object) -> float:
    return float(value)


def _insert_trade(repo: MarketDataMetaRepository, envelope: dict[str, Any], payload: dict[str, Any]) -> None:
    repo.insert_md_trade(
        raw_msg_id=str(envelope["raw_msg_id"]),
        venue_id=None if envelope.get("venue_id") is None else str(envelope.get("venue_id")),
        market_id=None if envelope.get("market_id") is None else str(envelope.get("market_id")),
        instrument_id=None if envelope.get("instrument_id") is None else str(envelope.get("instrument_id")),
        source_msg_key=None if envelope.get("source_msg_key") is None else str(envelope.get("source_msg_key")),
        price=_as_float(payload["price"]),
        qty=_as_float(payload["qty"]),
        side=str(payload["side"]).lower(),
        occurred_at=str(envelope.get("event_ts") or envelope["received_ts"]),
        received_ts=str(envelope["received_ts"]),
        extra_json={"payload": payload},
    )


def _insert_ohlcv(repo: MarketDataMetaRepository, envelope: dict[str, Any], payload: dict[str, Any]) -> None:
    repo.insert_md_ohlcv(
        raw_msg_id=str(envelope["raw_msg_id"]),
        venue_id=None if envelope.get("venue_id") is None else str(envelope.get("venue_id")),
        market_id=None if envelope.get("market_id") is None else str(envelope.get("market_id")),
        instrument_id=None if envelope.get("instrument_id") is None else str(envelope.get("instrument_id")),
        timeframe=str(payload["timeframe"]),
        open_ts=str(payload["open_ts"]),
        open_price=_as_float(payload["open"]),
        high=_as_float(payload["high"]),
        low=_as_float(payload["low"]),
        close=_as_float(payload["close"]),
        volume=_as_float(payload["volume"]),
        is_final=bool(payload["is_final"]),
        extra_json={"payload": payload},
    )


def _insert_bba(repo: MarketDataMetaRepository, envelope: dict[str, Any], payload: dict[str, Any]) -> None:
    repo.insert_md_best_bid_ask(
        raw_msg_id=str(envelope["raw_msg_id"]),
        venue_id=None if envelope.get("venue_id") is None else str(envelope.get("venue_id")),
        market_id=None if envelope.get("market_id") is None else str(envelope.get("market_id")),
        instrument_id=None if envelope.get("instrument_id") is None else str(envelope.get("instrument_id")),
        bid_px=_as_float(payload["bid_px"]),
        bid_qty=_as_float(payload["bid_qty"]),
        ask_px=_as_float(payload["ask_px"]),
        ask_qty=_as_float(payload["ask_qty"]),
        event_ts=str(envelope.get("event_ts") or envelope["received_ts"]),
        received_ts=str(envelope["received_ts"]),
        extra_json={"payload": payload},
    )


def _fallback_event_type(envelope: dict[str, Any]) -> str:
    venue = str(envelope.get("venue_id") or "unknown").lower()
    source_type = str(envelope.get("source_type") or "other").upper()
    if source_type.startswith("WS"):
        return f"venue.ws_message.{venue}"
    if source_type == "NEWS":
        return "news.headline"
    if source_type == "ONCHAIN":
        return "onchain.event"
    return f"ref.raw.{venue}"


def classify_envelope(envelope: dict[str, Any]) -> NormalizeResult:
    payload = envelope.get("payload_json")
    if not isinstance(payload, dict):
        payload = {}

    if {"price", "qty", "side"}.issubset(payload.keys()):
        return NormalizeResult(target="md_trades", event_type=None)

    if {"timeframe", "open_ts", "open", "high", "low", "close", "volume", "is_final"}.issubset(payload.keys()):
        return NormalizeResult(target="md_ohlcv", event_type=None)

    if {"bid_px", "bid_qty", "ask_px", "ask_qty"}.issubset(payload.keys()):
        return NormalizeResult(target="md_best_bid_ask", event_type=None)

    return NormalizeResult(target="md_events_json", event_type=_fallback_event_type(envelope))


def normalize_envelope(repo: MarketDataMetaRepository, envelope: dict[str, Any]) -> NormalizeResult:
    payload = envelope.get("payload_json")
    if not isinstance(payload, dict):
        payload = {}

    classified = classify_envelope(envelope)
    if classified.target == "md_trades":
        _insert_trade(repo, envelope, payload)
        return classified

    if classified.target == "md_ohlcv":
        _insert_ohlcv(repo, envelope, payload)
        return classified

    if classified.target == "md_best_bid_ask":
        _insert_bba(repo, envelope, payload)
        return classified

    event_type = classified.event_type or _fallback_event_type(envelope)
    repo.insert_md_events_json(
        raw_msg_id=str(envelope["raw_msg_id"]),
        tenant_id=str(envelope["tenant_id"]),
        event_type=event_type,
        event_ts=None if envelope.get("event_ts") is None else str(envelope.get("event_ts")),
        received_ts=str(envelope["received_ts"]),
        payload_jsonb=payload,
        payload_schema_ref="contracts/schemas/marketdata/md_events_json.schema.json",
        parser_version=str(envelope.get("parser_version") or "v0.1"),
        extra_json={"source_type": envelope.get("source_type")},
    )
    return classified
