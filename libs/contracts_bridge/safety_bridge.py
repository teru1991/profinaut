"""
Safety bridge: maps legacy safe_mode / strong command values to SafetyState dicts.

Design rules:
- Fail-closed: unknown legacy_mode / command → "SAFE_MODE" (safe side).
- schema_version is SemVer string "1.0.0".
"""
from __future__ import annotations

import uuid
from datetime import UTC, datetime
from typing import Any

# Legacy safe_mode value → new SafetyState current_mode
_LEGACY_SAFE_MODE_MAP: dict[str, str] = {
    "NORMAL": "NORMAL",
    "normal": "NORMAL",
    "SAFE": "SAFE_MODE",
    "safe": "SAFE_MODE",
    "SAFE_MODE": "SAFE_MODE",
    "HALTED": "HALT",
    "HALT": "HALT",
    "halted": "HALT",
    "EMERGENCY": "EMERGENCY_STOP",
    "EMERGENCY_STOP": "EMERGENCY_STOP",
}

# Legacy strong command → new SafetyState current_mode
_LEGACY_COMMAND_MAP: dict[str, str] = {
    "HALT": "HALT",
    "EMERGENCY_STOP": "EMERGENCY_STOP",
    "SAFE_MODE": "SAFE_MODE",
    "RESET": "NORMAL",
}


def map_legacy_safe_mode_to_safety_state(
    legacy_mode: str,
    reason: str = "legacy_bridge",
    activated_by: str | None = None,
    activated_at: str | None = None,
) -> dict[str, Any]:
    """
    Map a legacy safe_mode value to a SafetyState dict.

    Fail-closed: unrecognised legacy_mode maps to "SAFE_MODE".

    Args:
        activated_at: ISO-8601 timestamp string; defaults to current UTC time.
    """
    current_mode = _LEGACY_SAFE_MODE_MAP.get(legacy_mode, "SAFE_MODE")

    return {
        "schema_version": "1.0.0",
        "state_id": str(uuid.uuid4()),
        "current_mode": current_mode,
        "reason": reason,
        "activated_by": activated_by,
        "activated_at_utc": activated_at or datetime.now(UTC).isoformat(),
        "source": "legacy_bridge",
        "legacy_mode": legacy_mode,
    }


def map_legacy_command_to_safety_state(
    command: str,
    reason: str = "legacy_command_bridge",
    activated_by: str | None = None,
    activated_at: str | None = None,
) -> dict[str, Any]:
    """
    Map a legacy strong command to a SafetyState dict.

    Fail-closed: unrecognised command maps to "SAFE_MODE".

    Args:
        activated_at: ISO-8601 timestamp string; defaults to current UTC time.
    """
    current_mode = _LEGACY_COMMAND_MAP.get(command, "SAFE_MODE")

    return {
        "schema_version": "1.0.0",
        "state_id": str(uuid.uuid4()),
        "current_mode": current_mode,
        "reason": reason,
        "activated_by": activated_by,
        "activated_at_utc": activated_at or datetime.now(UTC).isoformat(),
        "source": "legacy_bridge",
        "legacy_command": command,
    }
