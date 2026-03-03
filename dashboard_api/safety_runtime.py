from __future__ import annotations

from datetime import UTC, datetime, timedelta
from dataclasses import asdict

from libs.safety_core import InMemorySafetyStore, JsonlAuditWriter, SafetyMode, compute_decision
from libs.safety_core.lease import ExecutionLease

store = InMemorySafetyStore()
audit = JsonlAuditWriter()
seen_idempotency_keys: set[str] = set()
leases_by_id: dict[str, ExecutionLease] = {}
lease_subject_index: dict[tuple[str, str], str] = {}


def current_mode() -> SafetyMode:
    state = store.get_current_state()
    if state is not None:
        return state.mode
    decision = compute_decision(store.get_directives(), datetime.now(UTC))
    return decision.mode


def get_active_lease(subject_kind: str, subject_id: str, now: datetime | None = None) -> ExecutionLease | None:
    now = now or datetime.now(UTC)
    lease_id = lease_subject_index.get((subject_kind, subject_id))
    if lease_id is None:
        return None
    lease = leases_by_id.get(lease_id)
    if lease is None or not lease.is_valid(now):
        return None
    return lease


def lease_to_dict(lease: ExecutionLease) -> dict:
    return asdict(lease)


def create_or_replace_lease(
    subject_kind: str,
    subject_id: str,
    scope_kind: str,
    selector: dict[str, str],
    ttl_seconds: int,
    actor: str,
    reason: str,
) -> ExecutionLease:
    issued_at = datetime.now(UTC)
    lease = ExecutionLease(
        subject_kind=subject_kind,
        subject_id=subject_id,
        scope_kind=scope_kind,
        selector=selector,
        issued_by=actor,
        reason=reason,
        issued_at=issued_at.isoformat(),
        expires_at=(issued_at + timedelta(seconds=ttl_seconds)).isoformat(),
    )
    leases_by_id[lease.lease_id] = lease
    lease_subject_index[(subject_kind, subject_id)] = lease.lease_id
    return lease


def renew_lease(lease_id: str, ttl_seconds: int) -> ExecutionLease | None:
    existing = leases_by_id.get(lease_id)
    if existing is None:
        return None
    issued_at = datetime.now(UTC)
    lease = ExecutionLease(
        lease_id=existing.lease_id,
        subject_kind=existing.subject_kind,
        subject_id=existing.subject_id,
        scope_kind=existing.scope_kind,
        selector=existing.selector,
        issued_by=existing.issued_by,
        reason=existing.reason,
        issued_at=issued_at.isoformat(),
        expires_at=(issued_at + timedelta(seconds=ttl_seconds)).isoformat(),
    )
    leases_by_id[lease.lease_id] = lease
    lease_subject_index[(lease.subject_kind, lease.subject_id)] = lease.lease_id
    return lease
