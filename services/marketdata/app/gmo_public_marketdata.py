from __future__ import annotations

import asyncio
import json
import logging
import urllib.parse
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any, AsyncContextManager, Awaitable, Callable

from services.marketdata.app.registry import RegistryConnection, load_venue_registry
from services.marketdata.app.transport import HttpTransportClient
from services.marketdata.app.ucel_core import (
    Capabilities,
    CoreError,
    Envelope,
    ErrorCode,
    Exchange,
    ExecuteContext,
    Meta,
    OpName,
    OrderBookDelta,
    OrderBookLevel,
    OrderBookSnapshot,
    Quality,
    TickerSnapshot,
    TradeEvent,
)

logger = logging.getLogger("marketdata.gmo_public")


@dataclass
class GmoPublicMetrics:
    ws_reconnect_total: int = 0
    orderbook_resync_total: int = 0
    parse_failures: int = 0
    stale_total: int = 0


@dataclass
class _OrderbookState:
    last_sequence: int | None = None
    degraded: bool = False


class GmoPublicMarketDataAdapter(Exchange):
    def __init__(
        self,
        *,
        symbol: str = "BTC_JPY",
        timeout_seconds: float = 5.0,
        http_client: HttpTransportClient | None = None,
        request_fn: Callable[[str, str], dict[str, Any]] | None = None,
    ) -> None:
        self._symbol = symbol
        self._timeout_seconds = timeout_seconds
        self._registry = load_venue_registry("gmocoin")
        self._http = http_client or HttpTransportClient()
        self._request_fn = request_fn
        self._metrics = GmoPublicMetrics()
        self._orderbook_state = _OrderbookState()

    @property
    def metrics(self) -> GmoPublicMetrics:
        return self._metrics

    def capabilities(self) -> Capabilities:
        supported = {conn.op for conn in self._registry.connections if conn.supported and conn.op is not None}
        return Capabilities(venue="gmocoin", supported_ops=frozenset(supported))

    def _execute_impl(self, op: OpName, params: dict[str, Any], ctx: ExecuteContext) -> Any:
        if op == OpName.FETCH_TICKER:
            return self._fetch_ticker(symbol=str(params["symbol"]), ctx=ctx)
        if op == OpName.FETCH_TRADES:
            return self._fetch_trades(symbol=str(params["symbol"]), limit=int(params["limit"]), ctx=ctx)
        if op == OpName.FETCH_ORDERBOOK_SNAPSHOT:
            return self._fetch_orderbook_snapshot(symbol=str(params["symbol"]), depth=int(params["depth"]), ctx=ctx)
        raise CoreError(ErrorCode.NOT_SUPPORTED, f"Unsupported op={op.value}")

    def _conn_for(self, op: OpName, source: str) -> RegistryConnection:
        for conn in self._registry.connections:
            if conn.source == source and conn.op == op and conn.supported:
                return conn
        raise CoreError(ErrorCode.NOT_SUPPORTED, f"No registry connection for op={op.value} source={source}")

    def _request_json(self, op: OpName, query: dict[str, str]) -> dict[str, Any]:
        conn = self._conn_for(op, "rest")
        if self._request_fn is not None:
            return self._request_fn(conn.connection_id, urllib.parse.urlencode(query))

        endpoint = next(
            record
            for record in self._load_catalog()["rest_endpoints"]
            if record.get("id") == conn.connection_id
        )
        url = f"{str(endpoint['base_url']).rstrip('/')}{endpoint['path']}?{urllib.parse.urlencode(query)}"
        raw = self._http.request(op_name=op.value, method="GET", url=url, timeout_seconds=self._timeout_seconds)
        return json.loads(raw.decode("utf-8"))

    def _load_catalog(self) -> dict[str, Any]:
        import json as _json
        from pathlib import Path

        catalog = Path(self._registry.catalog_path)
        return _json.loads(catalog.read_text(encoding="utf-8"))

    @staticmethod
    def _parse_dt(value: str | None) -> datetime:
        if not value:
            return datetime.now(UTC)
        return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(UTC)

    def _meta(self, *, symbol: str, ts_event: datetime, ctx: ExecuteContext, is_partial: bool = False) -> Meta:
        return Meta(
            venue="gmocoin",
            symbol=symbol,
            venue_symbol=symbol,
            ts_event=ts_event,
            ts_recv=datetime.now(UTC),
            trace_id=ctx.trace_id,
            request_id=ctx.request_id,
            run_id=ctx.run_id,
            quality=Quality(is_partial=is_partial),
        )

    def _fetch_ticker(self, *, symbol: str, ctx: ExecuteContext) -> Envelope[TickerSnapshot]:
        payload = self._request_json(OpName.FETCH_TICKER, {"symbol": symbol})
        item = (payload.get("data") or [{}])[0]
        ts_event = self._parse_dt(item.get("timestamp"))
        return Envelope(
            meta=self._meta(symbol=symbol, ts_event=ts_event, ctx=ctx),
            payload=TickerSnapshot(
                bid=float(item.get("bid")) if item.get("bid") is not None else None,
                ask=float(item.get("ask")) if item.get("ask") is not None else None,
                last=float(item.get("last")) if item.get("last") is not None else None,
            ),
        )

    def _fetch_trades(self, *, symbol: str, limit: int, ctx: ExecuteContext) -> Envelope[tuple[TradeEvent, ...]]:
        payload = self._request_json(OpName.FETCH_TRADES, {"symbol": symbol, "count": str(limit)})
        trades: list[TradeEvent] = []
        for item in payload.get("data") or payload.get("list") or []:
            ts_event = self._parse_dt(item.get("timestamp") or item.get("executed_at"))
            trades.append(
                TradeEvent(
                    trade_id=str(item.get("id") or item.get("timestamp") or len(trades)),
                    side=str(item.get("side") or "unknown").lower(),
                    price=float(item.get("price")),
                    amount=float(item.get("size") or item.get("amount")),
                    ts_event=ts_event,
                )
            )
        event_ts = trades[0].ts_event if trades else datetime.now(UTC)
        return Envelope(meta=self._meta(symbol=symbol, ts_event=event_ts, ctx=ctx), payload=tuple(trades))

    def _fetch_orderbook_snapshot(self, *, symbol: str, depth: int, ctx: ExecuteContext) -> Envelope[OrderBookSnapshot]:
        payload = self._request_json(OpName.FETCH_ORDERBOOK_SNAPSHOT, {"symbol": symbol})
        item = (payload.get("data") or [{}])[0] if isinstance(payload.get("data"), list) else payload.get("data", payload)
        ts_event = self._parse_dt(item.get("timestamp"))
        bids = tuple(OrderBookLevel(price=float(row["price"]), amount=float(row["size"])) for row in item.get("bids", [])[:depth])
        asks = tuple(OrderBookLevel(price=float(row["price"]), amount=float(row["size"])) for row in item.get("asks", [])[:depth])
        sequence = int(item["sequence"]) if item.get("sequence") is not None else None
        return Envelope(
            meta=self._meta(symbol=symbol, ts_event=ts_event, ctx=ctx),
            payload=OrderBookSnapshot(bids=bids, asks=asks, sequence=sequence),
        )

    async def process_orderbook_delta(self, payload: dict[str, Any], ctx: ExecuteContext) -> Envelope[OrderBookDelta] | None:
        try:
            sequence = int(payload.get("sequence"))
        except (TypeError, ValueError):
            self._metrics.parse_failures += 1
            return None

        expected = self._orderbook_state.last_sequence + 1 if self._orderbook_state.last_sequence is not None else None
        if expected is not None and sequence != expected:
            self._orderbook_state.degraded = True
            self._metrics.stale_total += 1
            logger.warning(
                "orderbook_gap_detected venue=gmocoin symbol=%s op=subscribe_orderbook prev_seq=%s next_seq=%s",
                self._symbol,
                self._orderbook_state.last_sequence,
                sequence,
            )
            self._metrics.orderbook_resync_total += 1
            snapshot = self.fetch_orderbook_snapshot(symbol=self._symbol, depth=50, ctx=ctx)
            self._orderbook_state.last_sequence = snapshot.payload.sequence
            self._orderbook_state.degraded = False

        self._orderbook_state.last_sequence = sequence
        delta = payload.get("changes") if isinstance(payload.get("changes"), dict) else payload
        bids = tuple(OrderBookLevel(price=float(row["price"]), amount=float(row["size"])) for row in delta.get("bids", []))
        asks = tuple(OrderBookLevel(price=float(row["price"]), amount=float(row["size"])) for row in delta.get("asks", []))
        return Envelope(
            meta=self._meta(symbol=self._symbol, ts_event=self._parse_dt(payload.get("timestamp")), ctx=ctx),
            payload=OrderBookDelta(bids=bids, asks=asks, sequence=sequence),
        )

    async def run_ws(
        self,
        *,
        connect_fn: Callable[[str], AsyncContextManager[Any]],
        url: str,
        on_ticker: Callable[[Envelope[TickerSnapshot]], Awaitable[None]],
        on_trade: Callable[[Envelope[TradeEvent]], Awaitable[None]],
        on_orderbook: Callable[[Envelope[OrderBookDelta]], Awaitable[None]],
        ctx: ExecuteContext,
        stop_after_messages: int | None = None,
    ) -> None:
        count = 0
        while True:
            try:
                async with connect_fn(url) as ws:
                    for channel in ("ticker", "trades", "orderbooks"):
                        await ws.send(json.dumps({"command": "subscribe", "channel": channel, "symbol": self._symbol}))
                    async for message in ws:
                        payload = json.loads(message)
                        channel = str(payload.get("channel") or "")
                        if channel == "ticker":
                            item = Envelope(
                                meta=self._meta(symbol=self._symbol, ts_event=self._parse_dt(payload.get("timestamp")), ctx=ctx),
                                payload=TickerSnapshot(
                                    bid=float(payload.get("bid")) if payload.get("bid") is not None else None,
                                    ask=float(payload.get("ask")) if payload.get("ask") is not None else None,
                                    last=float(payload.get("last") or payload.get("price")) if (payload.get("last") or payload.get("price")) is not None else None,
                                ),
                            )
                            await on_ticker(item)
                        elif channel == "trades":
                            trade = Envelope(
                                meta=self._meta(symbol=self._symbol, ts_event=self._parse_dt(payload.get("timestamp")), ctx=ctx),
                                payload=TradeEvent(
                                    trade_id=str(payload.get("id") or payload.get("timestamp") or "unknown"),
                                    side=str(payload.get("side") or "unknown").lower(),
                                    price=float(payload.get("price")),
                                    amount=float(payload.get("size") or payload.get("amount")),
                                    ts_event=self._parse_dt(payload.get("timestamp")),
                                ),
                            )
                            await on_trade(trade)
                        elif channel == "orderbooks" and str(payload.get("type", "")).lower() == "delta":
                            delta = await self.process_orderbook_delta(payload, ctx)
                            if delta is not None:
                                await on_orderbook(delta)

                        count += 1
                        if stop_after_messages is not None and count >= stop_after_messages:
                            return
            except asyncio.CancelledError:
                raise
            except Exception:
                self._metrics.ws_reconnect_total += 1
                if stop_after_messages is not None:
                    return
                await asyncio.sleep(0.05)
