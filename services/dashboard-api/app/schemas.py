from datetime import datetime
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field


class HealthResponse(BaseModel):
    status: str
    timestamp: datetime


class CapabilitiesResponse(BaseModel):
    service: str = "dashboard-api"
    version: str
    status: Literal["ok", "degraded"] = "ok"
    features: list[str]
    command_safety_enforce_reason: bool = False
    generated_at: datetime


class StatusComponent(BaseModel):
    name: str
    status: Literal["OK", "DEGRADED", "DOWN"]
    degraded_reason: str | None = None
    last_checked_at: datetime
    latency_ms: int | None = None


class StatusSummaryResponse(BaseModel):
    overall_status: Literal["OK", "DEGRADED", "DOWN"]
    components: list[StatusComponent]


class HeartbeatIn(BaseModel):
    instance_id: str
    bot_id: str
    runtime_mode: str
    exchange: str
    symbol: str
    version: str
    timestamp: datetime
    metadata: dict = Field(default_factory=dict)


class BotOut(BaseModel):
    bot_id: str
    name: str
    strategy_name: str
    instance_id: str | None = None
    runtime_mode: str | None = None
    exchange: str | None = None
    symbol: str | None = None
    status: str | None = None
    state: str = "UNKNOWN"
    degraded: bool = False
    degraded_reason: str | None = None
    last_seen: datetime | None = None
    version: str | None = None

    model_config = ConfigDict(from_attributes=True)


class PaginatedBots(BaseModel):
    page: int
    page_size: int
    total: int
    items: list[BotOut]


class ModuleIn(BaseModel):
    module_id: str
    name: str
    description: str | None = None
    enabled: bool
    execution_mode: str
    schedule_cron: str | None = None
    config: dict
    created_at: datetime
    updated_at: datetime


class ModuleOut(ModuleIn):
    model_config = ConfigDict(from_attributes=True)


class PaginatedModules(BaseModel):
    page: int
    page_size: int
    total: int
    items: list[ModuleOut]


class AuditLogOut(BaseModel):
    audit_id: str
    actor: str
    action: str
    target_type: str
    target_id: str
    result: str
    details: dict
    timestamp: datetime

    model_config = ConfigDict(from_attributes=True)


class PaginatedAuditLogs(BaseModel):
    page: int
    page_size: int
    total: int
    items: list[AuditLogOut]


class ModuleRunOut(BaseModel):
    run_id: str
    module_id: str
    trigger_type: str
    status: str
    started_at: datetime
    ended_at: datetime | None = None
    summary: dict | None = None

    model_config = ConfigDict(from_attributes=True)


class PaginatedModuleRuns(BaseModel):
    page: int
    page_size: int
    total: int
    items: list[ModuleRunOut]




class ModuleRunTriggerIn(BaseModel):
    trigger_type: str = "MANUAL"
    summary: dict | None = None


class ModuleRunStatusUpdateIn(BaseModel):
    status: str
    summary: dict | None = None
    ended_at: datetime | None = None



class ModuleRunStatsResponse(BaseModel):
    generated_at: datetime
    total_runs: int
    active_runs: int
    status_counts: dict


class ModuleRunPerformanceResponse(BaseModel):
    generated_at: datetime
    total_runs: int
    completed_runs: int
    success_rate: float
    avg_duration_seconds: float
    p95_duration_seconds: float

class ModuleRunFailureRateResponse(BaseModel):
    generated_at: datetime
    total_completed: int
    failed_runs: int
    failure_rate: float
    window_size_used: int

class ModuleRunThroughputResponse(BaseModel):
    generated_at: datetime
    window_hours: int
    total_runs: int
    runs_per_hour: float

class ModuleRunActiveAgeResponse(BaseModel):
    generated_at: datetime
    active_runs: int
    oldest_active_seconds: float
    avg_active_seconds: float

class CommandIn(BaseModel):
    type: str = Field(..., min_length=1, max_length=32)
    target_bot_id: str = Field(..., min_length=1, max_length=64)
    payload: dict = Field(default_factory=dict)
    reason: str | None = None
    expires_at: str | None = None
    created_at: datetime | None = None


