from datetime import UTC, datetime, timedelta
import json
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

from fastapi import Depends, FastAPI, HTTPException, Query, Request
from fastapi.responses import JSONResponse
from sqlalchemy import func, select
from sqlalchemy.orm import Session

from .auth import require_admin_actor
from .config import get_settings
from .database import get_db
from .models import (
    AlertRecord,
    AuditLog,
    Bot,
    BotStatus,
    CommandAckRecord,
    CommandRecord,
    CostLedgerRecord,
    ExecutionQualityTsRecord,
    Instance,
    MetricTsRecord,
    Module,
    ModuleRun,
    PositionCurrentRecord,
    ReconcileResultRecord,
)
from .notifications import NotificationEvent, NotificationRouter, Severity
from .schemas import (
    CapabilitiesResponse,
    CommandAckIn,
    CommandAckOut,
    CommandIn,
    CommandOut,
    CostIn,
    EquityDrawdownResponse,
    ExecutionQualityIn,
    ExecutionQualitySummaryResponse,
    ExposureSummaryResponse,
    HealthResponse,
    HeartbeatAlertCheckResponse,
    HeartbeatIn,
    IndexIn,
    IndexLatestItem,
    IndexLatestResponse,
    MetricIn,
    ModuleIn,
    ModuleOut,
    ModuleRunActiveAgeResponse,
    ModuleRunFailureRateResponse,
    ModuleRunOut,
    ModuleRunPerformanceResponse,
    ModuleRunStatsResponse,
    ModuleRunStatusUpdateIn,
    ModuleRunStuckCheckResponse,
    ModuleRunThroughputResponse,
    ModuleRunTriggerIn,
    NetPnlSummaryResponse,
    PaginatedAuditLogs,
    PaginatedBots,
    PaginatedModuleRuns,
    PaginatedModules,
    PaginatedReconcileResults,
    PositionIn,
    ReconcileIn,
    ReconcileOut,
    ResourceIn,
    ResourceLatestResponse,
    ResourceWindowSummaryResponse,
)

_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.append(str(_REPO_ROOT))

from libs.observability import audit_event, error_envelope, request_id_middleware

app = FastAPI(title="Profinaut Dashboard API", version="0.4.0")
app.add_middleware(request_id_middleware())
STALE_SECONDS = 120


@app.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    code = "HTTP_ERROR"
    message = "Request failed"
    details: dict[str, object] = {"status_code": exc.status_code}
    if isinstance(exc.detail, dict):
        code = str(exc.detail.get("code") or code)
        message = str(exc.detail.get("message") or message)
        details = dict(exc.detail.get("details") or details)
        details.setdefault("status_code", exc.status_code)

    audit_event(
        service="dashboard-api",
        event="http_error",
        request_id=request_id,
        code=code,
        status_code=exc.status_code,
        exception_type=type(exc).__name__,
    )
    return JSONResponse(
        status_code=exc.status_code,
        content=error_envelope(code=code, message=message, details=details, request_id=request_id),
    )


@app.exception_handler(Exception)
async def unhandled_exception_handler(request: Request, exc: Exception) -> JSONResponse:
    request_id = getattr(request.state, "request_id", "unknown")
    audit_event(
        service="dashboard-api",
        event="unhandled_exception",
        request_id=request_id,
        exception_type=type(exc).__name__,
    )
    return JSONResponse(
        status_code=500,
        content=error_envelope(code="INTERNAL_ERROR", message="Unexpected error", details={}, request_id=request_id),
    )


def write_audit(db: Session, actor: str, action: str, target_type: str, target_id: str, result: str, details: dict) -> None:
    audit_event(
        service="dashboard-api",
        event="audit_log_write",
        actor=actor,
        action=action,
        target_type=target_type,
        target_id=target_id,
        result=result,
        details=details,
    )
    db.add(
        AuditLog(
            actor=actor,
            action=action,
            target_type=target_type,
            target_id=target_id,
            result=result,
            details=details,
            timestamp=datetime.now(UTC),
        )
    )


def _get_request_id(request: Request) -> str:
    return str(getattr(request.state, "request_id", "unknown"))


def _fetch_marketdata_ticker(*, symbol: str, exchange: str, request_id: str) -> tuple[dict, int]:
    settings = get_settings()
    query = urllib.parse.urlencode({"symbol": symbol, "exchange": exchange})
    url = f"{settings.marketdata_base_url.rstrip('/')}/ticker/latest?{query}"
    started = time.perf_counter()
    req = urllib.request.Request(url, headers={"accept": "application/json"}, method="GET")

    try:
        with urllib.request.urlopen(req, timeout=settings.marketdata_timeout_seconds) as response:
            latency_ms = int((time.perf_counter() - started) * 1000)
            body = response.read().decode("utf-8")
            payload = json.loads(body) if body else {}
            return payload, latency_ms
    except TimeoutError as exc:
        latency_ms = int((time.perf_counter() - started) * 1000)
        audit_event(
            service="dashboard-api",
            event="marketdata_proxy_failure",
            request_id=request_id,
            code="UPSTREAM_TIMEOUT",
            upstream_latency_ms=latency_ms,
        )
        raise HTTPException(
            status_code=504,
            detail={
                "code": "UPSTREAM_TIMEOUT",
                "message": "MarketData upstream request timed out",
                "details": {"upstream_latency_ms": latency_ms},
            },
        ) from exc
    except urllib.error.HTTPError as exc:
        latency_ms = int((time.perf_counter() - started) * 1000)
        _ = exc.read()
        message = "MarketData upstream returned an error"
        details = {"upstream_status": exc.code, "upstream_latency_ms": latency_ms}

        audit_event(
            service="dashboard-api",
            event="marketdata_proxy_failure",
            request_id=request_id,
            code="UPSTREAM_HTTP_ERROR",
            upstream_latency_ms=latency_ms,
        )
        raise HTTPException(
            status_code=502,
            detail={"code": "UPSTREAM_HTTP_ERROR", "message": message, "details": details},
        ) from exc
    except (urllib.error.URLError, ValueError) as exc:
        latency_ms = int((time.perf_counter() - started) * 1000)
        audit_event(
            service="dashboard-api",
            event="marketdata_proxy_failure",
            request_id=request_id,
            code="UPSTREAM_UNAVAILABLE",
            upstream_latency_ms=latency_ms,
        )
        raise HTTPException(
            status_code=502,
            detail={
                "code": "UPSTREAM_UNAVAILABLE",
                "message": "MarketData upstream unavailable",
                "details": {"upstream_latency_ms": latency_ms},
            },
        ) from exc


