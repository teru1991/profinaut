from __future__ import annotations

import asyncio
import argparse
import contextlib
import json
import logging
import os
import random
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
import socket
import uvicorn

from fastapi import FastAPI, HTTPException, Query, Request
from fastapi.responses import JSONResponse, PlainTextResponse

_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.append(str(_REPO_ROOT))

from libs.observability import audit_event, error_envelope, request_id_middleware
from services.marketdata.app.bronze_store import BronzeStore, RawMetaRepository
from services.marketdata.app.object_store import build_object_store_from_env
from services.marketdata.app.gmo_ws_connector import GmoPublicWsConnector, GmoWsConfig
from services.marketdata.app.mock_exchange import MockRuntime, MockScenario, build_router as build_mock_router
from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.routes.raw_ingest import ingest_raw_envelope, router as raw_ingest_router
from services.marketdata.app.settings import load_settings

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
    enabled: bool = os.getenv("GMO_REST_ENABLED", "0").strip() == "1"
    gmo_api_base_url: str = os.getenv("GMO_MARKETDATA_BASE_URL", "https://api.coin.z.com/public/v1")
    symbol: str = os.getenv("MARKETDATA_SYMBOL", "BTC_JPY")
    market_id: str = os.getenv("MARKETDATA_MARKET_ID", "spot")
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


@dataclass
class DBHealthConfig:
    database_url: str | None = os.getenv("DATABASE_URL")
    timeout_ms: int = int(os.getenv("DB_PING_TIMEOUT_MS", "500"))


class DBHealthChecker:
    def __init__(self, config: DBHealthConfig):
        self._config = config

    def _resolve_target(self) -> tuple[str, int] | None:
        if not self._config.database_url:
            return None

        parsed = urllib.parse.urlparse(self._config.database_url)
        host = parsed.hostname
        if not host:
            return None

        port = parsed.port
        if port is None:
            if parsed.scheme in {"postgres", "postgresql"}:
                port = 5432
            elif parsed.scheme in {"mysql", "mariadb"}:
                port = 3306
            else:
                port = 5432
        return host, int(port)

    def ping(self) -> tuple[bool, float | None, str | None]:
        target = self._resolve_target()
        if target is None:
            return False, None, "DB_UNREACHABLE"

        started = time.perf_counter()
        try:
            connection = socket.create_connection(target, timeout=self._config.timeout_ms / 1000)
            connection.close()
            latency_ms = (time.perf_counter() - started) * 1000
            return True, latency_ms, None
        except OSError:
            return False, None, "DB_UNREACHABLE"


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

    def _record_failure(self, exc: Exception, reason: str) -> float:
        self._consecutive_failures += 1
        self._degraded_reason = reason
        self._apply_reason_state(self._degraded_reason)
        base = self._current_backoff
        sleep_for = min(base + random.uniform(0, max(base * 0.2, 0.01)), self._config.backoff_max_seconds)
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

    def _request_json(self, endpoint: str, query: dict[str, str]) -> dict[str, Any]:
        params = urllib.parse.urlencode(query)
        url = f"{self._config.gmo_api_base_url.rstrip('/')}/{endpoint}?{params}"
        req = urllib.request.Request(url, headers={"accept": "application/json"}, method="GET")
        with urllib.request.urlopen(req, timeout=self._config.timeout_seconds) as response:
            return json.loads(response.read().decode("utf-8"))

    def _ingest_rest_raw(self, endpoint: str, payload: dict[str, Any]) -> None:
        received_ts = datetime.now(UTC).isoformat().replace("+00:00", "Z")
        ingest_raw_envelope(
            {
                "tenant_id": "marketdata",
                "source_type": "REST_PUBLIC",
                "received_ts": received_ts,
                "payload_json": payload,
                "venue_id": "gmo",
                "market_id": self._config.market_id,
                "stream_name": endpoint,
                "endpoint": f"/{endpoint}",
                "event_ts": None,
                "source_msg_key": None,
            }
        )

    def _fetch_gmo_ticker(self) -> TickerSnapshot:
        payload = self._request_json("ticker", {"symbol": self._config.symbol})
        self._ingest_rest_raw("ticker", payload)

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
        ts = item.get("timestamp") or datetime.now(UTC).isoformat().replace("+00:00", "Z")
        return TickerSnapshot(symbol=item.get("symbol", self._config.symbol), ts=ts, bid=bid, ask=ask, last=last, source="gmo")

    def _fetch_gmo_ohlcv(self) -> None:
        payload = self._request_json("klines", {"symbol": self._config.symbol, "interval": "1min"})
        self._ingest_rest_raw("ohlcv", payload)

    async def run_forever(self) -> None:
        if not self._config.enabled:
            return
        while True:
            try:
                snapshot = await asyncio.to_thread(self._fetch_gmo_ticker)
                await asyncio.to_thread(self._fetch_gmo_ohlcv)
                async with self._lock:
                    self._record_success(snapshot)
                await asyncio.sleep(self._config.interval_seconds)
            except urllib.error.HTTPError as exc:
                reason = "UPSTREAM_RATE_LIMITED" if getattr(exc, "code", None) == 429 else "UPSTREAM_UNREACHABLE"
                async with self._lock:
                    sleep_for = self._record_failure(exc, reason)
                await asyncio.sleep(sleep_for)
            except (urllib.error.URLError, TimeoutError, ValueError, KeyError, RuntimeError) as exc:
                async with self._lock:
                    sleep_for = self._record_failure(exc, "UPSTREAM_UNREACHABLE")
                await asyncio.sleep(sleep_for)

    def _degraded_payload(self, *, symbol: str, reason: str, code: str, message: str) -> dict[str, Any]:
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
            "error": {"code": code, "message": message},
        }

    async def latest_payload(self, symbol: str | None = None) -> tuple[int, dict[str, Any]]:
        requested_symbol = symbol or self._config.symbol
        async with self._lock:
            snapshot = self._snapshot
            if snapshot is None:
                self._degraded_reason = "UPSTREAM_UNREACHABLE"
                self._apply_reason_state(self._degraded_reason)
                return 503, self._degraded_payload(symbol=requested_symbol, reason="UPSTREAM_UNREACHABLE", code="TICKER_NOT_READY", message="Ticker not ready")

            if requested_symbol != snapshot.symbol:
                return 400, self._degraded_payload(symbol=requested_symbol, reason="UNSUPPORTED_SYMBOL", code="UNSUPPORTED_SYMBOL", message=f"Only {snapshot.symbol} is currently available")

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






