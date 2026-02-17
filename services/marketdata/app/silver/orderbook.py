from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class OrderbookLevel:
    price: float
    size: float


class OrderbookEngine:
    def __init__(self) -> None:
        self._bids: dict[float, float] = {}
        self._asks: dict[float, float] = {}

    @staticmethod
    def _coerce_levels(levels: list[dict[str, Any]] | None) -> list[OrderbookLevel]:
        if not levels:
            return []
        result: list[OrderbookLevel] = []
        for level in levels:
            if not isinstance(level, dict):
                continue
            try:
                price = float(level.get("price"))
                size = float(level.get("size"))
            except (TypeError, ValueError):
                continue
            result.append(OrderbookLevel(price=price, size=size))
        return result

    def load_from_bbo(
        self,
        *,
        bid_px: float | None,
        bid_qty: float | None,
        ask_px: float | None,
        ask_qty: float | None,
    ) -> None:
        self._bids.clear()
        self._asks.clear()
        if bid_px is not None and bid_qty is not None and bid_qty > 0:
            self._bids[float(bid_px)] = float(bid_qty)
        if ask_px is not None and ask_qty is not None and ask_qty > 0:
            self._asks[float(ask_px)] = float(ask_qty)

    def apply_snapshot(self, snapshot: dict[str, Any]) -> None:
        self._bids.clear()
        self._asks.clear()
        for level in self._coerce_levels(snapshot.get("bids")):
            if level.size > 0:
                self._bids[level.price] = level.size
        for level in self._coerce_levels(snapshot.get("asks")):
            if level.size > 0:
                self._asks[level.price] = level.size

    def apply_delta(self, delta: dict[str, Any]) -> None:
        for level in self._coerce_levels(delta.get("bids")):
            if level.size <= 0:
                self._bids.pop(level.price, None)
            else:
                self._bids[level.price] = level.size
        for level in self._coerce_levels(delta.get("asks")):
            if level.size <= 0:
                self._asks.pop(level.price, None)
            else:
                self._asks[level.price] = level.size

    def derive_bbo(self) -> tuple[OrderbookLevel | None, OrderbookLevel | None]:
        best_bid = None
        if self._bids:
            bid_px = max(self._bids.keys())
            best_bid = OrderbookLevel(price=bid_px, size=self._bids[bid_px])

        best_ask = None
        if self._asks:
            ask_px = min(self._asks.keys())
            best_ask = OrderbookLevel(price=ask_px, size=self._asks[ask_px])

        return best_bid, best_ask

    @staticmethod
    def check_gap(prev_seq: int | None, next_seq: int | None) -> bool:
        if prev_seq is None or next_seq is None:
            return False
        return next_seq != prev_seq + 1