@app.get("/api/markets/ticker/latest")
def get_marketdata_ticker_latest(
    request: Request,
    actor: str = Depends(require_admin_actor),
    symbol: str = Query(default="BTC_JPY"),
    exchange: str = Query(default="gmo"),
) -> dict:
    request_id = _get_request_id(request)
    del actor
    payload, latency_ms = _fetch_marketdata_ticker(symbol=symbol, exchange=exchange, request_id=request_id)
    audit_event(
        service="dashboard-api",
        event="marketdata_proxy_success",
        request_id=request_id,
        upstream_latency_ms=latency_ms,
        degraded_reason=payload.get("degraded_reason") if isinstance(payload, dict) else None,
    )
    return {"request_id": request_id, "data": payload}


@app.get("/healthz", response_model=HealthResponse)
def get_healthz() -> HealthResponse:
    return HealthResponse(status="ok", timestamp=datetime.now(UTC))


@app.get("/capabilities", response_model=CapabilitiesResponse)
def get_capabilities() -> CapabilitiesResponse:
    return CapabilitiesResponse(
        version=app.version,
        status="ok",
        features=["bots", "commands", "portfolio", "analytics"],
        generated_at=datetime.now(UTC),
    )


@app.post("/ingest/heartbeat", status_code=202)
def ingest_heartbeat(payload: HeartbeatIn, db: Session = Depends(get_db)) -> dict:
    bot = db.get(Bot, payload.bot_id)
    if bot is None:
        bot = Bot(bot_id=payload.bot_id, name=payload.bot_id, strategy_name="unknown")
        db.add(bot)

    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        instance = Instance(
            instance_id=payload.instance_id,
            bot_id=payload.bot_id,
            runtime_mode=payload.runtime_mode,
            exchange=payload.exchange,
            symbol=payload.symbol,
            status="RUNNING",
        )
        db.add(instance)
    else:
        instance.runtime_mode = payload.runtime_mode
        instance.exchange = payload.exchange
        instance.symbol = payload.symbol
        instance.status = "RUNNING"

    status_row = db.get(BotStatus, payload.instance_id)
    if status_row is None:
        status_row = BotStatus(
            instance_id=payload.instance_id,
            bot_id=payload.bot_id,
            runtime_mode=payload.runtime_mode,
            exchange=payload.exchange,
            symbol=payload.symbol,
            version=payload.version,
            last_seen=payload.timestamp,
            metadata_json=payload.metadata,
        )
        db.add(status_row)
    else:
        status_row.bot_id = payload.bot_id
        status_row.runtime_mode = payload.runtime_mode
        status_row.exchange = payload.exchange
        status_row.symbol = payload.symbol
        status_row.version = payload.version
        status_row.last_seen = payload.timestamp
        status_row.metadata_json = payload.metadata

    db.commit()
    return {"status": "accepted"}


@app.post("/ingest/metrics", status_code=202)
def ingest_metric(payload: MetricIn, db: Session = Depends(get_db)) -> dict:
    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        raise HTTPException(status_code=404, detail="instance not found")

    db.add(
        MetricTsRecord(
            instance_id=payload.instance_id,
            symbol=payload.symbol,
            metric_type=payload.metric_type,
            value=payload.value,
            timestamp=payload.timestamp,
        )
    )
    db.commit()
    return {"status": "accepted"}


@app.post("/ingest/execution-quality", status_code=202)
def ingest_execution_quality(payload: ExecutionQualityIn, db: Session = Depends(get_db)) -> dict:
    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        raise HTTPException(status_code=404, detail="instance not found")

    db.add(
        ExecutionQualityTsRecord(
            instance_id=payload.instance_id,
            symbol=payload.symbol,
            slippage_bps=payload.slippage_bps,
            latency_ms=payload.latency_ms,
            fill_ratio=payload.fill_ratio,
            timestamp=payload.timestamp,
        )
    )
    db.commit()
    return {"status": "accepted"}






@app.post("/ingest/resource", status_code=202)
def ingest_resource(payload: ResourceIn, db: Session = Depends(get_db)) -> dict:
    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        raise HTTPException(status_code=404, detail="instance not found")

    db.add(
        MetricTsRecord(
            instance_id=payload.instance_id,
            symbol="system",
            metric_type="resource_cpu_pct",
            value=payload.cpu_pct,
            timestamp=payload.timestamp,
        )
    )
    db.add(
        MetricTsRecord(
            instance_id=payload.instance_id,
            symbol="system",
            metric_type="resource_memory_pct",
            value=payload.memory_pct,
            timestamp=payload.timestamp,
        )
    )
    db.commit()
    return {"status": "accepted"}


