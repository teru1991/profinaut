from __future__ import annotations

from dataclasses import asdict
from datetime import UTC, datetime

from fastapi import APIRouter, HTTPException, Query
from pydantic import BaseModel, Field

from dashboard_api.safety_runtime import (
    audit,
    create_or_replace_lease,
    current_mode,
    get_active_lease,
    lease_to_dict,
    leases_by_id,
    renew_lease,
    seen_idempotency_keys,
)
from libs.safety_core.audit import AuditEvent
from libs.safety_core.models import SafetyMode

router = APIRouter(prefix="/safety/lease", tags=["safety-lease"])


class LeaseIssueRequest(BaseModel):
    subject_kind: str = Field(min_length=1)
    subject_id: str = Field(min_length=1)
    scope_kind: str = Field(min_length=1)
    selector: dict[str, str]
    ttl_seconds: int = Field(gt=0, le=120)
    reason: str = Field(min_length=1)
    actor: str = Field(min_length=1)
    idempotency_key: str = Field(min_length=1)
    evidence: dict[str, str]


class LeaseRenewRequest(BaseModel):
    lease_id: str = Field(min_length=1)
    ttl_seconds: int = Field(gt=0, le=120)
    actor: str = Field(min_length=1)
    idempotency_key: str = Field(min_length=1)
    evidence: dict[str, str]


def _validate_evidence(evidence: dict[str, str]) -> None:
    if not any(k in evidence for k in ("trace_id", "run_id", "audit_id")):
        raise HTTPException(status_code=400, detail="evidence requires trace_id/run_id/audit_id")


def _write_audit(event_type: str, actor: str, mode_to: str, reason: str, ttl: int, evidence: dict[str, str]) -> None:
    audit.write_event(
        AuditEvent(
            event_type=event_type,
            actor=actor,
            scope="LEASE",
            mode_from=current_mode().value,
            mode_to=mode_to,
            reason=reason,
            ttl=ttl,
            evidence_ref=evidence,
        )
    )


@router.post("/issue")
def issue_lease(req: LeaseIssueRequest) -> dict:
    _validate_evidence(req.evidence)
    if req.idempotency_key in seen_idempotency_keys:
        active = get_active_lease(req.subject_kind, req.subject_id)
        return {"lease": lease_to_dict(active) if active else None, "idempotent_replay": True}

    mode = current_mode()
    if mode == SafetyMode.EMERGENCY_STOP:
        _write_audit("LEASE_DENIED", req.actor, mode.value, "emergency_stop_active", req.ttl_seconds, req.evidence)
        raise HTTPException(status_code=409, detail={"code": "EMERGENCY_STOP_ACTIVE", "message": "lease denied"})

    lease = create_or_replace_lease(
        subject_kind=req.subject_kind,
        subject_id=req.subject_id,
        scope_kind=req.scope_kind,
        selector=req.selector,
        ttl_seconds=req.ttl_seconds,
        actor=req.actor,
        reason=req.reason,
    )
    seen_idempotency_keys.add(req.idempotency_key)
    _write_audit("LEASE_ISSUED", req.actor, mode.value, req.reason, req.ttl_seconds, req.evidence)
    return {
        "lease": asdict(lease),
        "safety_mode": mode.value,
        "execution_constraint": "CLOSE_ONLY" if mode == SafetyMode.SAFE else "ALLOW",
        "idempotent_replay": False,
    }


@router.post("/renew")
def renew_lease_endpoint(req: LeaseRenewRequest) -> dict:
    _validate_evidence(req.evidence)
    if req.idempotency_key in seen_idempotency_keys:
        lease = leases_by_id.get(req.lease_id)
        return {"lease": lease_to_dict(lease) if lease else None, "idempotent_replay": True}

    mode = current_mode()
    if mode == SafetyMode.EMERGENCY_STOP:
        _write_audit("LEASE_RENEW_DENIED", req.actor, mode.value, "emergency_stop_active", req.ttl_seconds, req.evidence)
        raise HTTPException(status_code=409, detail={"code": "EMERGENCY_STOP_ACTIVE", "message": "renew denied"})

    lease = renew_lease(req.lease_id, req.ttl_seconds)
    if lease is None:
        _write_audit("LEASE_RENEW_FAILED", req.actor, mode.value, "lease_not_found", req.ttl_seconds, req.evidence)
        raise HTTPException(status_code=404, detail={"code": "LEASE_NOT_FOUND", "message": "renew failed"})

    seen_idempotency_keys.add(req.idempotency_key)
    _write_audit("LEASE_RENEWED", req.actor, mode.value, "lease_renewed", req.ttl_seconds, req.evidence)
    return {
        "lease": asdict(lease),
        "safety_mode": mode.value,
        "execution_constraint": "CLOSE_ONLY" if mode == SafetyMode.SAFE else "ALLOW",
        "idempotent_replay": False,
    }


@router.get("/status")
def lease_status(subject_kind: str = Query(...), subject_id: str = Query(...)) -> dict:
    now = datetime.now(UTC)
    lease = get_active_lease(subject_kind, subject_id, now)
    return {"lease": lease_to_dict(lease) if lease else None}
