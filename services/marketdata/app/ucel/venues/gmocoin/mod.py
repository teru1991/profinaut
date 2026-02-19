from __future__ import annotations

import json
from pathlib import Path
from typing import Any, AsyncContextManager, Awaitable, Callable

from services.marketdata.app.registry import load_venue_registry
from services.marketdata.app.ucel_core import (
    Capabilities,
    CoreError,
    Envelope,
    ErrorCode,
    Exchange,
    ExecuteContext,
    OpName,
    OrderBookDelta,
    OrderBookSnapshot,
    TickerSnapshot,
    TradeEvent,
)
from services.marketdata.app.ucel.venues.gmocoin.capabilities import build_scope_from_catalog, to_capabilities
from services.marketdata.app.ucel.venues.gmocoin.normalize import to_snapshot, to_ticker, to_trades
from services.marketdata.app.ucel.venues.gmocoin.orderbook import OrderBookEngine
from services.marketdata.app.ucel.venues.gmocoin.public_rest import GmoPublicRestClient
from services.marketdata.app.ucel.venues.gmocoin.public_ws import GmoPublicWsClient


class GmoPublicMarketDataAdapter(Exchange):
    def __init__(self, *, symbol: str = "BTC_JPY", timeout_seconds: float = 5.0, http_client: Any | None = None, request_fn: Callable[[str, str], dict[str, Any]] | None = None) -> None:
        self._symbol = symbol
        self._registry = load_venue_registry("gmocoin")
        catalog = json.loads(Path(self._registry.catalog_path).read_text(encoding="utf-8"))
        self._scope = build_scope_from_catalog(catalog)
        self._rest = GmoPublicRestClient(
            timeout_seconds=timeout_seconds,
            catalog_scope=self._scope.rest_by_op,
            http_client=http_client,
            request_fn=request_fn,
        )
        self._orderbook = OrderBookEngine()
        self._ws = GmoPublicWsClient(ws_scope=self._scope.ws_by_op, orderbook_engine=self._orderbook, symbol=self._symbol)

    @property
    def metrics(self) -> dict[str, int]:
        return {
            "ws_reconnect_total": self._ws.metrics.ws_reconnect_total,
            "orderbook_resync_total": self._orderbook.metrics.orderbook_resync_total,
            "parse_failures_total": self._orderbook.metrics.parse_failures_total,
        }

    @property
    def unsupported_catalog_ids(self) -> tuple[str, ...]:
        return self._scope.unsupported_catalog_ids

    def capabilities(self) -> Capabilities:
        return to_capabilities(self._scope)

    def _execute_impl(self, op: OpName, params: dict[str, Any], ctx: ExecuteContext) -> Any:
        GmoPublicRestClient.assert_public_only(ctx.has_auth, ctx.secret_ref)
        if op == OpName.FETCH_TICKER:
            return self._fetch_ticker(symbol=str(params["symbol"]), ctx=ctx)
        if op == OpName.FETCH_TRADES:
            return self._fetch_trades(symbol=str(params["symbol"]), limit=int(params.get("limit", 100)), since=params.get("since"), ctx=ctx)
        if op == OpName.FETCH_ORDERBOOK_SNAPSHOT:
            return self._fetch_orderbook_snapshot(symbol=str(params["symbol"]), depth=int(params.get("depth", 50)), ctx=ctx)
        raise CoreError(ErrorCode.NOT_SUPPORTED, f"Unsupported op={op.value}")

    def _fetch_ticker(self, *, symbol: str, ctx: ExecuteContext) -> Envelope[TickerSnapshot]:
        payload = self._rest.fetch(OpName.FETCH_TICKER, {"symbol": symbol})
        item = (payload.get("data") or [{}])[0]
        return to_ticker(symbol=symbol, item=item, trace_id=ctx.trace_id, request_id=ctx.request_id, run_id=ctx.run_id)

    def _fetch_trades(self, *, symbol: str, limit: int, since: str | None, ctx: ExecuteContext) -> Envelope[tuple[TradeEvent, ...]]:
        endpoint = self._scope.rest_by_op.get(OpName.FETCH_TRADES)
        query: dict[str, str] = {"symbol": symbol}
        if endpoint is None:
            raise CoreError(ErrorCode.NOT_SUPPORTED, "trades endpoint unavailable")
        names = {q.get("name") for q in endpoint.get("params", {}).get("query", []) if isinstance(q, dict)}
        if "count" in names:
            query["count"] = str(limit)
        elif limit:
            raise CoreError(ErrorCode.NOT_SUPPORTED, "catalog missing 'count' parameter definition")
        if since:
            raise CoreError(ErrorCode.NOT_SUPPORTED, "catalog does not define 'since' parameter")
        payload = self._rest.fetch(OpName.FETCH_TRADES, query)
        rows = payload.get("data") or payload.get("list") or []
        return to_trades(symbol=symbol, rows=rows, trace_id=ctx.trace_id, request_id=ctx.request_id, run_id=ctx.run_id)

    def _fetch_orderbook_snapshot(self, *, symbol: str, depth: int, ctx: ExecuteContext) -> Envelope[OrderBookSnapshot]:
        endpoint = self._scope.rest_by_op.get(OpName.FETCH_ORDERBOOK_SNAPSHOT)
        if endpoint is None:
            raise CoreError(ErrorCode.NOT_SUPPORTED, "orderbook snapshot endpoint unavailable")
        names = {q.get("name") for q in endpoint.get("params", {}).get("query", []) if isinstance(q, dict)}
        if depth and "depth" not in names and depth != 50:
            raise CoreError(ErrorCode.NOT_SUPPORTED, "catalog missing 'depth' parameter definition")
        payload = self._rest.fetch(OpName.FETCH_ORDERBOOK_SNAPSHOT, {"symbol": symbol})
        item = (payload.get("data") or [{}])[0] if isinstance(payload.get("data"), list) else payload.get("data", payload)
        return to_snapshot(symbol=symbol, item=item, depth=depth, trace_id=ctx.trace_id, request_id=ctx.request_id, run_id=ctx.run_id)

    async def run_ws(
        self,
        *,
        connect_fn: Callable[[str], AsyncContextManager[Any]],
        on_ticker: Callable[[Envelope[TickerSnapshot]], Awaitable[None]],
        on_trade: Callable[[Envelope[TradeEvent]], Awaitable[None]],
        on_orderbook: Callable[[Envelope[OrderBookDelta]], Awaitable[None]],
        ctx: ExecuteContext,
        stop_after_messages: int | None = None,
    ) -> None:
        GmoPublicRestClient.assert_public_only(ctx.has_auth, ctx.secret_ref)
        await self._ws.run(
            connect_fn=connect_fn,
            on_ticker=on_ticker,
            on_trade=on_trade,
            on_orderbook=on_orderbook,
            ctx=ctx,
            snapshot_fetcher=lambda: self.fetch_orderbook_snapshot(symbol=self._symbol, depth=50, ctx=ctx),
            stop_after_messages=stop_after_messages,
        )
