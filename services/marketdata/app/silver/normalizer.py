from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime
import os
from typing import Any

from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.silver.orderbook import OrderbookEngine

ORDERBOOK_GAP = "ORDERBOOK_GAP"
ORDERBOOK_RESYNC_FAILED = "ORDERBOOK_RESYNC_FAILED"
ORDERBOOK_SEQ_MISSING = "ORDERBOOK_SEQ_MISSING"


@dataclass(frozen=True)
class NormalizeResult:
    target: str
    event_type: str | None


_ORDERBOOK_ENGINES: dict[tuple[str, str], OrderbookEngine] = {}
_ORDERBOOK_LAST_SEQ: dict[tuple[str, str], int | None] = {}
_ORDERBOOK_DEGRADED_UNTIL_SNAPSHOT: dict[tuple[str, str], bool] = {}


def _as_float(value: object) -> float:
    return float(value)


def _utc_now() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _parse_seq(value: object) -> int | None:
    if value is None:
        return None
    try:
        return int(str(value))
    except ValueError:
        return None




def _parse_rfc3339(ts: str) -> datetime | None:
    try:
        return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC)
    except ValueError:
        return None


def _is_warm_state_too_old(last_update_ts: str | None, current_received_ts: str) -> bool:
    if not last_update_ts:
        return False
    max_age = float(os.getenv("ORDERBOOK_WARM_MAX_AGE_SECONDS", "300"))
    prev = _parse_rfc3339(last_update_ts)
    now = _parse_rfc3339(current_received_ts)
    if prev is None or now is None:
        return False
    return (now - prev).total_seconds() > max_age


def _init_orderbook_runtime_state(repo: MarketDataMetaRepository, *, venue_id: str, market_id: str, received_ts: str) -> None:
    key = (venue_id, market_id)
    if key in _ORDERBOOK_ENGINES:
        return

    engine = OrderbookEngine()
    state = repo.get_orderbook_state(venue_id=venue_id, market_id=market_id)
    if state is not None:
        engine.load_from_bbo(
            bid_px=None if state.get("bid_px") is None else float(state["bid_px"]),
            bid_qty=None if state.get("bid_qty") is None else float(state["bid_qty"]),
            ask_px=None if state.get("ask_px") is None else float(state["ask_px"]),
            ask_qty=None if state.get("ask_qty") is None else float(state["ask_qty"]),
        )
        _ORDERBOOK_LAST_SEQ[key] = _parse_seq(state.get("last_seq"))
        stale_from_restart = _is_warm_state_too_old(
            None if state.get("last_update_ts") is None else str(state["last_update_ts"]),
            received_ts,
        )
        _ORDERBOOK_DEGRADED_UNTIL_SNAPSHOT[key] = stale_from_restart
    else:
        _ORDERBOOK_LAST_SEQ[key] = None
        _ORDERBOOK_DEGRADED_UNTIL_SNAPSHOT[key] = False
    _ORDERBOOK_ENGINES[key] = engine

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


def _insert_event_hub(
    repo: MarketDataMetaRepository,
    envelope: dict[str, Any],
    payload: dict[str, Any],
    *,
    event_type: str,
    schema_ref: str,
    extra_json: dict[str, Any],
) -> None:
    repo.insert_md_events_json(
        raw_msg_id=str(envelope["raw_msg_id"]),
        tenant_id=str(envelope["tenant_id"]),
        event_type=event_type,
        event_ts=None if envelope.get("event_ts") is None else str(envelope.get("event_ts")),
        received_ts=str(envelope["received_ts"]),
        payload_jsonb=payload,
        payload_schema_ref=schema_ref,
        parser_version=str(envelope.get("parser_version") or "v0.1"),
        extra_json=extra_json,
    )


