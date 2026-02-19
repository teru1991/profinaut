from __future__ import annotations

import json
from typing import Any

from services.marketdata.app.gmo_public_marketdata import GmoPublicMarketDataAdapter
from services.marketdata.app.ucel_core import ExecuteContext


class _FakeWs:
    def __init__(self, messages: list[str]) -> None:
        self.sent: list[str] = []
        self._messages = list(messages)

    async def send(self, message: str) -> None:
        self.sent.append(message)

    def __aiter__(self) -> "_FakeWs":
        return self

    async def __anext__(self) -> str:
        if not self._messages:
            raise StopAsyncIteration
        return self._messages.pop(0)


class _FakeConnect:
    def __init__(self, ws: _FakeWs) -> None:
        self._ws = ws

    def __call__(self, _url: str) -> "_FakeConnect":
        return self

    async def __aenter__(self) -> _FakeWs:
        return self._ws

    async def __aexit__(self, exc_type: Any, exc: Any, tb: Any) -> bool:
        return False


def test_rest_fixture_parse() -> None:
    fixtures = {
        "crypto.public.rest.ticker.get": {"status": 0, "data": [{"symbol": "BTC_JPY", "timestamp": "2026-01-01T00:00:00.000Z", "bid": "100", "ask": "101", "last": "100.5"}]},
        "crypto.public.rest.trades.get": {"status": 0, "data": [{"id": "t1", "timestamp": "2026-01-01T00:00:01.000Z", "side": "BUY", "price": "100.4", "size": "0.5"}]},
        "crypto.public.rest.orderbooks.get": {"status": 0, "data": {"timestamp": "2026-01-01T00:00:02.000Z", "sequence": "10", "bids": [{"price": "100", "size": "1.2"}], "asks": [{"price": "101", "size": "1.1"}]}},
    }

    adapter = GmoPublicMarketDataAdapter(request_fn=lambda op, _q: fixtures[op])
    ctx = ExecuteContext(trace_id="trace-x", request_id="req-x", run_id="run-x")

    ticker = adapter.fetch_ticker(symbol="BTC_JPY", ctx=ctx)
    assert ticker.payload.bid == 100.0
    assert ticker.meta.trace_id == "trace-x"

    trades = adapter.fetch_trades(symbol="BTC_JPY", limit=1, ctx=ctx)
    assert len(trades.payload) == 1
    assert trades.payload[0].trade_id == "t1"

    snapshot = adapter.fetch_orderbook_snapshot(symbol="BTC_JPY", depth=10, ctx=ctx)
    assert snapshot.payload.sequence == 10
    assert snapshot.payload.bids[0].price == 100.0


async def _noop(*_args: Any, **_kwargs: Any) -> None:
    return None


def test_ws_gap_resync() -> None:
    calls: dict[str, int] = {"snapshot": 0}

    def _request(op: str, _query: str) -> dict[str, Any]:
        if op == "crypto.public.rest.orderbooks.get":
            calls["snapshot"] += 1
            return {
                "status": 0,
                "data": {"timestamp": "2026-01-01T00:00:00.000Z", "sequence": "100", "bids": [{"price": "99", "size": "1"}], "asks": [{"price": "100", "size": "1"}]},
            }
        return {"status": 0, "data": []}

    messages = [
        json.dumps({"channel": "orderbooks", "type": "delta", "sequence": 1, "changes": {"bids": [{"price": "100", "size": "1"}], "asks": [{"price": "101", "size": "1"}]}}),
        json.dumps({"channel": "orderbooks", "type": "delta", "sequence": 3, "changes": {"bids": [{"price": "100.2", "size": "1.1"}], "asks": []}}),
    ]

    ws = _FakeWs(messages)
    adapter = GmoPublicMarketDataAdapter(request_fn=_request)

    import asyncio

    asyncio.run(
        adapter.run_ws(
            connect_fn=_FakeConnect(ws),
            url="wss://example",
            on_ticker=_noop,
            on_trade=_noop,
            on_orderbook=_noop,
            ctx=ExecuteContext(trace_id="t", request_id="r", run_id="u"),
            stop_after_messages=2,
        )
    )

    assert calls["snapshot"] == 1
    assert adapter.metrics.orderbook_resync_total == 1
    assert len(ws.sent) == 3
