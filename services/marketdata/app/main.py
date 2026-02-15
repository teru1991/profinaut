from __future__ import annotations

import asyncio
import contextlib
import json
import logging
import os
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from datetime import UTC, datetime
import sys
from pathlib import Path
from typing import Any

import re

from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import JSONResponse

_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.append(str(_REPO_ROOT))

from libs.observability import audit_event, error_envelope, request_id_middleware

logger = logging.getLogger("marketdata")
if not logger.handlers:
    handler = logging.StreamHandler()
    handler.setFormatter(logging.Formatter("%(message)s"))
    logger.addHandler(handler)
logger.setLevel(logging.INFO)
SERVICE_NAME = "marketdata"


_ALLOWED_EXCHANGES = {"gmo"}
_SYMBOL_PATTERN = re.compile(r"^[A-Z0-9_/:.-]{3,32}$")


def _normalize_and_validate_params(exchange: str, symbol: str) -> tuple[str, str]:
    normalized_exchange = (exchange or "gmo").strip().lower()
    normalized_symbol = (symbol or "BTC_JPY").strip().upper()

    if normalized_exchange not in _ALLOWED_EXCHANGES:
        raise HTTPException(
            status_code=400,
            detail={
                "code": "INVALID_EXCHANGE",
                "message": f"Unsupported exchange '{normalized_exchange}'",
                "details": {"allowed_exchanges": sorted(_ALLOWED_EXCHANGES)},
            },
        )

    if not _SYMBOL_PATTERN.match(normalized_symbol):
        raise HTTPException(
            status_code=400,
            detail={
                "code": "INVALID_SYMBOL",
                "message": "Symbol format is invalid",
                "details": {"symbol": normalized_symbol},
            },
        )

    return normalized_exchange, normalized_symbol


@dataclass
class PollerConfig:
    gmo_api_base_url: str = os.getenv("GMO_MARKETDATA_BASE_URL", "https://api.coin.z.com/public/v1")
    symbol: str = os.getenv("MARKETDATA_SYMBOL", "BTC_JPY")
    interval_seconds: float = float(os.getenv("MARKETDATA_POLL_INTERVAL_SECONDS", "2"))
    stale_threshold_seconds: float = float(os.getenv("MARKETDATA_STALE_THRESHOLD_SECONDS", "10"))
    backoff_initial_seconds: float = float(os.getenv("MARKETDATA_BACKOFF_INITIAL_SECONDS", "1"))
    backoff_max_seconds: float = float(os.getenv("MARKETDATA_BACKOFF_MAX_SECONDS", "30"))
    timeout_seconds: float = float(os.getenv("MARKETDATA_HTTP_TIMEOUT_SECONDS", "5"))


@dataclass
class TickerSnapshot:
    symbol: str
    ts: str
    bid: float
    ask: float
    last: float
    source: str


