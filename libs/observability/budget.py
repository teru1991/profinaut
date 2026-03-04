from __future__ import annotations

import os
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]


@dataclass
class BudgetConfig:
    max_unique_series_per_metric: int = 2000
    max_total_unique_series: int = 20000
    metrics_on_exceed: str = "drop"
    max_event_fields: int = 64
    max_event_bytes: int = 8192
    max_unique_field_keys: int = 512
    logs_on_exceed: str = "truncate"
    degrade_on_budget_exceed: bool = True
    health_reason_code: str = "OBS_BUDGET_EXCEEDED"


@dataclass
class BudgetState:
    metrics_exceeded: bool = False
    logs_exceeded: bool = False
    last_exceeded_at: str | None = None
    metrics_total_unique_series: int = 0


_CFG: BudgetConfig | None = None
_STATE = BudgetState()


def now_utc_iso() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _repo_root() -> Path:
    current = Path(__file__).resolve()
    for parent in [current.parent] + list(current.parents):
        if (parent / "docs").exists() and (parent / "libs").exists():
            return parent
    return Path.cwd()


def is_strict_mode() -> bool:
    return (os.getenv("PROFINAUT_OBS_BUDGET_STRICT") or "").strip() == "1"


def load_budget_config() -> BudgetConfig:
    if tomllib is None:
        return BudgetConfig()

    policy_path = _repo_root() / "docs" / "policy" / "observability_budget.toml"
    if not policy_path.exists():
        return BudgetConfig()

    try:
        data = tomllib.loads(policy_path.read_text(encoding="utf-8"))
        metrics = data.get("metrics", {}) if isinstance(data.get("metrics"), dict) else {}
        logs = data.get("logs", {}) if isinstance(data.get("logs"), dict) else {}
        health = data.get("health", {}) if isinstance(data.get("health"), dict) else {}
        return BudgetConfig(
            max_unique_series_per_metric=int(metrics.get("max_unique_series_per_metric", 2000)),
            max_total_unique_series=int(metrics.get("max_total_unique_series", 20000)),
            metrics_on_exceed=str(metrics.get("on_exceed", "drop")),
            max_event_fields=int(logs.get("max_event_fields", 64)),
            max_event_bytes=int(logs.get("max_event_bytes", 8192)),
            max_unique_field_keys=int(logs.get("max_unique_field_keys", 512)),
            logs_on_exceed=str(logs.get("on_exceed", "truncate")),
            degrade_on_budget_exceed=bool(health.get("degrade_on_budget_exceed", True)),
            health_reason_code=str(health.get("reason_code", "OBS_BUDGET_EXCEEDED")),
        )
    except (OSError, ValueError, TypeError):
        return BudgetConfig()


def cfg() -> BudgetConfig:
    global _CFG
    if _CFG is None:
        _CFG = load_budget_config()
    return _CFG


def state() -> BudgetState:
    return _STATE


def mark_metrics_exceeded() -> None:
    _STATE.metrics_exceeded = True
    if _STATE.last_exceeded_at is None:
        _STATE.last_exceeded_at = now_utc_iso()


def mark_logs_exceeded() -> None:
    _STATE.logs_exceeded = True
    if _STATE.last_exceeded_at is None:
        _STATE.last_exceeded_at = now_utc_iso()


def reset_for_tests() -> None:
    global _CFG
    _CFG = None
    _STATE.metrics_exceeded = False
    _STATE.logs_exceeded = False
    _STATE.last_exceeded_at = None
    _STATE.metrics_total_unique_series = 0
