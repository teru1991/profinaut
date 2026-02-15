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


def test_policy_gate_blocks_safe_mode_before_live_checks():
    result = evaluate_policy_gate(
        PolicyGateInput(
            action="ORDER_INTENT",
            exchange="gmo",
            safe_mode="SAFE_MODE",
            live_enabled=True,
            live_mode="live",
            live_backoff_until_utc=datetime.now(timezone.utc) + timedelta(minutes=5),
        )
    )

    assert result.decision == "BLOCK"
    assert result.reason_code == "SAFE_MODE_BLOCKED"
