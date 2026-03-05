from datetime import datetime, timedelta, timezone

from app.policy_gate import PolicyGateInput, evaluate_policy_gate


def test_policy_gate_allow_non_live_exchange_order_intent():
    result = evaluate_policy_gate(
        PolicyGateInput(
            action="ORDER_INTENT",
            exchange="binance",
            safe_mode="NORMAL",
            live_enabled=False,
            live_mode="dry_run",
            live_backoff_until_utc=None,
        )
    )

    assert result.decision == "ALLOW"
    assert result.reason_code == "POLICY_ALLOW"


def test_policy_gate_halts_on_safe_mode_before_live_checks():
    result = evaluate_policy_gate(
        PolicyGateInput(
            action="ORDER_INTENT",
            exchange="gmo",
            safe_mode="SAFE_MODE",
            live_enabled=True,
            live_mode="live",
            live_backoff_until_utc=datetime.now(timezone.utc) + timedelta(minutes=5),
            metrics_ok=True,
            clock_ok=True,
            audit_ok=True,
            lease_ok=True,
            deps_ok=True,
        )
    )

    assert result.decision == "BLOCK"
    assert result.reason_code == "SAFE_MODE_BLOCKED"


def test_policy_gate_blocks_live_new_order_when_live_disabled():
    result = evaluate_policy_gate(
        PolicyGateInput(
            action="ORDER_INTENT",
            exchange="gmo",
            safe_mode="NORMAL",
            live_enabled=False,
            live_mode="dry_run",
            live_backoff_until_utc=None,
            metrics_ok=True,
            clock_ok=True,
            audit_ok=True,
            lease_ok=False,
            deps_ok=True,
        )
    )

    assert result.decision == "BLOCK"
    assert result.reason_code == "LIVE_DISABLED"
