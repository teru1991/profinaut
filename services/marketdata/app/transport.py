from __future__ import annotations

import asyncio
import random
import time
import urllib.error
import urllib.request
from collections import deque
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from email.utils import parsedate_to_datetime
from typing import Any, AsyncContextManager, Awaitable, Callable, Protocol


RETRYABLE_ERROR_CODES = {"TIMEOUT", "NETWORK", "UPSTREAM_5XX", "RATE_LIMITED"}


class TransportAuthError(RuntimeError):
    pass


@dataclass(frozen=True)
class RetryPolicy:
    max_attempts: int = 4
    initial_backoff_seconds: float = 0.5
    max_backoff_seconds: float = 8.0
    jitter_ratio: float = 0.2
    respect_retry_after: bool = True


@dataclass(frozen=True)
class CircuitBreakerPolicy:
    failure_threshold: int = 3
    recovery_timeout_seconds: float = 10.0


class RateLimiter:
    def __init__(self, *, max_cost_per_window: int, window_seconds: float, now_fn: Callable[[], datetime] | None = None) -> None:
        self._max_cost = max(1, int(max_cost_per_window))
        self._window = timedelta(seconds=max(window_seconds, 0.001))
        self._events: deque[tuple[datetime, int, str]] = deque()
        self._now_fn = now_fn or (lambda: datetime.now(UTC))
        self._limited_total = 0

    @property
    def metrics(self) -> dict[str, int]:
        return {"rate_limited_total": self._limited_total, "inflight_cost": self._inflight_cost()}

    def _prune(self, now: datetime) -> None:
        threshold = now - self._window
        while self._events and self._events[0][0] < threshold:
            self._events.popleft()

    def _inflight_cost(self) -> int:
        now = self._now_fn()
        self._prune(now)
        return sum(cost for _, cost, _ in self._events)

    def acquire(self, *, op_name: str, cost: int = 1) -> bool:
        now = self._now_fn()
        self._prune(now)
        normalized_cost = max(1, int(cost))
        if self._inflight_cost() + normalized_cost > self._max_cost:
            self._limited_total += 1
            return False
        self._events.append((now, normalized_cost, op_name))
        return True


class CircuitBreaker:
    def __init__(self, policy: CircuitBreakerPolicy, now_fn: Callable[[], datetime] | None = None) -> None:
        self._policy = policy
        self._now_fn = now_fn or (lambda: datetime.now(UTC))
        self._state = "closed"
        self._failures = 0
        self._opened_at: datetime | None = None

    @property
    def state(self) -> str:
        if self._state == "open" and self._opened_at is not None:
            if (self._now_fn() - self._opened_at).total_seconds() >= self._policy.recovery_timeout_seconds:
                self._state = "half_open"
        return self._state

    def allow(self) -> bool:
        return self.state in {"closed", "half_open"}

    def on_success(self) -> None:
        self._state = "closed"
        self._failures = 0
        self._opened_at = None

    def on_failure(self) -> None:
        self._failures += 1
        if self._failures >= self._policy.failure_threshold:
            self._state = "open"
            self._opened_at = self._now_fn()


