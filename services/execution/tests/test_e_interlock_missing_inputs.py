from app.e_interlocks import InterlockInputs, decide


def test_interlock_missing_required_inputs_halts():
    decision = decide(InterlockInputs(metrics_ok=None, clock_ok=True, deps_ok=True, lease_ok=True, audit_ok=True))
    assert decision.mode == "HALT"
    assert decision.reason == "missing_required_inputs"
