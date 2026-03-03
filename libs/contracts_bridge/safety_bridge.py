"""
Safety bridge: maps legacy safe_mode / strong command values to SafetyState dicts.

SSOT (contract):
- docs/contracts/safety_state.schema.json
  - required: state_id, mode, reason, activated_at
  - mode enum: NORMAL / SAFE / EMERGENCY_STOP
  - additionalProperties: false
  - schema_version: integer const 1 (if present)

Design rules:
- Fail-closed: unknown legacy inputs → SAFE.
- Explicit emergency-like legacy inputs (HALT / HALTED / EMERGENCY*) → EMERGENCY_STOP.
- We do NOT emit legacy-only vocab (SAFE_MODE / HALT / DEGRADED) in SafetyState.mode.
  Those belong to separate kill-switch / interlock concepts handled elsewhere.
"""
from __future__ import annotations

import uuid
from datetime import UTC, datetime
from typing import Any, Final

# -----------------------------
# Mapping helpers
# -----------------------------

_MODE_NORMAL: Final[str] = "NORMAL"
_MODE_SAFE: Final[str] = "SAFE"
_MODE_EMERGENCY: Final[str] = "EMERGENCY_STOP"


def _utc_now_iso() -> str:
    # ISO-8601 with timezone info
    return datetime.now(UTC).isoformat()


def _build_reason(prefix: str, legacy_value: str, reason: str) -> str:
    # Always include legacy value so incident forensics can trace origin,
    # without adding extra keys (additionalProperties=false).
    base = (reason or "").strip()
    if not base:
        base = "legacy_bridge"
    return f"{prefix}({legacy_value}): {base}"


def _as_mode_from_legacy_safe_mode(legacy_mode: str) -> str:
    v = (legacy_mode or "").strip()
    if not v:
        return _MODE_SAFE  # fail-closed

    v_up = v.upper()

    # Emergency-like / stop-everything semantics → EMERGENCY_STOP
    if v_up in {"HALT", "HALTED", "EMERGENCY", "EMERGENCY_STOP"}:
        return _MODE_EMERGENCY

    # "SAFE_MODE" / "SAFE" / "DEGRADED" are legacy variants of SAFE
    if v_up in {"SAFE", "SAFE_MODE", "DEGRADED"}:
        return _MODE_SAFE

    if v_up == "NORMAL":
        return _MODE_NORMAL

    return _MODE_SAFE  # fail-closed


def _as_mode_from_legacy_command(command: str) -> str:
    v = (command or "").strip()
    if not v:
        return _MODE_SAFE  # fail-closed

    v_up = v.upper()

    if v_up in {"HALT", "EMERGENCY_STOP", "EMERGENCY"}:
        return _MODE_EMERGENCY

    if v_up in {"SAFE_MODE", "SAFE"}:
        return _MODE_SAFE

    if v_up in {"RESET", "NORMAL"}:
        return _MODE_NORMAL

    return _MODE_SAFE  # fail-closed


def map_legacy_safe_mode_to_safety_state(
    legacy_mode: str,
    reason: str = "legacy_bridge",
    activated_by: str | None = None,
    activated_at: str | None = None,
) -> dict[str, Any]:
    """
    Map a legacy safe_mode value to a SafetyState dict (contract v1).

    Fail-closed:
      - unknown/empty legacy_mode → SAFE
      - emergency-like legacy_mode (HALT/HALTED/EMERGENCY*) → EMERGENCY_STOP

    Args:
        activated_at: ISO-8601 timestamp string; defaults to current UTC time.
    """
    mode = _as_mode_from_legacy_safe_mode(legacy_mode)
    return {
        "schema_version": 1,
        "state_id": str(uuid.uuid4()),
        "mode": mode,
        "reason": _build_reason("legacy_safe_mode", legacy_mode, reason),
        "activated_by": activated_by,
        "activated_at": activated_at or _utc_now_iso(),
    }


def map_legacy_command_to_safety_state(
    command: str,
    reason: str = "legacy_command_bridge",
    activated_by: str | None = None,
    activated_at: str | None = None,
) -> dict[str, Any]:
    """
    Map a legacy strong command to a SafetyState dict (contract v1).

    Fail-closed:
      - unknown/empty command → SAFE
      - emergency-like commands (HALT/EMERGENCY_STOP) → EMERGENCY_STOP
    """
    mode = _as_mode_from_legacy_command(command)
    return {
        "schema_version": 1,
        "state_id": str(uuid.uuid4()),
        "mode": mode,
        "reason": _build_reason("legacy_command", command, reason),
        "activated_by": activated_by,
        "activated_at": activated_at or _utc_now_iso(),
    }
