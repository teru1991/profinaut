from __future__ import annotations

import pytest

from libs.safety_core.egress_guard import prepare_egress
from libs.safety_core.errors import SecError


def test_egress_denies_llm_target() -> None:
    with pytest.raises(SecError):
        _ = prepare_egress(target="llm", payload={"text": "hello"})


def test_egress_redacts_and_allows_internal() -> None:
    r = prepare_egress(target="internal_api", payload={"Authorization": "Bearer SECRETSECRETSECRET"})
    assert r.allowed
    assert "SECRET" not in r.payload_json
    assert "***" in r.payload_json
