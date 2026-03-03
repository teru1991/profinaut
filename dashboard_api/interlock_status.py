from __future__ import annotations

from datetime import UTC, datetime

from fastapi import APIRouter

from libs.safety_core.runtime import interlock_engine

router = APIRouter(prefix="/safety", tags=["interlock"])


@router.get("/interlocks")
def get_interlock_status() -> dict:
    triggers = interlock_engine.last_triggers()
    return {
        "evaluated_at": datetime.now(UTC).isoformat(),
        "triggers": [t.__dict__ for t in triggers],
    }
