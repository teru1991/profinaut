from __future__ import annotations

from datetime import UTC, datetime, timedelta

import pytest

from dashboard_api import safety_runtime
from dashboard_api.safety_lease import LeaseIssueRequest, LeaseRenewRequest, issue_lease, renew_lease_endpoint
from libs.safety_core.lease import ExecutionLease
from libs.safety_core.models import SafetyMode, SafetyStateV1


def _reset_runtime() -> None:
    safety_runtime.seen_idempotency_keys.clear()
    safety_runtime.leases_by_id.clear()
    safety_runtime.lease_subject_index.clear()
    safety_runtime.store.set_current_state(SafetyStateV1(mode=SafetyMode.NORMAL, reason="test-reset", activated_by="tester"))


def test_lease_issue_renew_lifecycle() -> None:
    _reset_runtime()
    issued = issue_lease(
        LeaseIssueRequest(
            subject_kind="BOT",
            subject_id="bot-1",
            scope_kind="GLOBAL",
            selector={"venue": "bybit", "symbol": "BTCUSDT"},
            ttl_seconds=20,
            reason="test",
            actor="tester",
            idempotency_key="ik-1",
            evidence={"trace_id": "trace-1"},
        )
    )
    lease = ExecutionLease(**issued["lease"])
    assert lease.is_valid(datetime.now(UTC))

    renewed = renew_lease_endpoint(
        LeaseRenewRequest(
            lease_id=lease.lease_id,
            ttl_seconds=20,
            actor="tester",
            idempotency_key="ik-2",
            evidence={"trace_id": "trace-2"},
        )
    )
    renewed_lease = ExecutionLease(**renewed["lease"])
    assert renewed_lease.is_valid(datetime.now(UTC))


def test_lease_expired_is_invalid() -> None:
    lease = ExecutionLease(
        subject_kind="BOT",
        subject_id="bot-1",
        scope_kind="GLOBAL",
        selector={"venue": "bybit", "symbol": "BTCUSDT"},
        issued_by="tester",
        reason="test",
        issued_at=(datetime.now(UTC) - timedelta(seconds=30)).isoformat(),
        expires_at=(datetime.now(UTC) - timedelta(seconds=1)).isoformat(),
    )
    assert lease.is_valid(datetime.now(UTC)) is False


def test_emergency_stop_denies_issue_and_renew() -> None:
    _reset_runtime()
    safety_runtime.store.set_current_state(SafetyStateV1(mode=SafetyMode.EMERGENCY_STOP, reason="halt", activated_by="tester"))

    with pytest.raises(Exception):
        issue_lease(
            LeaseIssueRequest(
                subject_kind="BOT",
                subject_id="bot-2",
                scope_kind="GLOBAL",
                selector={"venue": "bybit"},
                ttl_seconds=20,
                reason="test",
                actor="tester",
                idempotency_key="ik-3",
                evidence={"trace_id": "trace-3"},
            )
        )

    with pytest.raises(Exception):
        renew_lease_endpoint(
            LeaseRenewRequest(
                lease_id="missing",
                ttl_seconds=20,
                actor="tester",
                idempotency_key="ik-4",
                evidence={"trace_id": "trace-4"},
            )
        )
