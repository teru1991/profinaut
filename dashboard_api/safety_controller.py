from __future__ import annotations

from datetime import UTC, datetime
from dataclasses import asdict

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from libs.safety_core import (
    ScopeKind,
    SafetyDirective,
    SafetyMode,
    apply_directive,
    compute_decision,
)

from dashboard_api.safety_runtime import audit as _audit
from dashboard_api.safety_runtime import seen_idempotency_keys as _idempotency_keys
from dashboard_api.safety_runtime import store as _store

router = APIRouter(prefix="/safety", tags=["safety"])


class SafetyDirectiveRequest(BaseModel):
    scope_kind: ScopeKind
    selector: str = Field(min_length=1)
    requested_mode: SafetyMode
    ttl_seconds: int = Field(gt=0)
    reason: str = Field(min_length=1)
    actor: str = Field(min_length=1)
    idempotency_key: str = Field(min_length=1)
    evidence: dict[str, str]


@router.post("/directives")
def post_safety_directive(req: SafetyDirectiveRequest) -> dict:
    if req.idempotency_key in _idempotency_keys:
        state = _store.get_current_state()
        decision = compute_decision(_store.get_directives(), datetime.now(UTC))
        return {
            "state": asdict(state) if state else None,
            "decision": {
                "mode": decision.mode.value,
                "latched": decision.latched,
                "sources": [asdict(d) for d in decision.sources],
            },
            "idempotent_replay": True,
        }

    directive = SafetyDirective(
        scope_kind=req.scope_kind,
        selector=req.selector,
        mode=req.requested_mode,
        ttl_seconds=req.ttl_seconds,
        reason=req.reason,
        actor=req.actor,
        evidence=req.evidence,
    )

    checks = {
        "stable_for_seconds": 300,
        "health_ok": True,
        "reconcile_ok": True,
    }
    if directive.mode == SafetyMode.NORMAL:
        checks = {"stable_for_seconds": 0, "health_ok": False, "reconcile_ok": False}

    try:
        state = apply_directive(_store, _audit, directive, datetime.now(UTC), checks)
    except PermissionError as exc:
        raise HTTPException(status_code=409, detail={"code": "DOWNGRADE_GUARD", "message": str(exc)}) from exc
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc

    _idempotency_keys.add(req.idempotency_key)
    decision = compute_decision(_store.get_directives(), datetime.now(UTC))
    return {
        "state": asdict(state),
        "decision": {
            "mode": decision.mode.value,
            "latched": decision.latched,
            "sources": [asdict(d) for d in decision.sources],
        },
        "idempotent_replay": False,
    }


@router.get("/state")
def get_safety_state() -> dict:
    now = datetime.now(UTC)
    _store.expire_directives(now)
    state = _store.get_current_state()
    directives = _store.get_directives()
    decision = compute_decision(directives, now)
    return {
        "state": asdict(state) if state else None,
        "active_directives": [asdict(d) for d in directives],
        "decision": {"mode": decision.mode.value, "latched": decision.latched},
    }
