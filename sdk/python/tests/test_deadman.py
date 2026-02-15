from datetime import datetime, timedelta, timezone

from profinaut_agent.deadman import DeadmanSwitch


def test_deadman_triggers_after_timeout() -> None:
    switch = DeadmanSwitch(stale_seconds=5)
    t0 = datetime.now(timezone.utc)

    assert switch.register_failure(t0, reason_code="HEARTBEAT_FAILURE") is None
    assert switch.register_failure(t0 + timedelta(seconds=4), reason_code="HEARTBEAT_FAILURE") is None
    transition = switch.register_failure(t0 + timedelta(seconds=5), reason_code="HEARTBEAT_FAILURE")
    assert transition is not None
    assert transition.reason_code == "HEARTBEAT_FAILURE"
    assert switch.safe_mode is True


def test_deadman_resets_on_success() -> None:
    switch = DeadmanSwitch(stale_seconds=1)
    t0 = datetime.now(timezone.utc)

    switch.register_failure(t0, reason_code="HEARTBEAT_FAILURE")
    switch.register_success(t0 + timedelta(milliseconds=1))

    assert switch.register_failure(t0 + timedelta(milliseconds=10), reason_code="HEARTBEAT_FAILURE") is None
