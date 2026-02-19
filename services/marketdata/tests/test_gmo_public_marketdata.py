from __future__ import annotations

import asyncio
import json
from pathlib import Path
from typing import Any

import pytest

from services.marketdata.app.gmo_public_marketdata import GmoPublicMarketDataAdapter
from services.marketdata.app.ucel_core import CoreError, ExecuteContext


FIX = Path(__file__).parent / "fixtures" / "gmocoin"


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
        msg = self._messages.pop(0)
        if isinstance(msg, Exception):
            raise msg
        return msg


class _Session:
    def __init__(self, ws: _FakeWs) -> None:
        self.ws = ws

    async def __aenter__(self) -> _FakeWs:
        return self.ws

    async def __aexit__(self, exc_type: Any, exc: Any, tb: Any) -> bool:
        return False


class _MockWsServer:
    def __init__(self, rounds: list[list[str | Exception]]) -> None:
        self.rounds = list(rounds)
        self.calls = 0
        self.subscriptions: list[list[dict[str, Any]]] = []

    def __call__(self, _url: str) -> _Session:
        messages = self.rounds[self.calls]
        self.calls += 1
        ws = _FakeWs([m for m in messages])

        original_send = ws.send

        async def _send(message: str) -> None:
            await original_send(message)
            if len(self.subscriptions) < self.calls:
                self.subscriptions.append([])
            self.subscriptions[self.calls - 1].append(json.loads(message))

        ws.send = _send  # type: ignore[assignment]
        return _Session(ws)


def _fixtures_request(op: str, _query: str) -> dict[str, Any]:
    mapping = {
        "crypto.public.rest.ticker.get": "ticker.json",
        "crypto.public.rest.trades.get": "trades.json",
        "crypto.public.rest.orderbooks.get": "orderbook_snapshot.json",
    }
    return json.loads((FIX / mapping[op]).read_text(encoding="utf-8"))


def test_rest_fixture_parsing_and_meta() -> None:
    adapter = GmoPublicMarketDataAdapter(request_fn=_fixtures_request)
    ctx = ExecuteContext(trace_id="trace-x", request_id="req-x", run_id="run-x")

    ticker = adapter.fetch_ticker(symbol="BTC_JPY", ctx=ctx)
    assert ticker.payload.bid == 100.0
    assert ticker.meta.venue == "gmocoin"
    assert ticker.meta.schema_version == "1.1.4"

    trades = adapter.fetch_trades(symbol="BTC_JPY", limit=1, ctx=ctx)
    assert len(trades.payload) == 1
    assert trades.payload[0].trade_id == "t1"
    assert trades.meta.request_id == "req-x"

    snapshot = adapter.fetch_orderbook_snapshot(symbol="BTC_JPY", depth=50, ctx=ctx)
    assert snapshot.payload.sequence == 10
    assert snapshot.meta.run_id == "run-x"


def test_public_flow_rejects_auth_context() -> None:
    adapter = GmoPublicMarketDataAdapter(request_fn=_fixtures_request)
    with pytest.raises(CoreError):
        adapter.fetch_ticker(symbol="BTC_JPY", ctx=ExecuteContext(has_auth=True))


def test_ws_reconnect_resubscribe_and_gap_resync() -> None:
    calls = {"snapshot": 0}

    def _request(op: str, _query: str) -> dict[str, Any]:
        if op == "crypto.public.rest.orderbooks.get":
            calls["snapshot"] += 1
            return {
                "status": 0,
                "data": {
                    "timestamp": "2026-01-01T00:00:00.000Z",
                    "sequence": "100",
                    "bids": [{"price": "99", "size": "1"}],
                    "asks": [{"price": "100", "size": "1"}],
                },
            }
        return _fixtures_request(op, _query)

    server = _MockWsServer(
        [
            [
                json.dumps({"channel": "ticker", "timestamp": "2026-01-01T00:00:00.000Z", "bid": "100", "ask": "101", "last": "100.2"}),
                ConnectionError("drop"),
            ],
            [
                json.dumps({"channel": "trades", "timestamp": "2026-01-01T00:00:01.000Z", "id": "t2", "side": "BUY", "price": "100", "size": "0.1"}),
                json.dumps({"channel": "orderbooks", "sequence": 1, "changes": {"bids": [{"price": "100", "size": "1"}], "asks": []}}),
                json.dumps({"channel": "orderbooks", "sequence": 2, "changes": {"bids": [{"price": "100.1", "size": "1"}], "asks": []}}),
                json.dumps({"channel": "orderbooks", "sequence": 4, "changes": {"bids": [{"price": "100.2", "size": "1"}], "asks": []}}),
            ],
        ]
    )

    seen = {"ticker": 0, "trade": 0, "book": 0}

    async def _run() -> None:
        adapter = GmoPublicMarketDataAdapter(request_fn=_request)
        await adapter.run_ws(
            connect_fn=server,
            on_ticker=lambda _x: _count(seen, "ticker"),
            on_trade=lambda _x: _count(seen, "trade"),
            on_orderbook=lambda _x: _count(seen, "book"),
            ctx=ExecuteContext(trace_id="t", request_id="r", run_id="u"),
            stop_after_messages=5,
        )
        assert adapter.metrics["ws_reconnect_total"] == 1
        assert adapter.metrics["orderbook_resync_total"] == 1

    async def _count(bucket: dict[str, int], key: str) -> None:
        bucket[key] += 1

    asyncio.run(_run())

    assert server.calls == 2
    assert len(server.subscriptions) == 2
    assert len(server.subscriptions[0]) == 3
    assert len(server.subscriptions[1]) == 3
    assert calls["snapshot"] == 1
    assert seen["ticker"] == 1 and seen["trade"] == 1