class HttpTransportClient:
    def __init__(
        self,
        *,
        retry_policy: RetryPolicy | None = None,
        rate_limiter: RateLimiter | None = None,
        circuit_breaker: CircuitBreaker | None = None,
        sleep_fn: Callable[[float], None] = time.sleep,
        request_fn: Callable[[urllib.request.Request, float], Any] | None = None,
    ) -> None:
        self._retry_policy = retry_policy or RetryPolicy()
        self._rate_limiter = rate_limiter
        self._circuit_breaker = circuit_breaker
        self._sleep_fn = sleep_fn
        self._request_fn = request_fn or urllib.request.urlopen

    @staticmethod
    def _retry_after_seconds(exc: urllib.error.HTTPError) -> float | None:
        value = exc.headers.get("Retry-After") if exc.headers else None
        if not value:
            return None
        if value.isdigit():
            return float(value)
        try:
            parsed = parsedate_to_datetime(value)
        except (TypeError, ValueError):
            return None
        return max((parsed - datetime.now(parsed.tzinfo or UTC)).total_seconds(), 0.0)

    @staticmethod
    def _error_code(exc: Exception) -> str:
        if isinstance(exc, TimeoutError):
            return "TIMEOUT"
        if isinstance(exc, urllib.error.HTTPError):
            if exc.code == 429:
                return "RATE_LIMITED"
            if exc.code >= 500:
                return "UPSTREAM_5XX"
            return "NON_RETRYABLE"
        if isinstance(exc, urllib.error.URLError):
            return "NETWORK"
        return "NON_RETRYABLE"

    def request(
        self,
        *,
        op_name: str,
        method: str,
        url: str,
        timeout_seconds: float,
        headers: dict[str, str] | None = None,
        body: bytes | None = None,
        cost: int = 1,
        is_private: bool = False,
        auth_header: str | None = None,
    ) -> bytes:
        if is_private and not auth_header:
            raise TransportAuthError(f"private op={op_name} requires auth")
        if not is_private:
            auth_header = None

        if self._rate_limiter is not None and not self._rate_limiter.acquire(op_name=op_name, cost=cost):
            raise RuntimeError("rate-limited by client limiter")

        if self._circuit_breaker is not None and not self._circuit_breaker.allow():
            raise RuntimeError("circuit-open")

        base_headers = dict(headers or {})
        if auth_header is not None:
            base_headers["Authorization"] = auth_header

        attempts = 0
        while attempts < self._retry_policy.max_attempts:
            attempts += 1
            req = urllib.request.Request(url=url, method=method.upper(), data=body, headers=base_headers)
            try:
                with self._request_fn(req, timeout_seconds) as resp:
                    payload = resp.read()
                if self._circuit_breaker is not None:
                    self._circuit_breaker.on_success()
                return payload
            except Exception as exc:  # noqa: BLE001
                code = self._error_code(exc)
                if code in {"TIMEOUT", "UPSTREAM_5XX"} and self._circuit_breaker is not None:
                    self._circuit_breaker.on_failure()

                if attempts >= self._retry_policy.max_attempts or code not in RETRYABLE_ERROR_CODES:
                    raise

                delay = min(
                    self._retry_policy.initial_backoff_seconds * (2 ** (attempts - 1)),
                    self._retry_policy.max_backoff_seconds,
                )
                jitter = delay * self._retry_policy.jitter_ratio
                delay += random.uniform(-jitter, jitter) if jitter > 0 else 0.0

                if (
                    code == "RATE_LIMITED"
                    and self._retry_policy.respect_retry_after
                    and isinstance(exc, urllib.error.HTTPError)
                ):
                    retry_after = self._retry_after_seconds(exc)
                    if retry_after is not None:
                        delay = max(delay, retry_after)

                self._sleep_fn(max(delay, 0.0))

        raise RuntimeError("unreachable")


class WsConnection(Protocol):
    async def send(self, message: str) -> None: ...

    def __aiter__(self) -> Any: ...


WsConnectFn = Callable[[str], AsyncContextManager[WsConnection]]


class WsTransportClient:
    def __init__(
        self,
        *,
        url: str,
        connect_fn: WsConnectFn,
        subscriptions: list[str],
        on_message: Callable[[str], Awaitable[None]],
        on_resubscribe: Callable[[], Awaitable[None]] | None = None,
        queue_maxsize: int = 1024,
        drop_oldest: bool = True,
        reconnect_seconds: float = 1.0,
    ) -> None:
        self._url = url
        self._connect_fn = connect_fn
        self._subscriptions = subscriptions
        self._on_message = on_message
        self._on_resubscribe = on_resubscribe
        self._queue: asyncio.Queue[str] = asyncio.Queue(maxsize=max(1, queue_maxsize))
        self._drop_oldest = drop_oldest
        self._reconnect_seconds = reconnect_seconds
        self._stop = asyncio.Event()

    async def stop(self) -> None:
        self._stop.set()

    async def _push_queue(self, message: str) -> None:
        if self._queue.full() and self._drop_oldest:
            _ = self._queue.get_nowait()
        if not self._queue.full():
            self._queue.put_nowait(message)

    async def _drain_queue(self) -> None:
        while not self._queue.empty():
            await self._on_message(self._queue.get_nowait())

    async def run_forever(self) -> None:
        while not self._stop.is_set():
            try:
                async with self._connect_fn(self._url) as ws:
                    for sub in self._subscriptions:
                        await ws.send(sub)
                    if self._on_resubscribe is not None:
                        await self._on_resubscribe()

                    async for raw in ws:
                        if self._stop.is_set():
                            break
                        if not isinstance(raw, str):
                            continue
                        await self._push_queue(raw)
                        await self._drain_queue()
            except asyncio.CancelledError:
                raise
            except Exception:
                await asyncio.sleep(self._reconnect_seconds)
                continue

            if not self._stop.is_set():
                await asyncio.sleep(self._reconnect_seconds)
