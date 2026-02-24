"""
Audit bridge: maps legacy AuditLog dicts to AuditEvent dicts with fail-closed redaction.

Design rules:
- Fail-closed redaction: if forbidden keys/patterns are found in the payload,
  the payload is replaced with {"redaction_failed": True, "forbidden_paths": [...]},
  and type/severity are forced to "SECRET_LEAK_GUARD_TRIGGERED" / "critical".
- schema_version is SemVer string "1.0.0".
"""
from __future__ import annotations

import re
import uuid
import warnings
from datetime import UTC, datetime
from typing import Any

from .redaction import (
    DEFAULT_FORBIDDEN_KEYS,
    DEFAULT_FORBIDDEN_REGEX,
    redact_obj_fail_closed,
)

# Legacy result value → AuditEvent severity
_RESULT_SEVERITY_MAP: dict[str, str] = {
    "ok": "info",
    "success": "info",
    "SUCCESS": "info",
    "error": "error",
    "ERROR": "error",
    "failure": "error",
    "FAILURE": "error",
    "warning": "warning",
    "WARNING": "warning",
}

# Legacy action value → AuditEvent type
_ACTION_TYPE_MAP: dict[str, str] = {
    "HALT": "SAFETY_ACTION",
    "EMERGENCY_STOP": "SAFETY_ACTION",
    "SAFE_MODE": "SAFETY_ACTION",
    "LOGIN": "AUTH_EVENT",
    "LOGOUT": "AUTH_EVENT",
    "CONFIG_CHANGE": "CONFIG_EVENT",
}


def map_legacy_auditlog_to_audit_event(
    legacy_log: dict[str, Any],
    component: str,
    run_id: str,
    forbidden_keys: list[str] | None = None,
    forbidden_regex: list[re.Pattern[str]] | None = None,
) -> dict[str, Any]:
    """
    Map a legacy AuditLog dict to an AuditEvent dict.

    Applies fail-closed redaction to the payload. If any forbidden key or
    pattern is detected, the payload is replaced with a redaction placeholder
    and type/severity are forced to SECRET_LEAK_GUARD_TRIGGERED / critical.

    Legacy log expected fields:
        timestamp, actor, action, target_type, target_id, result, details

    Returns AuditEvent dict with keys:
        schema_version, audit_id, trace_id, run_id, component,
        type, severity, occurred_at_utc, payload
    """
    if forbidden_keys is None:
        forbidden_keys = DEFAULT_FORBIDDEN_KEYS
    if forbidden_regex is None:
        forbidden_regex = DEFAULT_FORBIDDEN_REGEX

    timestamp = legacy_log.get("timestamp") or legacy_log.get("occurred_at")
    if timestamp is None:
        warnings.warn(
            "Legacy audit log is missing 'timestamp'; using current UTC time.",
            stacklevel=2,
        )
        timestamp = datetime.now(UTC).isoformat()

    action = legacy_log.get("action", "UNKNOWN")
    result = legacy_log.get("result", "ok")

    event_type = _ACTION_TYPE_MAP.get(action, "AUDIT_EVENT")
    severity = _RESULT_SEVERITY_MAP.get(result, "info")

    payload: dict[str, Any] = {
        "actor": legacy_log.get("actor", "unknown"),
        "action": action,
        "target_type": legacy_log.get("target_type", "UNKNOWN"),
        "target_id": legacy_log.get("target_id", "UNKNOWN"),
        "result": result,
        "details": legacy_log.get("details", {}),
    }

    redacted_payload, forbidden_paths = redact_obj_fail_closed(
        payload, forbidden_keys, forbidden_regex
    )

    if forbidden_paths:
        event_type = "SECRET_LEAK_GUARD_TRIGGERED"
        severity = "critical"

    # Preserve any correlation/trace identifier from the legacy log.
    trace_id = (
        legacy_log.get("trace_id")
        or legacy_log.get("correlation_id")
        or str(uuid.uuid4())
    )

    return {
        "schema_version": "1.0.0",
        "audit_id": str(uuid.uuid4()),
        "trace_id": trace_id,
        "run_id": run_id,
        "component": component,
        "type": event_type,
        "severity": severity,
        "occurred_at_utc": timestamp,
        "payload": redacted_payload,
    }
