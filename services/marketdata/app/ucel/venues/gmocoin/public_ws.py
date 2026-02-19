from __future__ import annotations

import asyncio
import json
import logging
from dataclasses import dataclass
from typing import Any, AsyncContextManager, Awaitable, Callable

from services.marketdata.app.ucel_core import Envelope, ExecuteContext, OrderBookDelta, TickerSnapshot, TradeEvent
from services.marketdata.app.ucel.venues.gmocoin.normalize import to_ticker, to_trades
from services.marketdata.app.ucel.venues.gmocoin.orderbook import OrderBookEngine

logger = logging.getLogger("marketdata.gmocoin.ws")


@dataclass
class WsMetrics:
    ws_reconnect_total: int = 0


class GmoPublicWsClient:
    def __init__(self, *, ws_scope: dict, orderbook_engine: OrderBookEngine, symbol: str) -> None:
        self._scope = ws_scope
        self._orderbook_engine = orderbook_engine
        self._symbol = symbol
        self.metrics = WsMetrics()

    def _subscriptions(self) -> list[str]:
        templates = []
        for op in self._scope.values():
            template = op.get("subscribe", {}).get("template")
            if isinstance(template, str):
                templates.append(template.replace("<symbol>", self._symbol))
        return templates

    def ws_url(self) -> str:
        first = next(iter(self._scope.values()), None)
        if not first:
            return ""
        return str(first.get("ws_url") or "")

    async def run(
        self,
        *,
        connect_fn: Callable[[str], AsyncContextManager[Any]],
        on_ticker: Callable[[Envelope[TickerSnapshot]], Awaitable[None]],
        on_trade: Callable[[Envelope[TradeEvent]], Awaitable[None]],
        on_orderbook: Callable[[Envelope[OrderBookDelta]], Awaitable[None]],
        ctx: ExecuteContext,
        snapshot_fetcher: Callable[[], Any],
        stop_after_messages: int | None = None,
    ) -> None:
        count = 0
        while True:
            try:
                async with connect_fn(self.ws_url()) as ws:
                    for sub in self._subscriptions():
                        await ws.send(sub)
                    async for message in ws:
                        payload = json.loads(message)
                        channel = str(payload.get("channel") or "")
                        if channel == "ticker":
                            await on_ticker(to_ticker(symbol=self._symbol, item=payload, trace_id=ctx.trace_id, request_id=ctx.request_id, run_id=ctx.run_id))
                        elif channel == "trades":
                            trade_rows = [payload]
                            events = to_trades(symbol=self._symbol, rows=trade_rows, trace_id=ctx.trace_id, request_id=ctx.request_id, run_id=ctx.run_id)
                            if events.payload:
                                await on_trade(Envelope(meta=events.meta, payload=events.payload[0]))
                        elif channel == "orderbooks":
                            delta = await self._orderbook_engine.process_delta(
                                symbol=self._symbol,
                                payload=payload,
                                ctx=ctx,
                                snapshot_fetcher=snapshot_fetcher,
                            )
                            if delta is not None:
                                await on_orderbook(delta)
                        count += 1
                        if stop_after_messages is not None and count >= stop_after_messages:
                            return
            except asyncio.CancelledError:
                raise
            except Exception as exc:
                self.metrics.ws_reconnect_total += 1
                logger.warning(
                    "ws_reconnect venue=gmocoin symbol=%s op=subscribe error_code=WS_RECONNECT trace_id=%s request_id=%s run_id=%s err=%s",
                    self._symbol,
                    ctx.trace_id,
                    ctx.request_id,
                    ctx.run_id,
                    type(exc).__name__,
                )
                await asyncio.sleep(0.05)