def _validation_error(request_id: str, *, code: str, message: str, details: dict[str, Any] | None = None) -> JSONResponse:
    return JSONResponse(
        status_code=400,
        content=error_envelope(code=code, message=message, details=details or {}, request_id=request_id),
    )


def _is_valid_tf(tf: str) -> bool:
    return tf in {"1m", "5m", "15m", "1h", "1d", "1min", "5min", "15min", "1hour", "1day"}


def _is_valid_rfc3339(ts: str) -> bool:
    try:
        _ = _parse_rfc3339(ts)
        return True
    except ValueError:
        return False
app = FastAPI(title="profinaut-marketdata", version="0.1.0")
app.add_middleware(request_id_middleware())
app.include_router(raw_ingest_router)
_poller = MarketDataPoller(PollerConfig())
_object_store, _object_store_status = build_object_store_from_env()
_db_checker = DBHealthChecker(DBHealthConfig())
_raw_meta_repo = RawMetaRepository()
_bronze_store: BronzeStore | None = None
if _object_store is not None:
    _bronze_store = BronzeStore(_object_store, _raw_meta_repo)
_gmo_ws_connector = GmoPublicWsConnector(GmoWsConfig())
_mock_runtime = MockRuntime(MockScenario())
app.include_router(build_mock_router(_mock_runtime))



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
    app.state.gmo_ws_task = None
    app.state.mock_stop_event = asyncio.Event()
    app.state.mock_task = None
    if _gmo_ws_connector.enabled:
        app.state.gmo_ws_task = asyncio.create_task(_gmo_ws_connector.run_forever())
    if _mock_runtime.scenario.enabled:
        app.state.mock_task = asyncio.create_task(_mock_runtime.run(app.state.mock_stop_event))


@app.on_event("shutdown")
async def shutdown() -> None:
    task = getattr(app.state, "poller_task", None)
    if task:
        task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await task
    ws_task = getattr(app.state, "gmo_ws_task", None)
    if ws_task:
        await _gmo_ws_connector.stop()
        ws_task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await ws_task
    mock_stop_event = getattr(app.state, "mock_stop_event", None)
    if mock_stop_event is not None:
        mock_stop_event.set()
    mock_task = getattr(app.state, "mock_task", None)
    if mock_task:
        mock_task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await mock_task


async def _db_health_snapshot() -> tuple[bool, float | None, str | None]:
    return await asyncio.to_thread(_db_checker.ping)


