from __future__ import annotations

import uuid
from datetime import datetime

from libs.contracts_bridge.safety_bridge import (
    map_legacy_command_to_safety_state,
    map_legacy_safe_mode_to_safety_state,
)


def _assert_contract_v1_safety_state(obj: dict) -> None:
    # Contract SSOT requires: state_id, mode, reason, activated_at
    # additionalProperties=false -> key set must match exactly (schema_version included)
    expected_keys = {"schema_version", "state_id", "mode", "reason", "activated_by", "activated_at"}
    assert set(obj.keys()) == expected_keys

    assert obj["schema_version"] == 1

    # uuid format
    uuid.UUID(obj["state_id"])

    assert obj["mode"] in {"NORMAL", "SAFE", "EMERGENCY_STOP"}

    assert isinstance(obj["reason"], str)
    assert obj["reason"].strip() != ""

    # activated_at must be ISO8601 parseable
    # datetime.fromisoformat accepts timezone-aware strings in py3.11+
    datetime.fromisoformat(obj["activated_at"])


def test_safe_mode_maps_to_safe_contract_mode() -> None:
    o = map_legacy_safe_mode_to_safety_state("SAFE_MODE", reason="r1")
    _assert_contract_v1_safety_state(o)
    assert o["mode"] == "SAFE"
    # legacy vocab must not leak into mode
    assert o["mode"] not in {"SAFE_MODE", "HALT", "DEGRADED"}


def test_degraded_maps_to_safe() -> None:
    o = map_legacy_safe_mode_to_safety_state("DEGRADED", reason="r2")
    _assert_contract_v1_safety_state(o)
    assert o["mode"] == "SAFE"


def test_halt_like_maps_to_emergency_stop() -> None:
    o1 = map_legacy_safe_mode_to_safety_state("HALT", reason="r3")
    o2 = map_legacy_safe_mode_to_safety_state("HALTED", reason="r4")
    o3 = map_legacy_command_to_safety_state("HALT", reason="r5")
    for o in (o1, o2, o3):
        _assert_contract_v1_safety_state(o)
        assert o["mode"] == "EMERGENCY_STOP"


def test_unknown_is_fail_closed_safe() -> None:
    o = map_legacy_safe_mode_to_safety_state("SOME_UNKNOWN_MODE", reason="r6")
    _assert_contract_v1_safety_state(o)
    assert o["mode"] == "SAFE"

    o2 = map_legacy_command_to_safety_state("SOME_UNKNOWN_COMMAND", reason="r7")
    _assert_contract_v1_safety_state(o2)
    assert o2["mode"] == "SAFE"


def test_normal_maps_to_normal() -> None:
    o = map_legacy_safe_mode_to_safety_state("NORMAL", reason="r8")
    _assert_contract_v1_safety_state(o)
    assert o["mode"] == "NORMAL"

    o2 = map_legacy_command_to_safety_state("RESET", reason="r9")
    _assert_contract_v1_safety_state(o2)
    assert o2["mode"] == "NORMAL"
