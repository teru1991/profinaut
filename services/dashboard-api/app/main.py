from datetime import datetime, timezone

from fastapi import Depends, FastAPI, HTTPException, Query
from sqlalchemy import func, select
from sqlalchemy.orm import Session

from .auth import require_admin_token
from .database import get_db
from .models import AuditLog, Bot, BotStatus, Instance, Module, ModuleRun
from .schemas import (
    HealthResponse,
    HeartbeatIn,
    ModuleIn,
    ModuleOut,
    PaginatedAuditLogs,
    PaginatedBots,
    PaginatedModuleRuns,
    PaginatedModules,
)

app = FastAPI(title="Profinaut Dashboard API", version="0.2.0")


@app.get("/healthz", response_model=HealthResponse)
def get_healthz() -> HealthResponse:
    return HealthResponse(status="ok", timestamp=datetime.now(timezone.utc))


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


@app.get("/bots", response_model=PaginatedBots, dependencies=[Depends(require_admin_token)])
def list_bots(
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    db: Session = Depends(get_db),
) -> PaginatedBots:
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

    items = [
        {
            "bot_id": r.bot_id,
            "name": r.name,
            "strategy_name": r.strategy_name,
            "instance_id": r.instance_id,
            "runtime_mode": r.runtime_mode,
            "exchange": r.exchange,
            "symbol": r.symbol,
            "status": r.status,
            "last_seen": r.last_seen,
            "version": r.version,
        }
        for r in rows
    ]

    return PaginatedBots(page=page, page_size=page_size, total=total, items=items)


@app.get("/modules", response_model=PaginatedModules, dependencies=[Depends(require_admin_token)])
def list_modules(
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    enabled: bool | None = Query(default=None),
    db: Session = Depends(get_db),
) -> PaginatedModules:
    base_query = select(Module)
    count_query = select(func.count()).select_from(Module)

    if enabled is not None:
        base_query = base_query.where(Module.enabled == enabled)
        count_query = count_query.where(Module.enabled == enabled)

    total = db.scalar(count_query) or 0
    offset = (page - 1) * page_size
    rows = db.scalars(base_query.order_by(Module.name).offset(offset).limit(page_size)).all()

    return PaginatedModules(page=page, page_size=page_size, total=total, items=rows)


@app.post("/modules", response_model=ModuleOut, status_code=201, dependencies=[Depends(require_admin_token)])
def create_or_update_module(payload: ModuleIn, db: Session = Depends(get_db)) -> ModuleOut:
    row = db.get(Module, payload.module_id)
    data = payload.model_dump()

    if row is None:
        row = Module(**data)
        db.add(row)
    else:
        for key, value in data.items():
            setattr(row, key, value)

    db.commit()
    db.refresh(row)
    return row


@app.get("/modules/{module_id}", response_model=ModuleOut, dependencies=[Depends(require_admin_token)])
def get_module(module_id: str, db: Session = Depends(get_db)) -> ModuleOut:
    row = db.get(Module, module_id)
    if row is None:
        raise HTTPException(status_code=404, detail="Module not found")
    return row


@app.delete("/modules/{module_id}", status_code=204, dependencies=[Depends(require_admin_token)])
def delete_module(module_id: str, db: Session = Depends(get_db)) -> None:
    row = db.get(Module, module_id)
    if row is not None:
        db.delete(row)
        db.commit()


@app.post("/commands", status_code=202, dependencies=[Depends(require_admin_token)])
def create_command(payload: dict) -> dict:
    return payload


@app.post("/commands/{command_id}/ack", status_code=202)
def ack_command(command_id: str, payload: dict) -> dict:
    return {"command_id": command_id, **payload}


@app.post("/reconcile", status_code=202, dependencies=[Depends(require_admin_token)])
def post_reconcile(payload: dict) -> dict:
    return payload


@app.get("/audit/logs", response_model=PaginatedAuditLogs, dependencies=[Depends(require_admin_token)])
def list_audit_logs(
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    db: Session = Depends(get_db),
) -> PaginatedAuditLogs:
    total = db.scalar(select(func.count()).select_from(AuditLog)) or 0
    offset = (page - 1) * page_size
    rows = db.scalars(select(AuditLog).order_by(AuditLog.timestamp.desc()).offset(offset).limit(page_size)).all()
    return PaginatedAuditLogs(page=page, page_size=page_size, total=total, items=rows)


@app.get("/module-runs", response_model=PaginatedModuleRuns, dependencies=[Depends(require_admin_token)])
def list_module_runs(
    page: int = Query(default=1, ge=1),
    page_size: int = Query(default=50, ge=1, le=200),
    module_id: str | None = Query(default=None),
    status: str | None = Query(default=None),
    db: Session = Depends(get_db),
) -> PaginatedModuleRuns:
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