@app.get("/healthz")
async def healthz() -> dict[str, Any]:
    db_ok, db_latency_ms, db_reason = await _db_health_snapshot()
    degraded_reasons: list[str] = []
    if db_reason is not None:
        degraded_reasons.append(db_reason)

    payload = {
        "status": "degraded" if degraded_reasons else "ok",
        "db_ok": db_ok,
        "db_latency_ms": db_latency_ms,
        "degraded_reasons": degraded_reasons,
    }
    payload.update(_mock_runtime.health())
    return payload


@app.get("/metrics")
async def metrics() -> PlainTextResponse:
    metric_lines = []
    for key, value in _mock_runtime.metrics().items():
        metric_lines.append(f"# TYPE {key} gauge")
        metric_lines.append(f"{key} {value}")
    body = "\n".join(metric_lines) + "\n"
    return PlainTextResponse(content=body, media_type="text/plain; version=0.0.4")


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

    db_ok, db_latency_ms, db_reason = await _db_health_snapshot()

    degraded_reasons: list[str] = []
    if degraded_reason is not None:
        degraded_reasons.append(degraded_reason)
    if _gmo_ws_connector.degraded_reason is not None:
        degraded_reasons.append(_gmo_ws_connector.degraded_reason)
    degraded_reasons.extend(_object_store_status.degraded_reasons)
    if db_reason is not None:
        degraded_reasons.append(db_reason)

    return {
        "service": "marketdata",
        "version": "0.1.0",
        "status": "degraded" if degraded or bool(_object_store_status.degraded_reasons) or (not db_ok) else "ok",
        "features": ["ticker_latest", "gmo_poller", "gmo_ws_connector"],
        "storage_backend": _object_store_status.backend,
        "db_ok": db_ok,
        "db_latency_ms": db_latency_ms,
        "degraded_reason": degraded_reason,
        "degraded_reasons": degraded_reasons,
        "generated_at": datetime.now(UTC).isoformat(),
    }




def _gold_bad_request(request_id: str, *, code: str, message: str, details: dict[str, Any]) -> JSONResponse:
    return JSONResponse(
        status_code=400,
        content=error_envelope(code=code, message=message, details=details, request_id=request_id),
    )


def _gold_read_unavailable(request_id: str, reason: str) -> JSONResponse:
    return JSONResponse(
        status_code=503,
        content=error_envelope(
            code="READ_MODEL_UNAVAILABLE",
            message="Read model is unavailable",
            details={"reason": reason},
            request_id=request_id,
        ),
    )


def _validate_required_text(name: str, value: str | None) -> str | None:
    if value is None:
        return None
    normalized = value.strip()
    return normalized or None


def _normalize_tf(value: str) -> str | None:
    normalized = value.strip().lower()
    mapping = {
        "1m": "1m",
        "1min": "1m",
        "5m": "5m",
        "5min": "5m",
        "15m": "15m",
        "15min": "15m",
        "1h": "1h",
        "1hour": "1h",
        "1d": "1d",
        "1day": "1d",
    }
    return mapping.get(normalized)


@app.get("/orderbook/bbo/latest")
async def orderbook_bbo_latest(
    request: Request,
    venue_id: str | None = Query(default=None),
    market_id: str | None = Query(default=None),
) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    if not venue_id:
        return _validation_error(request_id, code="MISSING_VENUE_ID", message="venue_id is required")
    if not market_id:
        return _validation_error(request_id, code="MISSING_MARKET_ID", message="market_id is required")
    try:
        repo = _connect_read_repo()
    except RuntimeError as exc:
        return _gold_read_unavailable(request_id, str(exc))

    state = repo.get_orderbook_state(venue_id=venue, market_id=market)
    if state is None:
        return JSONResponse(status_code=200, content={"found": False, "stale": True, "as_of": None, "bid": None, "ask": None, "degraded": False, "reason": None})

    stale_ms = int(os.getenv("LATEST_STALE_MS", "30000"))
    stale, _ = _compute_stale(str(state.get("as_of") or state.get("last_update_ts")), stale_ms / 1000)
    stale_reason = "ORDERBOOK_STATE_STALE" if stale and not state.get("reason") else state.get("reason")
    degraded = bool(state.get("degraded")) or stale
    return JSONResponse(
        status_code=200,
        content={
            "found": True,
            "stale": stale,
            "as_of": state.get("as_of"),
            "bid": None if state.get("bid_px") is None else {"price": state.get("bid_px"), "size": state.get("bid_qty")},
            "ask": None if state.get("ask_px") is None else {"price": state.get("ask_px"), "size": state.get("ask_qty")},
            "degraded": degraded,
            "reason": stale_reason,
        },
    )