@app.get("/analytics/resource/latest", response_model=ResourceLatestResponse)
def get_latest_resource(
    actor: str = Depends(require_admin_actor),
    instance_id: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> ResourceLatestResponse:
    del actor
    generated_at = datetime.now(UTC)

    cpu_query = select(MetricTsRecord).where(MetricTsRecord.metric_type == "resource_cpu_pct")
    mem_query = select(MetricTsRecord).where(MetricTsRecord.metric_type == "resource_memory_pct")
    if instance_id:
        cpu_query = cpu_query.where(MetricTsRecord.instance_id == instance_id)
        mem_query = mem_query.where(MetricTsRecord.instance_id == instance_id)

    latest_cpu = db.scalar(cpu_query.order_by(MetricTsRecord.timestamp.desc()).limit(1))
    latest_mem = db.scalar(mem_query.order_by(MetricTsRecord.timestamp.desc()).limit(1))

    resolved_instance_id = instance_id
    if resolved_instance_id is None:
        resolved_instance_id = latest_cpu.instance_id if latest_cpu is not None else (latest_mem.instance_id if latest_mem is not None else None)

    return ResourceLatestResponse(
        generated_at=generated_at,
        instance_id=resolved_instance_id,
        latest_cpu_pct=float(latest_cpu.value) if latest_cpu is not None else 0.0,
        latest_memory_pct=float(latest_mem.value) if latest_mem is not None else 0.0,
    )


@app.get("/analytics/resource/window", response_model=ResourceWindowSummaryResponse)
def get_resource_window_summary(
    actor: str = Depends(require_admin_actor),
    instance_id: str | None = Query(default=None),
    window_hours: int = Query(default=24, ge=1, le=168),
    db: Session = Depends(get_db),
) -> ResourceWindowSummaryResponse:
    del actor
    generated_at = datetime.now(UTC)
    cutoff = generated_at - timedelta(hours=window_hours)

    cpu_query = (
        select(MetricTsRecord)
        .where(MetricTsRecord.metric_type == "resource_cpu_pct")
        .where(MetricTsRecord.timestamp >= cutoff)
    )
    mem_query = (
        select(MetricTsRecord)
        .where(MetricTsRecord.metric_type == "resource_memory_pct")
        .where(MetricTsRecord.timestamp >= cutoff)
    )
    if instance_id:
        cpu_query = cpu_query.where(MetricTsRecord.instance_id == instance_id)
        mem_query = mem_query.where(MetricTsRecord.instance_id == instance_id)

    cpu_rows = db.scalars(cpu_query).all()
    mem_rows = db.scalars(mem_query).all()

    cpu_values = [float(r.value) for r in cpu_rows]
    mem_values = [float(r.value) for r in mem_rows]

    return ResourceWindowSummaryResponse(
        generated_at=generated_at,
        window_hours=window_hours,
        instance_id=instance_id,
        avg_cpu_pct=(sum(cpu_values) / len(cpu_values)) if cpu_values else 0.0,
        max_cpu_pct=max(cpu_values) if cpu_values else 0.0,
        avg_memory_pct=(sum(mem_values) / len(mem_values)) if mem_values else 0.0,
        max_memory_pct=max(mem_values) if mem_values else 0.0,
        cpu_samples=len(cpu_values),
        memory_samples=len(mem_values),
    )

@app.post("/ingest/indices", status_code=202)
def ingest_index(payload: IndexIn, db: Session = Depends(get_db)) -> dict:
    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        raise HTTPException(status_code=404, detail="instance not found")

    db.add(
        MetricTsRecord(
            instance_id=payload.instance_id,
            symbol=payload.index_name,
            metric_type="index",
            value=payload.value,
            timestamp=payload.timestamp,
        )
    )
    db.commit()
    return {"status": "accepted"}


@app.get("/analytics/indices/latest", response_model=IndexLatestResponse)
def get_latest_indices(
    actor: str = Depends(require_admin_actor),
    index_name: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> IndexLatestResponse:
    del actor
    generated_at = datetime.now(UTC)

    query = select(MetricTsRecord).where(MetricTsRecord.metric_type == "index")
    if index_name:
        query = query.where(MetricTsRecord.symbol == index_name)

    rows = db.scalars(query.order_by(MetricTsRecord.timestamp.desc())).all()
    latest_by_index: dict[str, MetricTsRecord] = {}
    for row in rows:
        if row.symbol not in latest_by_index:
            latest_by_index[row.symbol] = row

    items = [
        IndexLatestItem(index_name=k, value=float(v.value), timestamp=v.timestamp)
        for k, v in sorted(latest_by_index.items())
    ]

    return IndexLatestResponse(generated_at=generated_at, items=items)

@app.post("/ingest/costs", status_code=202)
def ingest_cost(payload: CostIn, db: Session = Depends(get_db)) -> dict:
    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        raise HTTPException(status_code=404, detail="instance not found")

    if payload.cost_type not in {"FEE", "FUNDING"}:
        raise HTTPException(status_code=400, detail="cost_type must be FEE or FUNDING")

    db.add(
        CostLedgerRecord(
            instance_id=payload.instance_id,
            symbol=payload.symbol,
            cost_type=payload.cost_type,
            amount=payload.amount,
            timestamp=payload.timestamp,
        )
    )
    db.commit()
    return {"status": "accepted"}


@app.post("/ingest/positions", status_code=202)
def ingest_position(payload: PositionIn, db: Session = Depends(get_db)) -> dict:
    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        raise HTTPException(status_code=404, detail="instance not found")

    existing = db.scalar(
        select(PositionCurrentRecord)
        .where(PositionCurrentRecord.instance_id == payload.instance_id)
        .where(PositionCurrentRecord.symbol == payload.symbol)
    )

    if existing is None:
        db.add(
            PositionCurrentRecord(
                instance_id=payload.instance_id,
                symbol=payload.symbol,
                net_exposure=payload.net_exposure,
                gross_exposure=payload.gross_exposure,
                updated_at=payload.updated_at,
            )
        )
    else:
        existing.net_exposure = payload.net_exposure
        existing.gross_exposure = payload.gross_exposure
        existing.updated_at = payload.updated_at

    db.commit()
    return {"status": "accepted"}


@app.get("/portfolio/exposure", response_model=ExposureSummaryResponse)
def get_portfolio_exposure(actor: str = Depends(require_admin_actor), db: Session = Depends(get_db)) -> ExposureSummaryResponse:
    del actor
    generated_at = datetime.now(UTC)
    rows = db.scalars(select(PositionCurrentRecord)).all()

    by_symbol_map: dict[str, dict[str, float]] = {}
    total_net = 0.0
    total_gross = 0.0
    for row in rows:
        net = float(row.net_exposure)
        gross = float(row.gross_exposure)
        total_net += net
        total_gross += gross

        slot = by_symbol_map.setdefault(row.symbol, {"net_exposure": 0.0, "gross_exposure": 0.0})
        slot["net_exposure"] += net
        slot["gross_exposure"] += gross

    by_symbol = [
        {
            "symbol": symbol,
            "net_exposure": values["net_exposure"],
            "gross_exposure": values["gross_exposure"],
        }
        for symbol, values in sorted(by_symbol_map.items())
    ]

    latest_equity = db.scalar(
        select(MetricTsRecord.value)
        .where(MetricTsRecord.metric_type == "equity")
        .order_by(MetricTsRecord.timestamp.desc())
        .limit(1)
    )

    key_metrics = {
        "latest_equity": float(latest_equity) if latest_equity is not None else 0.0,
        "tracked_positions": len(rows),
        "tracked_symbols": len(by_symbol),
    }

    return ExposureSummaryResponse(
        generated_at=generated_at,
        total_net_exposure=total_net,
        total_gross_exposure=total_gross,
        key_metrics=key_metrics,
        by_symbol=by_symbol,
    )

@app.get("/analytics/equity-drawdown", response_model=EquityDrawdownResponse)
def get_equity_drawdown(
    actor: str = Depends(require_admin_actor),
    symbol: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> EquityDrawdownResponse:
    del actor
    generated_at = datetime.now(UTC)

    query = (
        select(MetricTsRecord)
        .where(MetricTsRecord.metric_type == "equity")
        .order_by(MetricTsRecord.timestamp.asc())
    )
    if symbol:
        query = query.where(MetricTsRecord.symbol == symbol)

    rows = db.scalars(query).all()
    if not rows:
        return EquityDrawdownResponse(
            generated_at=generated_at,
            samples=0,
            peak_equity=0.0,
            latest_equity=0.0,
            max_drawdown_abs=0.0,
            max_drawdown_pct=0.0,
            current_drawdown_pct=0.0,
        )

    peak = float(rows[0].value)
    max_dd_abs = 0.0
    max_dd_pct = 0.0
    latest = float(rows[-1].value)

    for row in rows:
        equity = float(row.value)
        if equity > peak:
            peak = equity
        drawdown_abs = peak - equity
        drawdown_pct = (drawdown_abs / peak) if peak > 0 else 0.0
        if drawdown_abs > max_dd_abs:
            max_dd_abs = drawdown_abs
        if drawdown_pct > max_dd_pct:
            max_dd_pct = drawdown_pct

    current_dd_pct = ((peak - latest) / peak) if peak > 0 else 0.0

    return EquityDrawdownResponse(
        generated_at=generated_at,
        samples=len(rows),
        peak_equity=peak,
        latest_equity=latest,
        max_drawdown_abs=max_dd_abs,
        max_drawdown_pct=max_dd_pct,
        current_drawdown_pct=current_dd_pct,
    )


@app.get("/analytics/execution-quality", response_model=ExecutionQualitySummaryResponse)
def get_execution_quality_summary(
    actor: str = Depends(require_admin_actor),
    symbol: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> ExecutionQualitySummaryResponse:
    del actor
    generated_at = datetime.now(UTC)

    query = select(ExecutionQualityTsRecord)
    if symbol:
        query = query.where(ExecutionQualityTsRecord.symbol == symbol)
    rows = db.scalars(query).all()

    samples = len(rows)
    if samples == 0:
        return ExecutionQualitySummaryResponse(
            generated_at=generated_at,
            avg_slippage_bps=0.0,
            avg_latency_ms=0.0,
            avg_fill_ratio=0.0,
            samples=0,
        )

    avg_slippage = sum(float(r.slippage_bps) for r in rows) / samples
    avg_latency = sum(float(r.latency_ms) for r in rows) / samples
    avg_fill_ratio = sum(float(r.fill_ratio) for r in rows) / samples

    return ExecutionQualitySummaryResponse(
        generated_at=generated_at,
        avg_slippage_bps=avg_slippage,
        avg_latency_ms=avg_latency,
        avg_fill_ratio=avg_fill_ratio,
        samples=samples,
    )


@app.get("/analytics/net-pnl", response_model=NetPnlSummaryResponse)
def get_net_pnl_summary(
    actor: str = Depends(require_admin_actor),
    symbol: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> NetPnlSummaryResponse:
    del actor
    generated_at = datetime.now(UTC)

    def latest_metric(metric_type: str) -> float:
        query = select(MetricTsRecord.value).where(MetricTsRecord.metric_type == metric_type)
        if symbol:
            query = query.where(MetricTsRecord.symbol == symbol)
        value = db.scalar(query.order_by(MetricTsRecord.timestamp.desc()).limit(1))
        return float(value) if value is not None else 0.0

    fees_query = select(func.coalesce(func.sum(CostLedgerRecord.amount), 0)).where(CostLedgerRecord.cost_type == "FEE")
    funding_query = select(func.coalesce(func.sum(CostLedgerRecord.amount), 0)).where(CostLedgerRecord.cost_type == "FUNDING")
    if symbol:
        fees_query = fees_query.where(CostLedgerRecord.symbol == symbol)
        funding_query = funding_query.where(CostLedgerRecord.symbol == symbol)

    realized = latest_metric("realized_pnl")
    unrealized = latest_metric("unrealized_pnl")
    fees = float(db.scalar(fees_query) or 0.0)
    funding = float(db.scalar(funding_query) or 0.0)
    net_pnl = realized + unrealized - fees + funding

    return NetPnlSummaryResponse(
        generated_at=generated_at,
        realized=realized,
        unrealized=unrealized,
        fees=fees,
        funding=funding,
        net_pnl=net_pnl,
    )


@app.get("/bots", response_model=PaginatedBots)
def list_bots(
    actor: str = Depends(require_admin_actor),
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    db: Session = Depends(get_db),
) -> PaginatedBots:
    del actor
    now = datetime.now(UTC)
    total = db.scalar(select(func.count()).select_from(Bot)) or 0
    offset = (page - 1) * page_size

    rows = db.execute(
        select(
            Bot.bot_id,
            Bot.name,
            Bot.strategy_name,
            BotStatus.instance_id,
            BotStatus.runtime_mode,
            BotStatus.exchange,
            BotStatus.symbol,
            Instance.status,
            BotStatus.last_seen,
            BotStatus.version,
        )
        .select_from(Bot)
        .join(BotStatus, Bot.bot_id == BotStatus.bot_id, isouter=True)
        .join(Instance, Instance.instance_id == BotStatus.instance_id, isouter=True)
        .order_by(Bot.bot_id)
        .offset(offset)
        .limit(page_size)
    ).all()

    items = []
    for r in rows:
        last_seen = r.last_seen
        if last_seen is not None and last_seen.tzinfo is None:
            last_seen = last_seen.replace(tzinfo=UTC)

        is_stale = bool(last_seen and (now - last_seen).total_seconds() > STALE_SECONDS)

        items.append(
            {
                "state": r.status or "UNKNOWN",
                "degraded": is_stale,
                "degraded_reason": "STALE_HEARTBEAT" if is_stale else None,
                "bot_id": r.bot_id,
                "name": r.name,
                "strategy_name": r.strategy_name,
                "instance_id": r.instance_id,
                "runtime_mode": r.runtime_mode,
                "exchange": r.exchange,
                "symbol": r.symbol,
                "status": r.status,
                "last_seen": last_seen,
                "version": r.version,
            }
        )

    return PaginatedBots(page=page, page_size=page_size, total=total, items=items)


@app.get("/modules", response_model=PaginatedModules)
def list_modules(
    actor: str = Depends(require_admin_actor),
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    enabled: bool | None = Query(default=None),
    db: Session = Depends(get_db),
) -> PaginatedModules:
    del actor
    base_query = select(Module)
    count_query = select(func.count()).select_from(Module)

    if enabled is not None:
        base_query = base_query.where(Module.enabled == enabled)
        count_query = count_query.where(Module.enabled == enabled)

    total = db.scalar(count_query) or 0
    offset = (page - 1) * page_size
    rows = db.scalars(base_query.order_by(Module.name).offset(offset).limit(page_size)).all()

    return PaginatedModules(page=page, page_size=page_size, total=total, items=rows)


@app.post("/modules", response_model=ModuleOut, status_code=201)
def create_or_update_module(payload: ModuleIn, actor: str = Depends(require_admin_actor), db: Session = Depends(get_db)) -> ModuleOut:
    row = db.get(Module, payload.module_id)
    data = payload.model_dump()

    if row is None:
        row = Module(**data)
        db.add(row)
        action = "MODULE_CREATE"
    else:
        for key, value in data.items():
            setattr(row, key, value)
        action = "MODULE_UPDATE"

    write_audit(db, actor, action, "module", payload.module_id, "SUCCESS", {"enabled": payload.enabled})
    db.commit()
    db.refresh(row)
    return row


@app.get("/modules/{module_id}", response_model=ModuleOut)
def get_module(module_id: str, actor: str = Depends(require_admin_actor), db: Session = Depends(get_db)) -> ModuleOut:
    del actor
    row = db.get(Module, module_id)
    if row is None:
        raise HTTPException(status_code=404, detail="Module not found")
    return row


@app.delete("/modules/{module_id}", status_code=204)
def delete_module(module_id: str, actor: str = Depends(require_admin_actor), db: Session = Depends(get_db)) -> None:
    row = db.get(Module, module_id)
    if row is not None:
        db.delete(row)
        write_audit(db, actor, "MODULE_DELETE", "module", module_id, "SUCCESS", {})
        db.commit()


@app.post("/modules/{module_id}/run", response_model=ModuleRunOut, status_code=202)
def trigger_module_run(
    module_id: str,
    payload: ModuleRunTriggerIn,
    actor: str = Depends(require_admin_actor),
    db: Session = Depends(get_db),
) -> ModuleRunOut:
    module = db.get(Module, module_id)
    if module is None:
        raise HTTPException(status_code=404, detail="Module not found")

    run = ModuleRun(
        run_id=str(__import__("uuid").uuid4()),
        module_id=module_id,
        trigger_type=payload.trigger_type,
        status="QUEUED",
        started_at=datetime.now(UTC),
        ended_at=None,
        summary=payload.summary,
    )
    db.add(run)
    write_audit(
        db,
        actor,
        "MODULE_RUN_TRIGGER",
        "module",
        module_id,
        "SUCCESS",
        {"run_id": run.run_id, "trigger_type": payload.trigger_type},
    )
    db.commit()
    db.refresh(run)
    return run


@app.patch("/module-runs/{run_id}", response_model=ModuleRunOut)
def update_module_run_status(
    run_id: str,
    payload: ModuleRunStatusUpdateIn,
    actor: str = Depends(require_admin_actor),
    db: Session = Depends(get_db),
) -> ModuleRunOut:
    run = db.get(ModuleRun, run_id)
    if run is None:
        raise HTTPException(status_code=404, detail="Module run not found")

    run.status = payload.status
    if payload.summary is not None:
        run.summary = payload.summary
    if payload.ended_at is not None:
        run.ended_at = payload.ended_at
    elif payload.status in {"SUCCEEDED", "FAILED", "CANCELED"}:
        run.ended_at = datetime.now(UTC)

    write_audit(
        db,
        actor,
        "MODULE_RUN_UPDATE",
        "module_run",
        run_id,
        "SUCCESS",
        {"status": payload.status},
    )
    db.commit()
    db.refresh(run)
    return run


@app.post("/commands", response_model=CommandOut, status_code=202)
def create_command(payload: CommandIn, actor: str = Depends(require_admin_actor), db: Session = Depends(get_db)) -> CommandOut:
    if payload.expires_at <= payload.issued_at:
        raise HTTPException(status_code=400, detail="expires_at must be greater than issued_at")

    existing = db.get(CommandRecord, payload.command_id)
    if existing is not None:
        raise HTTPException(status_code=409, detail="command_id already exists")

    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        raise HTTPException(status_code=404, detail="instance not found")

    row = CommandRecord(
        command_id=payload.command_id,
        instance_id=payload.instance_id,
        command_type=payload.command_type,
        issued_at=payload.issued_at,
        expires_at=payload.expires_at,
        payload=payload.payload,
        status="PENDING",
        created_by=actor,
    )
    db.add(row)
    write_audit(db, actor, "COMMAND_CREATE", "command", payload.command_id, "SUCCESS", {"type": payload.command_type})
    db.commit()
    db.refresh(row)
    return row


@app.get("/instances/{instance_id}/commands/pending", response_model=list[CommandOut])
def get_pending_commands(instance_id: str, db: Session = Depends(get_db)) -> list[CommandOut]:
    now = datetime.now(UTC)
    rows = db.scalars(
        select(CommandRecord)
        .where(CommandRecord.instance_id == instance_id)
        .where(CommandRecord.status == "PENDING")
        .where(CommandRecord.expires_at > now)
        .order_by(CommandRecord.issued_at)
    ).all()
    return rows


@app.post("/commands/{command_id}/ack", response_model=CommandAckOut, status_code=202)
def ack_command(command_id: str, payload: CommandAckIn, db: Session = Depends(get_db)) -> CommandAckOut:
    if payload.command_id != command_id:
        raise HTTPException(status_code=400, detail="command_id mismatch")

    command = db.get(CommandRecord, command_id)
    if command is None:
        raise HTTPException(status_code=404, detail="command not found")

    ack = CommandAckRecord(
        command_id=payload.command_id,
        instance_id=payload.instance_id,
        status=payload.status,
        reason=payload.reason,
        timestamp=payload.timestamp,
    )
    db.add(ack)
    command.status = payload.status

    write_audit(
        db,
        actor="agent",
        action="COMMAND_ACK",
        target_type="command",
        target_id=command_id,
        result="SUCCESS",
        details={"ack_status": payload.status, "reason": payload.reason},
    )
    db.commit()
    db.refresh(ack)
    return ack



@app.post("/alerts/module-runs/stuck-check", response_model=ModuleRunStuckCheckResponse)
def check_stuck_module_runs(
    actor: str = Depends(require_admin_actor),
    stale_after_seconds: int = Query(default=900, ge=60, le=86400),
    db: Session = Depends(get_db),
) -> ModuleRunStuckCheckResponse:
    checked_at = datetime.now(UTC)
    cutoff_ts = checked_at.timestamp() - stale_after_seconds

    running_rows = db.scalars(select(ModuleRun).where(ModuleRun.status.in_(["QUEUED", "RUNNING"]))).all()
    stuck_rows = [r for r in running_rows if r.started_at.timestamp() < cutoff_ts]

    settings = get_settings()
    router = NotificationRouter(settings.discord_webhook_url)

    alerts_created = 0
    for row in stuck_rows:
        target_id = row.run_id
        existing = db.scalar(
            select(AlertRecord)
            .where(AlertRecord.source == "module_runs")
            .where(AlertRecord.target_type == "module_run")
            .where(AlertRecord.target_id == target_id)
            .where(AlertRecord.status == "OPEN")
        )
        if existing is not None:
            continue

        alert = AlertRecord(
            source="module_runs",
            severity="WARNING",
            message=f"Module run {row.run_id} appears stuck in status {row.status}",
            target_type="module_run",
            target_id=row.run_id,
            status="OPEN",
            metadata_json={"module_id": row.module_id, "started_at": row.started_at.isoformat(), "status": row.status},
        )
        db.add(alert)
        db.flush()

        sent = router.route(
            NotificationEvent(
                severity=Severity.WARNING,
                title="Module Run Stuck",
                message=alert.message,
                timestamp=checked_at,
                metadata={"run_id": row.run_id, "module_id": row.module_id},
            )
        )
        if sent:
            alert.last_notified_at = checked_at

        write_audit(
            db,
            actor,
            "MODULE_RUN_STUCK_ALERT",
            "module_run",
            row.run_id,
            "SUCCESS",
            {"notified": sent},
        )
        alerts_created += 1

    db.commit()
    return ModuleRunStuckCheckResponse(
        checked_at=checked_at,
        threshold_seconds=stale_after_seconds,
        stuck_runs=len(stuck_rows),
        alerts_created=alerts_created,
    )

@app.post("/alerts/heartbeat-check", response_model=HeartbeatAlertCheckResponse)
def check_heartbeat_alerts(
    actor: str = Depends(require_admin_actor),
    stale_after_seconds: int = Query(default=90, ge=30, le=3600),
    db: Session = Depends(get_db),
) -> HeartbeatAlertCheckResponse:
    checked_at = datetime.now(UTC)
    stale_cutoff = checked_at.timestamp() - stale_after_seconds

    status_rows = db.scalars(select(BotStatus)).all()
    stale_rows = [r for r in status_rows if r.last_seen.timestamp() < stale_cutoff]

    settings = get_settings()
    router = NotificationRouter(settings.discord_webhook_url)

    alerts_created = 0
    for row in stale_rows:
        existing = db.scalar(
            select(AlertRecord)
            .where(AlertRecord.target_type == "instance")
            .where(AlertRecord.target_id == row.instance_id)
            .where(AlertRecord.source == "heartbeat_monitor")
            .where(AlertRecord.status == "OPEN")
        )
        if existing is not None:
            continue

        alert = AlertRecord(
            source="heartbeat_monitor",
            severity=Severity.CRITICAL.value,
            message=f"Heartbeat lost for instance {row.instance_id}",
            target_type="instance",
            target_id=row.instance_id,
            status="OPEN",
            created_at=checked_at,
            metadata_json={"last_seen": row.last_seen.isoformat(), "bot_id": row.bot_id},
        )
        db.add(alert)
        db.flush()

        event = NotificationEvent(
            severity=Severity.CRITICAL,
            title="Heartbeat Lost",
            message=f"Instance {row.instance_id} last seen at {row.last_seen.isoformat()}",
            timestamp=checked_at,
            metadata={"instance_id": row.instance_id, "bot_id": row.bot_id},
        )
        sent = router.route(event)
        if sent:
            alert.last_notified_at = checked_at

        write_audit(
            db,
            actor=actor,
            action="HEARTBEAT_LOSS_ALERT",
            target_type="instance",
            target_id=row.instance_id,
            result="SUCCESS",
            details={"severity": "CRITICAL", "notified": sent},
        )
        alerts_created += 1

    db.commit()
    return HeartbeatAlertCheckResponse(
        checked_at=checked_at,
        stale_threshold_seconds=stale_after_seconds,
        stale_instances=len(stale_rows),
        alerts_created=alerts_created,
    )


@app.post("/reconcile", response_model=ReconcileOut, status_code=202)
def post_reconcile(payload: ReconcileIn, actor: str = Depends(require_admin_actor), db: Session = Depends(get_db)) -> ReconcileOut:
    instance = db.get(Instance, payload.instance_id)
    if instance is None:
        raise HTTPException(status_code=404, detail="instance not found")

    row = ReconcileResultRecord(**payload.model_dump())
    db.add(row)

    notified = False
    if payload.status == "MISMATCH":
        checked_at = datetime.now(UTC)
        alert = AlertRecord(
            source="reconcile",
            severity="WARNING",
            message=(
                f"Reconciliation mismatch for {payload.instance_id}: "
                f"exchange={payload.exchange_equity}, internal={payload.internal_equity}, diff={payload.difference}"
            ),
            target_type="instance",
            target_id=payload.instance_id,
            status="OPEN",
            metadata_json={"difference": payload.difference, "timestamp": payload.timestamp.isoformat()},
        )
        db.add(alert)
        settings = get_settings()
        router = NotificationRouter(settings.discord_webhook_url)
        event = NotificationEvent(
            severity=Severity.WARNING,
            title="Reconciliation Mismatch",
            message=alert.message,
            timestamp=checked_at,
            metadata={"instance_id": payload.instance_id, "difference": payload.difference},
        )
        notified = router.route(event)
        if notified:
            alert.last_notified_at = checked_at

    write_audit(
        db,
        actor,
        "RECONCILE_SUBMIT",
        "reconcile",
        payload.instance_id,
        "SUCCESS",
        {"status": payload.status, "difference": payload.difference, "notified": notified},
    )
    db.commit()
    db.refresh(row)
    return row


@app.get("/reconcile/results", response_model=PaginatedReconcileResults)
def list_reconcile_results(
    actor: str = Depends(require_admin_actor),
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    instance_id: str | None = Query(default=None),
    status: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> PaginatedReconcileResults:
    del actor
    base_query = select(ReconcileResultRecord)
    count_query = select(func.count()).select_from(ReconcileResultRecord)

    if instance_id:
        base_query = base_query.where(ReconcileResultRecord.instance_id == instance_id)
        count_query = count_query.where(ReconcileResultRecord.instance_id == instance_id)
    if status:
        base_query = base_query.where(ReconcileResultRecord.status == status)
        count_query = count_query.where(ReconcileResultRecord.status == status)

    total = db.scalar(count_query) or 0
    offset = (page - 1) * page_size
    rows = db.scalars(base_query.order_by(ReconcileResultRecord.timestamp.desc()).offset(offset).limit(page_size)).all()
    return PaginatedReconcileResults(page=page, page_size=page_size, total=total, items=rows)


@app.get("/audit/logs", response_model=PaginatedAuditLogs)
def list_audit_logs(
    actor: str = Depends(require_admin_actor),
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    db: Session = Depends(get_db),
) -> PaginatedAuditLogs:
    del actor
    total = db.scalar(select(func.count()).select_from(AuditLog)) or 0
    offset = (page - 1) * page_size
    rows = db.scalars(select(AuditLog).order_by(AuditLog.timestamp.desc()).offset(offset).limit(page_size)).all()
    return PaginatedAuditLogs(page=page, page_size=page_size, total=total, items=rows)



@app.post("/module-runs/{run_id}/cancel", response_model=ModuleRunOut)
def cancel_module_run(
    run_id: str,
    actor: str = Depends(require_admin_actor),
    db: Session = Depends(get_db),
) -> ModuleRunOut:
    run = db.get(ModuleRun, run_id)
    if run is None:
        raise HTTPException(status_code=404, detail="Module run not found")

    if run.status in {"SUCCEEDED", "FAILED", "CANCELED"}:
        raise HTTPException(status_code=409, detail="module run already terminal")

    run.status = "CANCELED"
    run.ended_at = datetime.now(UTC)

    write_audit(
        db,
        actor,
        "MODULE_RUN_CANCEL",
        "module_run",
        run_id,
        "SUCCESS",
        {},
    )
    db.commit()
    db.refresh(run)
    return run


@app.get("/module-runs/stats", response_model=ModuleRunStatsResponse)
def get_module_run_stats(
    actor: str = Depends(require_admin_actor),
    module_id: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> ModuleRunStatsResponse:
    del actor
    generated_at = datetime.now(UTC)

    query = select(ModuleRun)
    if module_id:
        query = query.where(ModuleRun.module_id == module_id)
    rows = db.scalars(query).all()

    status_counts: dict[str, int] = {}
    for row in rows:
        status_counts[row.status] = status_counts.get(row.status, 0) + 1

    active_runs = sum(status_counts.get(s, 0) for s in ["QUEUED", "RUNNING"])
    return ModuleRunStatsResponse(
        generated_at=generated_at,
        total_runs=len(rows),
        active_runs=active_runs,
        status_counts=status_counts,
    )



@app.get("/analytics/module-runs/performance", response_model=ModuleRunPerformanceResponse)
def get_module_run_performance(
    actor: str = Depends(require_admin_actor),
    module_id: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> ModuleRunPerformanceResponse:
    del actor
    generated_at = datetime.now(UTC)

    query = select(ModuleRun)
    if module_id:
        query = query.where(ModuleRun.module_id == module_id)
    rows = db.scalars(query).all()

    completed = [r for r in rows if r.ended_at is not None]
    durations = [max(0.0, (r.ended_at - r.started_at).total_seconds()) for r in completed]

    completed_runs = len(completed)
    success_runs = sum(1 for r in completed if r.status == "SUCCEEDED")
    success_rate = (success_runs / completed_runs) if completed_runs > 0 else 0.0

    if durations:
        durations_sorted = sorted(durations)
        avg_duration = sum(durations_sorted) / len(durations_sorted)
        p95_index = max(0, ((95 * len(durations_sorted) + 99) // 100) - 1)
        p95_duration = durations_sorted[p95_index]
    else:
        avg_duration = 0.0
        p95_duration = 0.0

    return ModuleRunPerformanceResponse(
        generated_at=generated_at,
        total_runs=len(rows),
        completed_runs=completed_runs,
        success_rate=success_rate,
        avg_duration_seconds=avg_duration,
        p95_duration_seconds=p95_duration,
    )



@app.get("/analytics/module-runs/failure-rate", response_model=ModuleRunFailureRateResponse)
def get_module_run_failure_rate(
    actor: str = Depends(require_admin_actor),
    module_id: str | None = Query(default=None),
    window_size: int = Query(default=50, ge=1, le=500),
    db: Session = Depends(get_db),
) -> ModuleRunFailureRateResponse:
    del actor
    generated_at = datetime.now(UTC)

    query = select(ModuleRun).where(ModuleRun.ended_at.is_not(None))
    if module_id:
        query = query.where(ModuleRun.module_id == module_id)
    rows = db.scalars(query.order_by(ModuleRun.started_at.desc()).limit(window_size)).all()

    total_completed = len(rows)
    failed_runs = sum(1 for r in rows if r.status == "FAILED")
    failure_rate = (failed_runs / total_completed) if total_completed > 0 else 0.0

    return ModuleRunFailureRateResponse(
        generated_at=generated_at,
        total_completed=total_completed,
        failed_runs=failed_runs,
        failure_rate=failure_rate,
        window_size_used=window_size,
    )



@app.get("/analytics/module-runs/throughput", response_model=ModuleRunThroughputResponse)
def get_module_run_throughput(
    actor: str = Depends(require_admin_actor),
    module_id: str | None = Query(default=None),
    window_hours: int = Query(default=24, ge=1, le=168),
    db: Session = Depends(get_db),
) -> ModuleRunThroughputResponse:
    del actor
    generated_at = datetime.now(UTC)
    cutoff = generated_at - timedelta(hours=window_hours)

    query = select(ModuleRun).where(ModuleRun.started_at >= cutoff)
    if module_id:
        query = query.where(ModuleRun.module_id == module_id)
    rows = db.scalars(query).all()

    total_runs = len(rows)
    runs_per_hour = (total_runs / window_hours) if window_hours > 0 else 0.0

    return ModuleRunThroughputResponse(
        generated_at=generated_at,
        window_hours=window_hours,
        total_runs=total_runs,
        runs_per_hour=runs_per_hour,
    )



@app.get("/analytics/module-runs/active-age", response_model=ModuleRunActiveAgeResponse)
def get_module_run_active_age(
    actor: str = Depends(require_admin_actor),
    module_id: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> ModuleRunActiveAgeResponse:
    del actor
    generated_at = datetime.now(UTC)

    query = select(ModuleRun).where(ModuleRun.status.in_(["QUEUED", "RUNNING"]))
    if module_id:
        query = query.where(ModuleRun.module_id == module_id)
    rows = db.scalars(query).all()

    if not rows:
        return ModuleRunActiveAgeResponse(
            generated_at=generated_at,
            active_runs=0,
            oldest_active_seconds=0.0,
            avg_active_seconds=0.0,
        )

    ages = []
    for r in rows:
        started_at = r.started_at
        if started_at.tzinfo is None:
            started_at = started_at.replace(tzinfo=UTC)
        ages.append(max(0.0, (generated_at - started_at).total_seconds()))
    return ModuleRunActiveAgeResponse(
        generated_at=generated_at,
        active_runs=len(rows),
        oldest_active_seconds=max(ages),
        avg_active_seconds=(sum(ages) / len(ages)),
    )

@app.get("/module-runs", response_model=PaginatedModuleRuns)
def list_module_runs(
    actor: str = Depends(require_admin_actor),
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    module_id: str | None = Query(default=None),
    status: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> PaginatedModuleRuns:
    del actor
    base_query = select(ModuleRun)
    count_query = select(func.count()).select_from(ModuleRun)

    if module_id:
        base_query = base_query.where(ModuleRun.module_id == module_id)
        count_query = count_query.where(ModuleRun.module_id == module_id)
    if status:
        base_query = base_query.where(ModuleRun.status == status)
        count_query = count_query.where(ModuleRun.status == status)

    total = db.scalar(count_query) or 0
    offset = (page - 1) * page_size
    rows = db.scalars(base_query.order_by(ModuleRun.started_at.desc()).offset(offset).limit(page_size)).all()
    return PaginatedModuleRuns(page=page, page_size=page_size, total=total, items=rows)
