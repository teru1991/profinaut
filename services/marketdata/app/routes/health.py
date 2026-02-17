from __future__ import annotations

import os
from datetime import UTC, datetime

from fastapi import APIRouter

from services.marketdata.app.metrics import ingest_metrics, quality_gate_metrics
from services.marketdata.app.settings import load_settings

router = APIRouter()


def _build_id() -> str:
    return os.getenv("GIT_SHA") or os.getenv("BUILD_ID") or "unknown"


@router.get("/healthz")
def healthz() -> dict[str, object]:
    settings = load_settings()
    checks = [
        {"name": "object_store", "ok": settings.object_store_backend is not None},
        {"name": "db", "ok": settings.db_dsn is not None},
    ]

    return {
        "status": "degraded" if settings.degraded else "ok",
        "checks": checks,
        "ts": datetime.now(UTC).isoformat(),
    }


@router.get("/capabilities")
def capabilities() -> dict[str, object]:
    settings = load_settings()
    return {
        "ingest_raw_enabled": settings.ingest_raw_enabled,
        "silver_enabled": settings.silver_enabled,
        "storage_backend": settings.object_store_backend,
        "db_enabled": settings.db_dsn is not None,
        "degraded": settings.degraded,
        "degraded_reasons": settings.degraded_reasons,
        "build": {"id": _build_id()},
        "ingest_stats": ingest_metrics.summary(),
        "quality_gate_stats": quality_gate_metrics.summary(),
    }
