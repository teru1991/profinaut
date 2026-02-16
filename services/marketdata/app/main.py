from __future__ import annotations

import asyncio
import contextlib
import json
import logging
import os
import time
import sqlite3
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
from fastapi.responses import JSONResponse, PlainTextResponse

_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.append(str(_REPO_ROOT))

from libs.observability import audit_event, error_envelope, request_id_middleware
from services.marketdata.app.bronze_store import BronzeStore, RawMetaRepository
from services.marketdata.app.object_store import build_object_store_from_env

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

    def _is_stale_due_to_age(self) -> bool:
        """Check if the last successful data fetch is stale based on age threshold."""
        if self._last_success_monotonic is None:
            return True
        last_success_age = time.monotonic() - self._last_success_monotonic
        return last_success_age > self._config.stale_threshold_seconds

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
            stale = self._is_stale_due_to_age()

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




def _parse_rfc3339(ts: str) -> datetime:
    return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC)


def _compute_stale(as_of_ts: str, threshold_seconds: float) -> tuple[bool, int | None]:
    try:
        as_of = _parse_rfc3339(as_of_ts)
    except ValueError:
        return True, None
    stale_by_ms = int((datetime.now(UTC) - as_of).total_seconds() * 1000)
    return stale_by_ms > int(threshold_seconds * 1000), max(stale_by_ms, 0)


def _connect_read_repo() -> MarketDataMetaRepository:
    settings = load_settings()
    if settings.db_dsn is None:
        raise RuntimeError("DB_NOT_CONFIGURED")
    if not settings.db_dsn.startswith("sqlite:///"):
        raise RuntimeError("READ_MODEL_UNAVAILABLE")

    db_path = settings.db_dsn.removeprefix("sqlite:///")
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA foreign_keys = ON")
    return MarketDataMetaRepository(conn)




app = FastAPI(title="profinaut-marketdata", version="0.1.0")
app.add_middleware(request_id_middleware())
app.include_router(health_router)
app.include_router(raw_ingest_router)
_poller = MarketDataPoller(PollerConfig())
_object_store, _object_store_status = build_object_store_from_env()
_raw_meta_repo = RawMetaRepository()
_bronze_store: BronzeStore | None = None
if _object_store is not None:
    _bronze_store = BronzeStore(_object_store, _raw_meta_repo)



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

    degraded_reasons: list[str] = []
    if degraded_reason is not None:
        degraded_reasons.append(degraded_reason)
    degraded_reasons.extend(_object_store_status.degraded_reasons)

    return {
        "service": "marketdata",
        "version": "0.1.0",
        "status": "degraded" if degraded or bool(_object_store_status.degraded_reasons) else "ok",
        "features": ["ticker_latest", "gmo_poller"],
        "storage_backend": _object_store_status.backend,
        "degraded_reason": degraded_reason,
        "degraded_reasons": degraded_reasons,
        "generated_at": datetime.now(UTC).isoformat(),
    }


