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
from typing import Any

from fastapi import FastAPI, HTTPException

logger = logging.getLogger("marketdata")
if not logger.handlers:
    handler = logging.StreamHandler()
    handler.setFormatter(logging.Formatter("%(message)s"))
    logger.addHandler(handler)
logger.setLevel(logging.INFO)


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
        logger.info(
            json.dumps(
                {
                    "event": "marketdata_state_transition",
                    "from_state": self._state,
                    "to_state": new_state,
                    "reason": reason,
                    "ts_utc": datetime.now(UTC).isoformat(),
                },
                ensure_ascii=False,
            )
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
        logger.warning(
            json.dumps(
                {
                    "event": "gmo_poll_failure",
                    "error": str(exc),
                    "consecutive_failures": self._consecutive_failures,
                    "backoff_seconds": sleep_for,
                    "ts_utc": datetime.now(UTC).isoformat(),
                },
                ensure_ascii=False,
            )
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
                snapshot = self._fetch_gmo_ticker()
                async with self._lock:
                    self._record_success(snapshot)
                await asyncio.sleep(self._config.interval_seconds)
            except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, ValueError, KeyError, RuntimeError) as exc:
                async with self._lock:
                    sleep_for = self._record_failure(exc)
                await asyncio.sleep(sleep_for)

    async def latest_payload(self) -> dict[str, Any]:
        async with self._lock:
            snapshot = self._snapshot
            if snapshot is None:
                raise HTTPException(status_code=503, detail="Ticker not ready")

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

            return {
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
_poller = MarketDataPoller(PollerConfig())


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


@app.get("/ticker/latest")
async def ticker_latest() -> dict[str, Any]:
    return await _poller.latest_payload()
