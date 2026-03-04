from libs.observability.redaction import forbidden_keys, policy


def test_policy_loads():
    redaction_policy = policy()
    assert redaction_policy.max_depth >= 1
    assert redaction_policy.max_keys >= 10


def test_forbidden_keys_loads_even_if_missing():
    _ = forbidden_keys()
