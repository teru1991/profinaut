from __future__ import annotations

from datetime import UTC, datetime

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from libs.safety_core import KillRequest, ScopeKind, SafetyMode, apply_ui_kill, compute_decision
from libs.safety_core.runtime import audit_writer, idempotency_keys, store

router = APIRouter(prefix="/safety", tags=["safety-kill"])


class KillRequestBody(BaseModel):
    requested_mode: SafetyMode
    scope_kind: ScopeKind
    selector: str = Field(min_length=1)
    ttl_seconds: int = Field(gt=0)
    reason: str = Field(min_length=1)
    actor: str = Field(min_length=1)
    idempotency_key: str = Field(min_length=1)
    evidence: dict[str, str]


@router.post("/kill")
def post_safety_kill(req: KillRequestBody) -> dict:
    checks = {"stable_for_seconds": 300, "health_ok": True, "reconcile_ok": True}
    if req.requested_mode == SafetyMode.NORMAL:
        checks = {"stable_for_seconds": 0, "health_ok": False, "reconcile_ok": False}

    request = KillRequest(
        requested_mode=req.requested_mode,
        scope_kind=req.scope_kind,
        selector=req.selector,
        ttl_seconds=req.ttl_seconds,
        reason=req.reason,
        actor=req.actor,
        idempotency_key=req.idempotency_key,
        evidence=req.evidence,
    )
    try:
        state, replay = apply_ui_kill(
            store=store,
            audit=audit_writer,
            request=request,
            idempotency_keys=idempotency_keys,
            now=datetime.now(UTC),
            checks=checks,
        )
    except PermissionError as exc:
        raise HTTPException(status_code=409, detail={"code": "DOWNGRADE_GUARD", "message": str(exc)}) from exc
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc

    decision = compute_decision(store.get_directives(), datetime.now(UTC))
    return {
        "state": state.__dict__,
        "decision": {"mode": decision.mode.value, "latched": decision.latched},
        "idempotent_replay": replay,
    }
