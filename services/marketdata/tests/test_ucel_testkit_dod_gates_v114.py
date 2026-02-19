from __future__ import annotations

import asyncio
import json
import urllib.error
from email.message import Message
from typing import Any

from services.marketdata.app.gmo_public_marketdata import GmoPublicMarketDataAdapter
from services.marketdata.app.transport import HttpTransportClient, RetryPolicy, WsTransportClient
from services.marketdata.app.ucel_core import ExecuteContext


class MockRestServer:
    def __init__(self) -> None:
        self._routes: dict[str, list[Any]] = {}
        self.calls: dict[str, int] = {}

    def queue(self, path: str, responses: list[Any]) -> None:
        self._routes[path] = list(responses)
        self.calls[path] = 0

    def request(self, op_name: str, _query: str) -> dict[str, Any]:
        self.calls[op_name] = self.calls.get(op_name, 0) + 1
        route = self._routes.get(op_name)
        if not route:
            raise AssertionError(f"no queued response for {op_name}")
        item = route.pop(0)
        if isinstance(item, Exception):
            raise item
        return item


class _ScriptedWs:
    def __init__(self, events: list[Any]) -> None:
        self._events = list(events)
        self.sent: list[str] = []

    async def send(self, message: str) -> None:
        self.sent.append(message)

    def __aiter__(self) -> "_ScriptedWs":
        return self

    async def __anext__(self) -> str:
        if not self._events:
            raise StopAsyncIteration
        event = self._events.pop(0)
        if event == "disconnect":
            raise RuntimeError("injected disconnect")
        return str(event)


class MockWsServer:
    def __init__(self, sessions: list[_ScriptedWs]) -> None:
        self._sessions = list(sessions)
        self.calls = 0

    def __call__(self, _url: str) -> "MockWsServer":
        self.calls += 1
        return self

    async def __aenter__(self) -> _ScriptedWs:
        if not self._sessions:
            raise RuntimeError("no more sessions")
        return self._sessions.pop(0)

    async def __aexit__(self, exc_type: Any, exc: Any, tb: Any) -> bool:
        return False


def _collect(items: list[str], value: str) -> asyncio.Future[None]:
    items.append(value)
    fut: asyncio.Future[None] = asyncio.Future()
    fut.set_result(None)
    return fut


def test_ws_reconnect_and_resubscribe_gate() -> None:
    ws_first = _ScriptedWs(["first", "disconnect"])
    ws_second = _ScriptedWs(["second"])
    ws_server = MockWsServer([ws_first, ws_second])

    received: list[str] = []
    resubscribed: list[str] = []

    async def _run() -> None:
        client = WsTransportClient(
            url="wss://example",
            connect_fn=ws_server,
            subscriptions=["sub-ticker", "sub-orderbook"],
            on_message=lambda msg: _collect(received, msg),
            on_resubscribe=lambda: _collect(resubscribed, "ok"),
            reconnect_seconds=0.01,
            queue_maxsize=2,
        )
        task = asyncio.create_task(client.run_forever())
        await asyncio.sleep(0.05)
        await client.stop()
        await asyncio.wait_for(task, timeout=1)

    asyncio.run(_run())

    assert ws_server.calls >= 2
    assert received == ["first", "second"]
    assert resubscribed[:2] == ["ok", "ok"]
    assert len(ws_first.sent) == 2
    assert len(ws_second.sent) == 2


def test_orderbook_gap_to_resync_gate() -> None:
    rest = MockRestServer()
    rest.queue(
        "crypto.public.rest.orderbooks.get",
        [
            {
                "status": 0,
                "data": {
                    "timestamp": "2026-01-01T00:00:00.000Z",
                    "sequence": "100",
                    "bids": [{"price": "99", "size": "1"}],
                    "asks": [{"price": "100", "size": "1"}],
                },
            }
        ],
    )

    adapter = GmoPublicMarketDataAdapter(request_fn=rest.request)

    async def _run() -> None:
        ctx = ExecuteContext(trace_id="trace", request_id="req", run_id="run")
        first = await adapter.process_orderbook_delta(
            {"sequence": 1, "changes": {"bids": [{"price": "100", "size": "1"}], "asks": [{"price": "101", "size": "1"}]}},
            ctx,
        )
        second = await adapter.process_orderbook_delta(
            {"sequence": 3, "changes": {"bids": [{"price": "100.1", "size": "1"}], "asks": []}},
            ctx,
        )
        assert first is not None
        assert second is not None

    asyncio.run(_run())

    assert adapter.metrics.stale_total == 1
    assert adapter.metrics.orderbook_resync_total == 1
    assert adapter._orderbook_state.degraded is False
    assert rest.calls["crypto.public.rest.orderbooks.get"] == 1


def test_http_429_retry_after_respected_without_storm() -> None:
    sleeps: list[float] = []
    calls = {"count": 0}

    def _request(_req: Any, _timeout: float) -> Any:
        calls["count"] += 1
        if calls["count"] <= 2:
            headers = Message()
            headers["Retry-After"] = "1"
            raise urllib.error.HTTPError("http://x", 429, "too many", headers, None)

        class _Resp:
            def __enter__(self) -> "_Resp":
                return self

            def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> bool:
                return False

            def read(self) -> bytes:
                return b"ok"

        return _Resp()

    client = HttpTransportClient(
        retry_policy=RetryPolicy(max_attempts=3, initial_backoff_seconds=0.01, jitter_ratio=0.0, respect_retry_after=True),
        sleep_fn=sleeps.append,
        request_fn=_request,
    )

    body = client.request(op_name="fetch_ticker", method="GET", url="http://x", timeout_seconds=1)

    assert body == b"ok"
    assert calls["count"] == 3
    assert sleeps == [1.0, 1.0]


def test_retry_taxonomy_matches_transport_contract() -> None:
    headers = Message()
    http_500 = urllib.error.HTTPError("http://x", 500, "bad gateway", headers, None)
    http_400 = urllib.error.HTTPError("http://x", 400, "bad request", headers, None)

    assert HttpTransportClient._error_code(TimeoutError("t")) == "TIMEOUT"
    assert HttpTransportClient._error_code(urllib.error.URLError("down")) == "NETWORK"
    assert HttpTransportClient._error_code(http_500) == "UPSTREAM_5XX"
    assert HttpTransportClient._error_code(http_400) == "NON_RETRYABLE"


def test_secret_sanitize_framework_masks_secret_values() -> None:
    from services.marketdata.app.logging import scrub_sensitive_fields

    raw = {
        "api_key": "k-123",
        "secret_key": "s-123",
        "Authorization": "Bearer abc",
        "key_id": "kid-001",
        "payload": {"a": 1},
        "safe": "ok",
    }

    sanitized = scrub_sensitive_fields(raw)

    assert sanitized["api_key"] == "<redacted>"
    assert sanitized["secret_key"] == "<redacted>"
    assert sanitized["Authorization"] == "<redacted>"
    assert sanitized["payload"] == "<redacted>"
    assert sanitized["key_id"] == "kid-001"
    assert sanitized["safe"] == "ok"
    assert "k-123" not in json.dumps(sanitized)
    assert "s-123" not in json.dumps(sanitized)
