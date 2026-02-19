from __future__ import annotations

import asyncio
import urllib.error
from datetime import UTC, datetime, timedelta
from email.message import Message
from typing import Any

from services.marketdata.app.transport import (
    CircuitBreaker,
    CircuitBreakerPolicy,
    HttpTransportClient,
    RateLimiter,
    RetryPolicy,
    TransportAuthError,
    WsTransportClient,
)


class _FakeHttpResponse:
    def __init__(self, payload: bytes):
        self._payload = payload

    def __enter__(self) -> "_FakeHttpResponse":
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> bool:
        return False

    def read(self) -> bytes:
        return self._payload


class _FakeWs:
    def __init__(self, messages: list[str], *, raise_error: bool = False):
        self._messages = list(messages)
        self._raise_error = raise_error
        self.sent: list[str] = []

    async def send(self, message: str) -> None:
        self.sent.append(message)

    def __aiter__(self) -> "_FakeWs":
        return self

    async def __anext__(self) -> str:
        if self._messages:
            return self._messages.pop(0)
        if self._raise_error:
            raise RuntimeError("socket-disconnected")
        raise StopAsyncIteration


class _FakeConnect:
    def __init__(self, sockets: list[_FakeWs]):
        self._sockets = sockets
        self.calls = 0

    def __call__(self, _url: str) -> "_FakeConnect":
        self.calls += 1
        return self

    async def __aenter__(self) -> _FakeWs:
        if not self._sockets:
            raise RuntimeError("no socket")
        return self._sockets.pop(0)

    async def __aexit__(self, exc_type: Any, exc: Any, tb: Any) -> bool:
        return False


def test_http_transport_respects_retry_after_for_429() -> None:
    sleeps: list[float] = []
    calls = {"n": 0}

    def _request(_req: Any, _timeout: float) -> _FakeHttpResponse:
        calls["n"] += 1
        if calls["n"] == 1:
            headers = Message()
            headers["Retry-After"] = "2"
            raise urllib.error.HTTPError("http://x", 429, "too many", headers, None)
        return _FakeHttpResponse(b"ok")

    client = HttpTransportClient(
        retry_policy=RetryPolicy(max_attempts=2, initial_backoff_seconds=0.1, jitter_ratio=0.0, respect_retry_after=True),
        sleep_fn=sleeps.append,
        request_fn=_request,
    )

    body = client.request(op_name="ticker_latest", method="GET", url="http://x", timeout_seconds=1)

    assert body == b"ok"
    assert calls["n"] == 2
    assert sleeps == [2.0]


def test_http_transport_auth_boundary() -> None:
    client = HttpTransportClient(request_fn=lambda _req, _timeout: _FakeHttpResponse(b"ok"))

    try:
        client.request(op_name="private_balance", method="GET", url="http://x", timeout_seconds=1, is_private=True)
        raise AssertionError("expected auth error")
    except TransportAuthError:
        pass


def test_circuit_breaker_opens_and_half_opens() -> None:
    now = datetime(2026, 1, 1, tzinfo=UTC)

    def _now() -> datetime:
        return current["value"]

    current = {"value": now}
    breaker = CircuitBreaker(CircuitBreakerPolicy(failure_threshold=2, recovery_timeout_seconds=5), now_fn=_now)

    assert breaker.allow()
    breaker.on_failure()
    assert breaker.allow()
    breaker.on_failure()
    assert not breaker.allow()

    current["value"] = now + timedelta(seconds=6)
    assert breaker.allow()
    assert breaker.state == "half_open"


def test_ws_transport_reconnects_and_resubscribes() -> None:
    sockets = [_FakeWs(["m1"], raise_error=True), _FakeWs(["m2"]) ]
    connect = _FakeConnect(sockets)
    received: list[str] = []
    resubscribed: list[str] = []

    async def _run() -> None:
        client = WsTransportClient(
            url="wss://example",
            connect_fn=connect,
            subscriptions=["sub-a"],
            on_message=lambda m: _collect(received, m),
            on_resubscribe=lambda: _collect(resubscribed, "ok"),
            reconnect_seconds=0.01,
            queue_maxsize=2,
        )
        task = asyncio.create_task(client.run_forever())
        await asyncio.sleep(0.06)
        await client.stop()
        await asyncio.wait_for(task, timeout=1)

    asyncio.run(_run())

    assert connect.calls >= 2
    assert received == ["m1", "m2"]
    assert resubscribed[:2] == ["ok", "ok"]


def _collect(items: list[str], value: str) -> asyncio.Future[None]:
    items.append(value)
    fut: asyncio.Future[None] = asyncio.Future()
    fut.set_result(None)
    return fut


def test_rate_limiter_metrics_and_cost() -> None:
    now = datetime(2026, 1, 1, tzinfo=UTC)

    def _now() -> datetime:
        return current["value"]

    current = {"value": now}
    limiter = RateLimiter(max_cost_per_window=3, window_seconds=10, now_fn=_now)

    assert limiter.acquire(op_name="op-a", cost=2)
    assert not limiter.acquire(op_name="op-b", cost=2)
    assert limiter.metrics["rate_limited_total"] == 1
