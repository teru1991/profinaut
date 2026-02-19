from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Awaitable, Callable

from services.marketdata.app.ucel_core import Envelope, ExecuteContext, OrderBookSnapshot
from services.marketdata.app.ucel.venues.gmocoin.normalize import to_delta

logger = logging.getLogger("marketdata.gmocoin.orderbook")


@dataclass
class OrderBookMetrics:
    orderbook_resync_total: int = 0
    parse_failures_total: int = 0


@dataclass
class OrderBookState:
    last_sequence: int | None = None
    degraded: bool = False


class OrderBookEngine:
    def __init__(self) -> None:
        self.state = OrderBookState()
        self.metrics = OrderBookMetrics()

    async def process_delta(
        self,
        *,
        symbol: str,
        payload: dict,
        ctx: ExecuteContext,
        snapshot_fetcher: Callable[[], Envelope[OrderBookSnapshot]],
    ):
        try:
            sequence = int(payload.get("sequence"))
        except (TypeError, ValueError):
            self.metrics.parse_failures_total += 1
            return None

        expected = self.state.last_sequence + 1 if self.state.last_sequence is not None else None
        if expected is not None and sequence != expected:
            self.state.degraded = True
            self.metrics.orderbook_resync_total += 1
            logger.warning(
                "orderbook_gap_detected venue=gmocoin symbol=%s op=subscribe_orderbook prev_seq=%s next_seq=%s trace_id=%s request_id=%s run_id=%s error_code=SEQUENCE_GAP",
                symbol,
                self.state.last_sequence,
                sequence,
                ctx.trace_id,
                ctx.request_id,
                ctx.run_id,
            )
            snapshot = snapshot_fetcher()
            self.state.last_sequence = snapshot.payload.sequence
            self.state.degraded = False

        self.state.last_sequence = sequence
        return to_delta(symbol=symbol, payload=payload, trace_id=ctx.trace_id, request_id=ctx.request_id, run_id=ctx.run_id)