class AckOut(BaseModel):
    command_id: str
    ok: bool
    reason: str | None = None
    ts: datetime


class CommandOut(BaseModel):
    id: str
    type: str
    target_bot_id: str
    payload: dict
    reason: str | None = None
    expires_at: datetime | None = None
    status: Literal["pending", "applied", "nack"]
    created_at: datetime
    ack: AckOut | None = None


class CommandAckIn(BaseModel):
    ok: bool
    reason: str | None = None
    ts: datetime


class CommandAckOut(BaseModel):
    command_id: str
    status: Literal["pending", "applied", "nack"]
    ack: AckOut


class AlertOut(BaseModel):
    alert_id: str
    source: str
    severity: str
    message: str
    target_type: str
    target_id: str
    status: str
    created_at: datetime
    last_notified_at: datetime | None = None
    metadata_json: dict

    model_config = ConfigDict(from_attributes=True)




class ModuleRunStuckCheckResponse(BaseModel):
    checked_at: datetime
    threshold_seconds: int
    stuck_runs: int
    alerts_created: int

class HeartbeatAlertCheckResponse(BaseModel):
    checked_at: datetime
    stale_threshold_seconds: int
    stale_instances: int
    alerts_created: int


class ResourceIn(BaseModel):
    instance_id: str
    cpu_pct: float
    memory_pct: float
    timestamp: datetime


class ResourceLatestResponse(BaseModel):
    generated_at: datetime
    instance_id: str | None = None
    latest_cpu_pct: float
    latest_memory_pct: float


class ResourceWindowSummaryResponse(BaseModel):
    generated_at: datetime
    window_hours: int
    instance_id: str | None = None
    avg_cpu_pct: float
    max_cpu_pct: float
    avg_memory_pct: float
    max_memory_pct: float
    cpu_samples: int
    memory_samples: int


class IndexIn(BaseModel):
    instance_id: str
    index_name: str
    value: float
    timestamp: datetime


class IndexLatestItem(BaseModel):
    index_name: str
    value: float
    timestamp: datetime


class IndexLatestResponse(BaseModel):
    generated_at: datetime
    items: list[IndexLatestItem]


class MetricIn(BaseModel):
    instance_id: str
    symbol: str
    metric_type: str
    value: float
    timestamp: datetime


class PositionIn(BaseModel):
    instance_id: str
    symbol: str
    net_exposure: float
    gross_exposure: float
    updated_at: datetime


class ExposureBySymbol(BaseModel):
    symbol: str
    net_exposure: float
    gross_exposure: float


class ExposureSummaryResponse(BaseModel):
    generated_at: datetime
    total_net_exposure: float
    total_gross_exposure: float
    key_metrics: dict
    by_symbol: list[ExposureBySymbol]






class ExecutionQualityIn(BaseModel):
    instance_id: str
    symbol: str
    slippage_bps: float
    latency_ms: float
    fill_ratio: float
    timestamp: datetime


class ExecutionQualitySummaryResponse(BaseModel):
    generated_at: datetime
    avg_slippage_bps: float
    avg_latency_ms: float
    avg_fill_ratio: float
    samples: int

class CostIn(BaseModel):
    instance_id: str
    symbol: str
    cost_type: str
    amount: float
    timestamp: datetime

class EquityDrawdownResponse(BaseModel):
    generated_at: datetime
    samples: int
    peak_equity: float
    latest_equity: float
    max_drawdown_abs: float
    max_drawdown_pct: float
    current_drawdown_pct: float

class NetPnlSummaryResponse(BaseModel):
    generated_at: datetime
    realized: float
    unrealized: float
    fees: float
    funding: float
    net_pnl: float

class ReconcileIn(BaseModel):
    instance_id: str
    exchange_equity: float
    internal_equity: float
    difference: float
    status: str
    timestamp: datetime


class ReconcileOut(ReconcileIn):
    reconcile_id: str

    model_config = ConfigDict(from_attributes=True)


class PaginatedReconcileResults(BaseModel):
    page: int
    page_size: int
    total: int
    items: list[ReconcileOut]
