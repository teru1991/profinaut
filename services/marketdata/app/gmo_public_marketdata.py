from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from services.marketdata.app.ucel.venues.gmocoin.mod import GmoPublicMarketDataAdapter as _Adapter
from services.marketdata.app.ucel_core import ExecuteContext


@dataclass
class _CompatMetrics:
    stale_total: int
    ws_reconnect_total: int
    orderbook_resync_total: int
    parse_failures_total: int

    def __getitem__(self, key: str) -> int:
        return getattr(self, key)


class GmoPublicMarketDataAdapter(_Adapter):
    @property
    def _orderbook_state(self):
        return self._orderbook.state

    @property
    def metrics(self) -> _CompatMetrics:
        return _CompatMetrics(
            stale_total=self._orderbook.metrics.orderbook_resync_total,
            ws_reconnect_total=self._ws.metrics.ws_reconnect_total,
            orderbook_resync_total=self._orderbook.metrics.orderbook_resync_total,
            parse_failures_total=self._orderbook.metrics.parse_failures_total,
        )

    async def process_orderbook_delta(self, payload: dict[str, Any], ctx: ExecuteContext):
        return await self._orderbook.process_delta(
            symbol=self._symbol,
            payload=payload,
            ctx=ctx,
            snapshot_fetcher=lambda: self.fetch_orderbook_snapshot(symbol=self._symbol, depth=50, ctx=ctx),
        )


__all__ = ["GmoPublicMarketDataAdapter"]
