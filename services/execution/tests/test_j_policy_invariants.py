from datetime import datetime, timezone

from app.j_policy_decision import GateInput, decide
from app.j_policy_ssot import JPolicySSOT


def _load_ssot() -> JPolicySSOT:
    from pathlib import Path

    repo_root = Path(__file__).resolve().parents[3]
    return JPolicySSOT.load(repo_root / "docs" / "specs" / "domains" / "J")


def test_invariant_fail_close_on_missing_required_inputs():
    ssot = _load_ssot()
    res = decide(
        ssot,
        GateInput(
            op="live_send_new_order",
            role="oncall",
            current_mode="SAFE",
            safe_mode="NORMAL",
            now_utc=datetime.now(timezone.utc),
        ),
    )
    assert res.decision == "HALT"
    assert res.reason_code in {"J_POLICY_DENY_MISSING_REQUIRED_INPUT", "J_POLICY_DENY_UNKNOWN_INPUT"}


def test_invariant_audit_chain_broken_halts():
    ssot = _load_ssot()
    res = decide(
        ssot,
        GateInput(
            op="live_send_new_order",
            role="oncall",
            current_mode="SAFE",
            safe_mode="NORMAL",
            metrics_ok=True,
            clock_ok=True,
            audit_ok=False,
            lease_ok=True,
            deps_ok=True,
            now_utc=datetime.now(timezone.utc),
        ),
    )
    assert res.decision == "HALT"
    assert res.reason_code == "J_POLICY_HALT_AUDIT_CHAIN_BROKEN"


def test_invariant_cancel_only_allows_cancel_denies_new():
    ssot = _load_ssot()
    r1 = decide(
        ssot,
        GateInput(
            op="live_send_new_order",
            role="oncall",
            current_mode="CANCEL_ONLY",
            safe_mode="NORMAL",
            metrics_ok=True,
            clock_ok=True,
            audit_ok=True,
            lease_ok=True,
            deps_ok=True,
            now_utc=datetime.now(timezone.utc),
        ),
    )
    assert r1.decision == "CANCEL_ONLY"

    r2 = decide(
        ssot,
        GateInput(
            op="live_send_cancel",
            role="oncall",
            current_mode="CANCEL_ONLY",
            safe_mode="NORMAL",
            metrics_ok=True,
            clock_ok=True,
            audit_ok=True,
            lease_ok=True,
            deps_ok=True,
            now_utc=datetime.now(timezone.utc),
        ),
    )
    assert r2.decision == "ALLOW"


def test_invariant_lease_missing_blocks_live_new_order():
    ssot = _load_ssot()
    res = decide(
        ssot,
        GateInput(
            op="live_send_new_order",
            role="oncall",
            current_mode="SAFE",
            safe_mode="NORMAL",
            metrics_ok=True,
            clock_ok=True,
            audit_ok=True,
            lease_ok=False,
            deps_ok=True,
            now_utc=datetime.now(timezone.utc),
        ),
    )
    assert res.decision in {"DENY", "CANCEL_ONLY"}
    assert res.reason_code == "J_POLICY_DENY_LEASE_MISSING"