@app.get("/orderbook/state")
async def orderbook_state(
    request: Request,
    venue_id: str | None = Query(default=None),
    market_id: str | None = Query(default=None),
) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    if not venue_id:
        return _validation_error(request_id, code="MISSING_VENUE_ID", message="venue_id is required")
    if not market_id:
        return _validation_error(request_id, code="MISSING_MARKET_ID", message="market_id is required")
    try:
        repo = _connect_read_repo()
    except RuntimeError as exc:
        return _gold_read_unavailable(request_id, str(exc))

    state = repo.get_orderbook_state(venue_id=venue, market_id=market)
    if state is None:
        return JSONResponse(status_code=200, content={"found": False, "degraded": False, "reason": None})

    return JSONResponse(
        status_code=200,
        content={
            "found": True,
            "last_seq": state.get("last_seq"),
            "last_update_ts": state.get("last_update_ts"),
            "degraded": bool(state.get("degraded")),
            "reason": state.get("reason"),
        },
    )


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

    provided = [venue_id is not None, market_id is not None, instrument_id is not None]
    if any(provided) and not all(provided):
        return _validation_error(
            request_id,
            code="MISSING_IDENTIFIERS",
            message="venue_id, market_id and instrument_id must be provided together",
        )

    if venue_id and market_id and instrument_id:
        try:
            repo = _connect_read_repo()
        except RuntimeError as exc:
            return _gold_read_unavailable(request_id, str(exc))

        bba = repo.get_latest_best_bid_ask(venue_id=venue, market_id=market, instrument_id=instrument)
        threshold_seconds = float(os.getenv("READ_STALE_THRESHOLD_SECONDS", "10"))
        if bba is not None:
            stale, stale_by_ms = _compute_stale(str(bba["received_ts"]), threshold_seconds)
            return JSONResponse(
                status_code=200,
                content={
                    "found": True,
                    "venue_id": venue,
                    "market_id": market,
                    "instrument_id": instrument,
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

        trade = repo.get_latest_trade(venue_id=venue, market_id=market, instrument_id=instrument)
        if trade is not None:
            stale, stale_by_ms = _compute_stale(str(trade["received_ts"]), threshold_seconds)
            return JSONResponse(
                status_code=200,
                content={
                    "found": True,
                    "venue_id": venue,
                    "market_id": market,
                    "instrument_id": instrument,
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
                "venue_id": venue,
                "market_id": market,
                "instrument_id": instrument,
                "stale": True,
                "stale_by_ms": None,
            },
        )

    _, normalized_symbol = _normalize_and_validate_params(exchange=exchange, symbol=symbol)
    status_code, payload = await _poller.latest_payload(symbol=normalized_symbol)
    payload["request_id"] = request_id
    payload["exchange"] = "gmo"
    return JSONResponse(status_code=status_code, content=payload)


@app.get("/ohlcv/latest")
async def ohlcv_latest(
    request: Request,
    venue_id: str = Query(...),
    market_id: str = Query(...),
    tf: str = Query(default="1m"),
    instrument_id: str | None = Query(default=None),
    timeframe: str | None = Query(default=None),
) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    effective_tf = str(timeframe or tf)
    if not _is_valid_tf(effective_tf):
        return _validation_error(request_id, code="INVALID_TF", message="timeframe is invalid", details={"tf": effective_tf})
    try:
        repo = _connect_read_repo()
    except RuntimeError as exc:
        return JSONResponse(
            status_code=503,
            content={
                "found": False,
                "stale": True,
                "as_of": None,
                "tf": effective_tf,
                "candles": [],
                "error": {"code": "READ_MODEL_UNAVAILABLE", "message": str(exc)},
                "request_id": request_id,
                "degraded": True,
            },
        )

    row = (
        repo.get_latest_ohlcv(venue_id=venue_id, market_id=market_id, instrument_id=instrument_id, timeframe=effective_tf)
        if instrument_id
        else repo.get_latest_ohlcv_by_market(venue_id=venue_id, market_id=market_id, timeframe=effective_tf)
    )
    if row is None:
        return JSONResponse(status_code=200, content={"found": False, "stale": True, "as_of": None, "tf": effective_tf, "candles": []})

    stale_ms = int(os.getenv("LATEST_STALE_MS", "30000"))
    stale, _ = _compute_stale(str(row.get("open_ts")), stale_ms / 1000)
    candle = {
        "open_ts": row.get("open_ts"),
        "open": row.get("open"),
        "high": row.get("high"),
        "low": row.get("low"),
        "close": row.get("close"),
        "volume": row.get("volume"),
        "is_final": row.get("is_final"),
    }
    return JSONResponse(
        status_code=200,
        content={
            "found": True,
            "stale": stale,
            "as_of": row.get("open_ts"),
            "tf": effective_tf,
            "candles": [candle],
        },
    )


@app.get("/ohlcv/range")
async def ohlcv_range(
    request: Request,
    venue_id: str = Query(...),
    market_id: str = Query(...),
    tf: str = Query(default="1m"),
    from_ts: str = Query(..., alias="from"),
    to_ts: str = Query(..., alias="to"),
    limit: int = Query(default=100, ge=1, le=1000),
) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    if not _is_valid_tf(tf):
        return _validation_error(request_id, code="INVALID_TF", message="timeframe is invalid", details={"tf": tf})
    if not _is_valid_rfc3339(from_ts):
        return _validation_error(request_id, code="INVALID_FROM_TS", message="from must be RFC3339", details={"from": from_ts})
    if not _is_valid_rfc3339(to_ts):
        return _validation_error(request_id, code="INVALID_TO_TS", message="to must be RFC3339", details={"to": to_ts})
    if _parse_rfc3339(from_ts) > _parse_rfc3339(to_ts):
        return _validation_error(request_id, code="INVALID_TS_RANGE", message="from must be <= to")
    try:
        repo = _connect_read_repo()
    except RuntimeError as exc:
        return JSONResponse(
            status_code=503,
            content={
                "found": False,
                "stale": True,
                "as_of": None,
                "tf": tf,
                "candles": [],
                "error": {"code": "READ_MODEL_UNAVAILABLE", "message": str(exc)},
                "request_id": request_id,
                "degraded": True,
            },
        )

    rows = repo.get_ohlcv_range(
        venue_id=venue_id,
        market_id=market_id,
        timeframe=tf,
        from_ts=from_ts,
        to_ts=to_ts,
        limit=limit,
    )
    if not rows:
        return JSONResponse(status_code=200, content={"found": False, "stale": True, "as_of": None, "tf": tf, "candles": []})

    as_of = str(rows[-1].get("open_ts"))
    stale_ms = int(os.getenv("LATEST_STALE_MS", "30000"))
    stale, _ = _compute_stale(as_of, stale_ms / 1000)

    candles = [
        {
            "open_ts": row.get("open_ts"),
            "open": row.get("open"),
            "high": row.get("high"),
            "low": row.get("low"),
            "close": row.get("close"),
            "volume": row.get("volume"),
            "is_final": row.get("is_final"),
        }
        for row in rows
    ]
    return JSONResponse(status_code=200, content={"found": True, "stale": stale, "as_of": as_of, "tf": tf, "candles": candles})


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


def _cli_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Marketdata service")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=int(os.getenv("PORT", "18080")))
    parser.add_argument("--config", default="config/collector.toml")
    parser.add_argument("--mock", action="store_true", default=os.getenv("MOCK_ENABLED", "0").strip() == "1")
    parser.add_argument("--mock-gap-every", type=int, default=int(os.getenv("MOCK_GAP_EVERY", "0")))
    parser.add_argument("--mock-disconnect-every", type=int, default=int(os.getenv("MOCK_DISCONNECT_EVERY", "0")))
    parser.add_argument("--mock-silence-ms", type=int, default=int(os.getenv("MOCK_SILENCE_MS", "0")))
    parser.add_argument("--mock-mongo-down-ms", type=int, default=int(os.getenv("MOCK_MONGO_DOWN_MS", "0")))
    parser.add_argument("--mock-binary-rate", type=float, default=float(os.getenv("MOCK_BINARY_RATE", "0.0")))
    return parser


def main() -> int:
    args = _cli_parser().parse_args()
    _mock_runtime.scenario.enabled = bool(args.mock)
    _mock_runtime.scenario.gap_every = int(args.mock_gap_every)
    _mock_runtime.scenario.disconnect_every = int(args.mock_disconnect_every)
    _mock_runtime.scenario.silence_ms = int(args.mock_silence_ms)
    _mock_runtime.scenario.mongo_down_ms = int(args.mock_mongo_down_ms)
    _mock_runtime.scenario.binary_rate = float(args.mock_binary_rate)
    uvicorn.run(app, host=args.host, port=args.port, log_level="warning")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