@app.get("/ticker/latest")
async def ticker_latest(
    request: Request,
    exchange: str = Query(default="gmo"),
    symbol: str = Query(default="BTC_JPY"),
    venue_id: str | None = Query(default=None),
    market_id: str | None = Query(default=None),
    instrument_id: str | None = Query(default=None),
) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")

    if venue_id and market_id and instrument_id:
        try:
            repo = _connect_read_repo()
        except RuntimeError as exc:
            return JSONResponse(
                status_code=503,
                content={
                    "code": "READ_MODEL_UNAVAILABLE",
                    "message": "Read model is unavailable",
                    "reason": str(exc),
                    "request_id": request_id,
                },
            )

        bba = repo.get_latest_best_bid_ask(venue_id=venue_id, market_id=market_id, instrument_id=instrument_id)
        threshold_seconds = float(os.getenv("READ_STALE_THRESHOLD_SECONDS", "10"))
        if bba is not None:
            stale, stale_by_ms = _compute_stale(str(bba["received_ts"]), threshold_seconds)
            return JSONResponse(
                status_code=200,
                content={
                    "found": True,
                    "venue_id": venue_id,
                    "market_id": market_id,
                    "instrument_id": instrument_id,
                    "bid": bba["bid_px"],
                    "ask": bba["ask_px"],
                    "bid_qty": bba["bid_qty"],
                    "ask_qty": bba["ask_qty"],
                    "price": (float(bba["bid_px"]) + float(bba["ask_px"])) / 2,
                    "as_of": bba["received_ts"],
                    "stale": stale,
                    "stale_by_ms": stale_by_ms,
                },
            )

        trade = repo.get_latest_trade(venue_id=venue_id, market_id=market_id, instrument_id=instrument_id)
        if trade is not None:
            stale, stale_by_ms = _compute_stale(str(trade["received_ts"]), threshold_seconds)
            return JSONResponse(
                status_code=200,
                content={
                    "found": True,
                    "venue_id": venue_id,
                    "market_id": market_id,
                    "instrument_id": instrument_id,
                    "price": trade["price"],
                    "bid": None,
                    "ask": None,
                    "as_of": trade["received_ts"],
                    "stale": stale,
                    "stale_by_ms": stale_by_ms,
                },
            )

        return JSONResponse(
            status_code=404,
            content={
                "found": False,
                "venue_id": venue_id,
                "market_id": market_id,
                "instrument_id": instrument_id,
                "stale": True,
                "stale_by_ms": None,
            },
        )

    _, normalized_symbol = _normalize_and_validate_params(exchange=exchange, symbol=symbol)
    status_code, payload = await _poller.latest_payload(symbol=normalized_symbol)
    payload["request_id"] = request_id
    payload["exchange"] = "gmo"
    return JSONResponse(status_code=status_code, content=payload)


def _raw_dependency_unavailable(request_id: str) -> JSONResponse:
    return JSONResponse(
        status_code=503,
        content=error_envelope(
            code="RAW_DEPENDENCY_UNAVAILABLE",
            message="Raw metadata/object dependencies are unavailable",
            details={"storage_backend": _object_store_status.backend},
            request_id=request_id,
        ),
    )


@app.get("/raw/meta/{raw_msg_id}")
async def get_raw_meta(raw_msg_id: str, request: Request) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")

    if _bronze_store is None:
        return _raw_dependency_unavailable(request_id)

    meta = _raw_meta_repo.get(raw_msg_id)
    if meta is None:
        return JSONResponse(
            status_code=404,
            content=error_envelope(
                code="RAW_META_NOT_FOUND",
                message=f"raw_msg_id not found: {raw_msg_id}",
                details={"raw_msg_id": raw_msg_id},
                request_id=request_id,
            ),
        )

    return JSONResponse(
        status_code=200,
        content={
            "raw_msg_id": meta.raw_msg_id,
            "object_key": meta.object_key,
            "payload_hash": meta.payload_hash,
            "received_ts": meta.received_ts,
            "quality_json": meta.quality_json,
            "content_encoding": meta.content_encoding,
            "content_type": meta.content_type,
            "object_size": meta.object_size,
            "request_id": request_id,
        },
    )


@app.get("/raw/download/{raw_msg_id}")
async def download_raw(raw_msg_id: str, request: Request):
    request_id = getattr(request.state, "request_id", "unknown")

    if _bronze_store is None:
        return _raw_dependency_unavailable(request_id)

    meta = _raw_meta_repo.get(raw_msg_id)
    if meta is None:
        return JSONResponse(
            status_code=404,
            content=error_envelope(
                code="RAW_META_NOT_FOUND",
                message=f"raw_msg_id not found: {raw_msg_id}",
                details={"raw_msg_id": raw_msg_id},
                request_id=request_id,
            ),
        )

    max_bytes = int(os.getenv("RAW_DOWNLOAD_MAX_BYTES", "1048576"))
    if meta.object_size is not None and meta.object_size > max_bytes:
        return JSONResponse(
            status_code=413,
            content=error_envelope(
                code="RAW_DOWNLOAD_TOO_LARGE",
                message="Raw object exceeds size limit",
                details={"raw_msg_id": raw_msg_id, "object_size": meta.object_size, "max_bytes": max_bytes},
                request_id=request_id,
            ),
        )

    payloads = _bronze_store.replay_payload(meta.object_key)
    ndjson = "".join(json.dumps(item, separators=(",", ":"), sort_keys=True) + "\n" for item in payloads)
    headers = {"x-request-id": request_id}
    return PlainTextResponse(content=ndjson, media_type="application/x-ndjson", headers=headers)
