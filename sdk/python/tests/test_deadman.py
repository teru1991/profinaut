from datetime import datetime, timedelta, timezone

from profinaut_agent.deadman import DeadmanSwitch


def test_deadman_triggers_after_timeout() -> None:
    switch = DeadmanSwitch(timeout_seconds=5, fallback_action="SAFE_MODE")
    t0 = datetime.now(timezone.utc)

    assert switch.register_failure(t0) is None
    assert switch.register_failure(t0 + timedelta(seconds=4)) is None
    assert switch.register_failure(t0 + timedelta(seconds=5)) == "SAFE_MODE"


def test_deadman_resets_on_success() -> None:
    switch = DeadmanSwitch(timeout_seconds=1, fallback_action="FLATTEN")
    t0 = datetime.now(timezone.utc)

    switch.register_failure(t0)
    switch.register_success()

    assert switch.register_failure(t0 + timedelta(seconds=5)) is None
