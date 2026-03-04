from __future__ import annotations

import json
import os
from datetime import UTC, datetime
from typing import Any

from libs.observability.redaction import load_forbidden_keys, sanitize

SCHEMA_VERSION_LOG_EVENT = "obs.log_event.v1"
_REQUIRED_KEYS = [
    "schema_version",
    "ts",
    "level",
    "msg",
    "logger",
    "service",
    "op",
    "run_id",
    "instance_id",
]
_FORBIDDEN_KEYS_CACHE: set[str] | None = None


def now_utc_iso() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def is_strict_mode() -> bool:
    return (os.getenv("PROFINAUT_OBS_LOG_STRICT") or "").strip() == "1"


def forbidden_keys() -> set[str]:
    global _FORBIDDEN_KEYS_CACHE
    if _FORBIDDEN_KEYS_CACHE is None:
        _FORBIDDEN_KEYS_CACHE = load_forbidden_keys()
    return _FORBIDDEN_KEYS_CACHE


def redact_fields(fields: dict[str, Any] | None) -> dict[str, Any] | None:
    if fields is None:
        return None
    sanitized, _violations = sanitize(fields)
    if isinstance(sanitized, dict):
        return sanitized
    return {"sanitized": True}


def validate_required_keys(event: dict[str, Any], strict: bool) -> None:
    missing = [key for key in _REQUIRED_KEYS if key not in event or event.get(key) in (None, "", [])]
    if missing and strict:
        raise ValueError(f"log_event missing required keys: {missing}")


def build_log_event(
    *,
    level: str,
    msg: str,
    logger: str,
    service: str,
    op: str,
    corr: dict[str, Any] | None,
    fields: dict[str, Any] | None = None,
    reason_code: str | None = None,
) -> dict[str, Any]:
    correlation = corr or {}
    sanitized_fields, violations = sanitize(fields or {})
    effective_reason = reason_code
    if violations and not effective_reason:
        effective_reason = "REDACTION_VIOLATION"

    event: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION_LOG_EVENT,
        "ts": now_utc_iso(),
        "level": level,
        "msg": msg,
        "logger": logger,
        "service": service,
        "op": op,
        "run_id": correlation.get("run_id"),
        "instance_id": correlation.get("instance_id"),
        "trace_id": correlation.get("trace_id"),
        "event_uid": correlation.get("event_uid"),
        "reason_code": effective_reason,
        "fields": sanitized_fields if isinstance(sanitized_fields, dict) else {"sanitized": True},
    }
    if violations:
        event["fields"]["redaction_violation_count"] = len(violations)

    for key in list(event.keys()):
        if event[key] is None:
            event.pop(key)

    validate_required_keys(event, strict=is_strict_mode())
    return event


def emit_json(event: dict[str, Any]) -> None:
    print(json.dumps(event, ensure_ascii=False, separators=(",", ":"), sort_keys=True), flush=True)