class MarketDataPoller:
    def __init__(self, config: PollerConfig):
        self._config = config
        self._snapshot: TickerSnapshot | None = None
        self._last_success_monotonic: float | None = None
        self._degraded_reason: str | None = None
        self._consecutive_failures: int = 0
        self._current_backoff: float = config.backoff_initial_seconds
        self._state: str = "healthy"
        self._lock = asyncio.Lock()

    def _transition(self, new_state: str, reason: str | None = None) -> None:
        if self._state == new_state:
            return
        audit_event(
            service=SERVICE_NAME,
            event="marketdata_state_transition",
            from_state=self._state,
            to_state=new_state,
            degraded_reason=reason,
        )
        self._state = new_state

    def _apply_reason_state(self, degraded_reason: str | None) -> None:
        if degraded_reason is None:
            self._transition("healthy")
        else:
            self._transition("degraded", reason=degraded_reason)

    def _record_success(self, snapshot: TickerSnapshot) -> None:
        self._snapshot = snapshot
        self._last_success_monotonic = time.monotonic()
        self._consecutive_failures = 0
        self._current_backoff = self._config.backoff_initial_seconds
        self._degraded_reason = None
        self._apply_reason_state(self._degraded_reason)

    def _record_failure(self, exc: Exception) -> float:
        self._consecutive_failures += 1
        self._degraded_reason = "UPSTREAM_ERROR"
        self._apply_reason_state(self._degraded_reason)
        sleep_for = self._current_backoff
        self._current_backoff = min(self._current_backoff * 2, self._config.backoff_max_seconds)
        audit_event(
            service=SERVICE_NAME,
            event="gmo_poll_failure",
            degraded_reason=self._degraded_reason,
            error=str(exc),
            consecutive_failures=self._consecutive_failures,
            backoff_seconds=sleep_for,
        )
        return sleep_for

    def _fetch_gmo_ticker(self) -> TickerSnapshot:
        params = urllib.parse.urlencode({"symbol": self._config.symbol})
        url = f"{self._config.gmo_api_base_url.rstrip('/')}/ticker?{params}"
        req = urllib.request.Request(url, headers={"accept": "application/json"}, method="GET")
        with urllib.request.urlopen(req, timeout=self._config.timeout_seconds) as response:
            payload = json.loads(response.read().decode("utf-8"))

        status = int(payload.get("status", 0))
        if status != 0:
            raise RuntimeError(f"GMO status not ok: {status}")

        data = payload.get("data") or []
        if not data:
            raise RuntimeError("GMO ticker response data is empty")

        item = data[0]
        bid = float(item["bid"])
        ask = float(item["ask"])
        last = float(item.get("last") or item.get("price"))
        ts = item.get("timestamp") or datetime.now(UTC).isoformat()
        return TickerSnapshot(
            symbol=item.get("symbol", self._config.symbol),
            ts=ts,
            bid=bid,
            ask=ask,
            last=last,
            source="gmo",
        )

    async def run_forever(self) -> None:
        while True:
            try:
                snapshot = await asyncio.to_thread(self._fetch_gmo_ticker)
                async with self._lock:
                    self._record_success(snapshot)
                await asyncio.sleep(self._config.interval_seconds)
            except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, ValueError, KeyError, RuntimeError) as exc:
                async with self._lock:
                    sleep_for = self._record_failure(exc)
                await asyncio.sleep(sleep_for)

    def _degraded_payload(
        self,
        *,
        symbol: str,
        reason: str,
        code: str,
        message: str,
    ) -> dict[str, Any]:
        return {
            "symbol": symbol,
            "ts": None,
            "bid": None,
            "ask": None,
            "last": None,
            "mid": None,
            "source": "gmo",
            "quality": {"status": "DEGRADED"},
            "stale": True,
            "degraded_reason": reason,
            "error": {
                "code": code,
                "message": message,
            },
        }

    async def latest_payload(self, symbol: str | None = None) -> tuple[int, dict[str, Any]]:
        requested_symbol = symbol or self._config.symbol
        async with self._lock:
            snapshot = self._snapshot
            if snapshot is None:
                self._degraded_reason = "UPSTREAM_ERROR"
                self._apply_reason_state(self._degraded_reason)
                return 503, self._degraded_payload(
                    symbol=requested_symbol,
                    reason="UPSTREAM_ERROR",
                    code="TICKER_NOT_READY",
                    message="Ticker not ready",
                )

            if requested_symbol != snapshot.symbol:
                return 400, self._degraded_payload(
                    symbol=requested_symbol,
                    reason="UNSUPPORTED_SYMBOL",
                    code="UNSUPPORTED_SYMBOL",
                    message=f"Only {snapshot.symbol} is currently available",
                )

            degraded_reason = self._degraded_reason
            stale = True
            if self._last_success_monotonic is not None:
                last_success_age = time.monotonic() - self._last_success_monotonic
                stale = last_success_age > self._config.stale_threshold_seconds

            if stale:
                degraded_reason = "STALE_TICKER"

            self._apply_reason_state(degraded_reason)

            quality_status = "OK"
            if degraded_reason == "STALE_TICKER":
                quality_status = "STALE"
            elif degraded_reason is not None:
                quality_status = "DEGRADED"

            return 200, {
                "symbol": snapshot.symbol,
                "ts": snapshot.ts,
                "bid": snapshot.bid,
                "ask": snapshot.ask,
                "last": snapshot.last,
                "mid": (snapshot.bid + snapshot.ask) / 2,
                "source": snapshot.source,
                "quality": {"status": quality_status},
                "stale": stale,
                "degraded_reason": degraded_reason,
            }




app = FastAPI(title="profinaut-marketdata", version="0.1.0")
app.add_middleware(request_id_middleware())
_poller = MarketDataPoller(PollerConfig())


@app.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    code = "HTTP_ERROR"
    message = str(exc.detail)
    details: dict[str, object] = {}
    if isinstance(exc.detail, dict):
        code = str(exc.detail.get("code") or code)
        message = str(exc.detail.get("message") or message)
        details = dict(exc.detail.get("details") or {})
    audit_event(service=SERVICE_NAME, event="http_error", request_id=request_id, code=code, message=message)
    return JSONResponse(
        status_code=exc.status_code,
        content=error_envelope(code=code, message=message, details=details, request_id=request_id),
    )


@app.exception_handler(Exception)
async def unhandled_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    audit_event(service=SERVICE_NAME, event="unhandled_exception", request_id=request_id, error=str(exc))
    return JSONResponse(
        status_code=500,
        content=error_envelope(
            code="INTERNAL_ERROR",
            message="Unexpected error",
            details={},
            request_id=request_id,
        ),
    )

@app.on_event("startup")
async def startup() -> None:
    app.state.poller_task = asyncio.create_task(_poller.run_forever())


@app.on_event("shutdown")
async def shutdown() -> None:
    task = getattr(app.state, "poller_task", None)
    if task:
        task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await task


@app.get("/healthz")
def healthz() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/capabilities")
async def get_capabilities() -> dict[str, Any]:
    """Return service capabilities and health status."""
    async with _poller._lock:
        degraded_reason = _poller._degraded_reason
        if _poller._snapshot is not None and _poller._last_success_monotonic is not None:
            last_success_age = time.monotonic() - _poller._last_success_monotonic
            if last_success_age > _poller._config.stale_threshold_seconds:
                degraded_reason = "STALE_TICKER"
        degraded = degraded_reason is not None

    return {
        "service": "marketdata",
        "version": "0.1.0",
        "status": "degraded" if degraded else "ok",
        "features": ["ticker_latest", "gmo_poller"],
        "degraded_reason": degraded_reason,
        "generated_at": datetime.now(UTC).isoformat(),
    }


@app.get("/ticker/latest")
async def ticker_latest(
    request: Request,
    exchange: str = Query(default="gmo"),
    symbol: str = Query(default="BTC_JPY"),
) -> JSONResponse:
    _, normalized_symbol = _normalize_and_validate_params(exchange=exchange, symbol=symbol)
    status_code, payload = await _poller.latest_payload(symbol=normalized_symbol)
    request_id = getattr(request.state, "request_id", "unknown")
    payload["request_id"] = request_id
    payload["exchange"] = "gmo"
    return JSONResponse(status_code=status_code, content=payload)