def _normalize_orderbook(repo: MarketDataMetaRepository, envelope: dict[str, Any], payload: dict[str, Any]) -> NormalizeResult:
    venue_id = str(envelope.get("venue_id") or "gmo")
    market_id = str(envelope.get("market_id") or payload.get("symbol") or "spot")
    key = (venue_id, market_id)
    event_ts = str(envelope.get("event_ts") or envelope["received_ts"])
    seq = _parse_seq(envelope.get("seq") or payload.get("sequence") or payload.get("seq"))

    _init_orderbook_runtime_state(repo, venue_id=venue_id, market_id=market_id, received_ts=str(envelope["received_ts"]))
    engine = _ORDERBOOK_ENGINES.setdefault(key, OrderbookEngine())
    prev_seq = _ORDERBOOK_LAST_SEQ.get(key)

    if seq is None:
        repo.upsert_orderbook_state(
            venue_id=venue_id,
            market_id=market_id,
            bid_px=None,
            bid_qty=None,
            ask_px=None,
            ask_qty=None,
            as_of=event_ts,
            last_update_ts=str(envelope["received_ts"]),
            last_seq=None,
            degraded=True,
            reason=ORDERBOOK_SEQ_MISSING,
        )

    event_type = str(payload.get("type") or "snapshot").lower()
    is_snapshot = event_type == "snapshot" or ("changes" not in payload and "asks" in payload and "bids" in payload)

    if seq is not None and OrderbookEngine.check_gap(prev_seq, seq):
        gap_payload = {
            "raw_msg_id": str(envelope["raw_msg_id"]),
            "venue_id": venue_id,
            "market_id": market_id,
            "received_ts": str(envelope["received_ts"]),
            "event_ts": envelope.get("event_ts"),
            "prev_seq": prev_seq,
            "next_seq": seq,
            "seq": seq,
            "extra_json": {"fallback_event_ts": event_ts},
        }
        _insert_event_hub(
            repo,
            envelope,
            gap_payload,
            event_type="md_orderbook_gap",
            schema_ref="contracts/schemas/marketdata/md_orderbook_gap.schema.json",
            extra_json={"reason": ORDERBOOK_GAP},
        )
        repo.upsert_orderbook_state(
            venue_id=venue_id,
            market_id=market_id,
            bid_px=None,
            bid_qty=None,
            ask_px=None,
            ask_qty=None,
            as_of=event_ts,
            last_update_ts=str(envelope["received_ts"]),
            last_seq=str(seq),
            degraded=True,
            reason=ORDERBOOK_GAP,
        )
        _ORDERBOOK_LAST_SEQ[key] = seq
        return NormalizeResult(target="md_events_json", event_type="md_orderbook_gap")

    if is_snapshot:
        event_name = "md_orderbook_snapshot"
        schema_ref = "contracts/schemas/marketdata/md_orderbook_snapshot.schema.json"
        engine.apply_snapshot(payload)
        _ORDERBOOK_DEGRADED_UNTIL_SNAPSHOT[key] = False
    else:
        event_name = "md_orderbook_delta"
        schema_ref = "contracts/schemas/marketdata/md_orderbook_delta.schema.json"
        delta_payload = payload.get("changes") if isinstance(payload.get("changes"), dict) else payload
        engine.apply_delta(delta_payload)

    best_bid, best_ask = engine.derive_bbo()
    if best_bid is not None and best_ask is not None:
        repo.insert_md_best_bid_ask(
            raw_msg_id=str(envelope["raw_msg_id"]),
            venue_id=venue_id,
            market_id=market_id,
            instrument_id=str(envelope.get("instrument_id") or market_id),
            bid_px=best_bid.price,
            bid_qty=best_bid.size,
            ask_px=best_ask.price,
            ask_qty=best_ask.size,
            event_ts=event_ts,
            received_ts=str(envelope["received_ts"]),
            extra_json={"source": "orderbook"},
        )

    _insert_event_hub(
        repo,
        envelope,
        {
            "raw_msg_id": str(envelope["raw_msg_id"]),
            "venue_id": venue_id,
            "market_id": market_id,
            "received_ts": str(envelope["received_ts"]),
            "event_ts": envelope.get("event_ts"),
            "seq": seq,
            "levels": {
                "bids": payload.get("bids") or [],
                "asks": payload.get("asks") or [],
            },
            "extra_json": {"fallback_event_ts": event_ts},
        },
        event_type=event_name,
        schema_ref=schema_ref,
        extra_json={"source_type": envelope.get("source_type")},
    )

    degraded_until_snapshot = _ORDERBOOK_DEGRADED_UNTIL_SNAPSHOT.get(key, False)
    degraded = (seq is None) or degraded_until_snapshot
    reason = ORDERBOOK_SEQ_MISSING if seq is None else ("ORDERBOOK_STATE_STALE" if degraded_until_snapshot else None)

    repo.upsert_orderbook_state(
        venue_id=venue_id,
        market_id=market_id,
        bid_px=None if best_bid is None else best_bid.price,
        bid_qty=None if best_bid is None else best_bid.size,
        ask_px=None if best_ask is None else best_ask.price,
        ask_qty=None if best_ask is None else best_ask.size,
        as_of=event_ts,
        last_update_ts=str(envelope["received_ts"]),
        last_seq=None if seq is None else str(seq),
        degraded=degraded,
        reason=reason,
    )
    if seq is not None:
        _ORDERBOOK_LAST_SEQ[key] = seq
    return NormalizeResult(target="md_events_json", event_type=event_name)


def classify_envelope(envelope: dict[str, Any]) -> NormalizeResult:
    payload = envelope.get("payload_json")
    if not isinstance(payload, dict):
        payload = {}

    stream_name = str(envelope.get("stream_name") or payload.get("channel") or "").lower()
    if stream_name in {"orderbook", "orderbooks"} or "bids" in payload or "asks" in payload:
        return NormalizeResult(target="md_orderbook", event_type="md_orderbook_snapshot")

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

    if classified.target == "md_orderbook":
        return _normalize_orderbook(repo, envelope, payload)

    event_type = classified.event_type or _fallback_event_type(envelope)
    _insert_event_hub(
        repo,
        envelope,
        payload,
        event_type=event_type,
        schema_ref="contracts/schemas/marketdata/md_events_json.schema.json",
        extra_json={"source_type": envelope.get("source_type")},
    )
    return classified
