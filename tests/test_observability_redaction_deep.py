from libs.observability.redaction import policy, sanitize


def test_deep_redaction_masks_restricted_keys():
    obj = {
        "authorization": "Bearer abc.def.ghi",
        "nested": {"api_key": "SECRET123"},
        "arr": [{"token": "t"}],
        "ok": "hello",
    }
    sanitized, violations = sanitize(obj)
    assert sanitized["authorization"] == policy().mask_value
    assert sanitized["nested"]["api_key"] == policy().mask_value
    assert sanitized["arr"][0]["token"] == policy().mask_value
    assert sanitized["ok"] == "hello"
    assert len(violations) >= 1


def test_depth_limit_truncates():
    obj: dict[str, object] = {}
    cursor = obj
    for _ in range(20):
        nested: dict[str, object] = {}
        cursor["x"] = nested
        cursor = nested

    sanitized, violations = sanitize(obj)
    assert "TRUNCATED" in str(sanitized) or any(v.get("where") == "depth" for v in violations)
