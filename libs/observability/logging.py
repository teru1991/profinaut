from __future__ import annotations

import json
import os
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from libs.observability import budget
from libs.observability.audit import emit_audit_event

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]

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
_SEEN_FIELD_KEYS: set[str] = set()


def now_utc_iso() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def is_strict_mode() -> bool:
    return (os.getenv("PROFINAUT_OBS_LOG_STRICT") or "").strip() == "1"


def _repo_root() -> Path:
    current = Path(__file__).resolve()
    for parent in [current.parent] + list(current.parents):
        if (parent / "docs").exists() and (parent / "services").exists():
            return parent
    return Path.cwd()


def load_forbidden_keys() -> set[str]:
    if tomllib is None:
        return set()

    path = _repo_root() / "docs" / "policy" / "forbidden_keys.toml"
    if not path.exists():
        return set()

    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return set()

    keys: list[str] | None = None
    if isinstance(data.get("keys"), dict):
        key_list = data["keys"].get("list")
        if isinstance(key_list, list):
            keys = [str(item) for item in key_list]
    if keys is None and isinstance(data.get("forbidden"), dict):
        key_list = data["forbidden"].get("keys")
        if isinstance(key_list, list):
            keys = [str(item) for item in key_list]

    if not keys:
        return set()
    return {key.strip().lower() for key in keys if key.strip()}


def forbidden_keys() -> set[str]:
    global _FORBIDDEN_KEYS_CACHE
    if _FORBIDDEN_KEYS_CACHE is None:
        _FORBIDDEN_KEYS_CACHE = load_forbidden_keys()
    return _FORBIDDEN_KEYS_CACHE


def redact_fields(fields: dict[str, Any] | None) -> dict[str, Any] | None:
    if fields is None:
        return None
    blocked = forbidden_keys()
    if not blocked:
        return dict(fields)

    redacted: dict[str, Any] = {}
    for key, value in fields.items():
        if str(key).lower() in blocked:
            redacted[key] = "***"
        else:
            redacted[key] = value
    return redacted


def _apply_log_budget(fields: dict[str, Any] | None) -> dict[str, Any] | None:
    if fields is None:
        return None

    cfg = budget.cfg()
    sanitized = dict(fields)

    if len(sanitized) > cfg.max_event_fields:
        budget.mark_logs_exceeded()
        emit_audit_event("budget_exceeded", service="observability", details={"kind": "logs_fields"})
        if cfg.logs_on_exceed == "drop":
            return {"_dropped": True}
        keep_keys = list(sanitized.keys())[: cfg.max_event_fields]
        sanitized = {k: sanitized[k] for k in keep_keys}
        sanitized["_truncated"] = True

    dropped_new_keys = False
    for key in list(sanitized.keys()):
        key_s = str(key)
        if key_s not in _SEEN_FIELD_KEYS and len(_SEEN_FIELD_KEYS) >= cfg.max_unique_field_keys:
            dropped_new_keys = True
            sanitized.pop(key, None)
            budget.mark_logs_exceeded()
        else:
            _SEEN_FIELD_KEYS.add(key_s)

    if dropped_new_keys:
        sanitized["_keys_dropped"] = True
        emit_audit_event("budget_exceeded", service="observability", details={"kind": "logs_unique_keys"})

    return sanitized


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
        "reason_code": reason_code,
        "fields": _apply_log_budget(redact_fields(fields)),
    }
    for key in list(event.keys()):
        if event[key] is None:
            event.pop(key)

    cfg = budget.cfg()
    serialized = json.dumps(event, ensure_ascii=False, separators=(",", ":"))
    if len(serialized.encode("utf-8")) > cfg.max_event_bytes:
        budget.mark_logs_exceeded()
        emit_audit_event("budget_exceeded", service=service, details={"kind": "logs_bytes"})
        event["msg"] = event["msg"][:128]
        if "fields" in event:
            event["fields"] = {"_truncated": True}

    validate_required_keys(event, strict=is_strict_mode())
    return event


def emit_json(event: dict[str, Any]) -> None:
    print(json.dumps(event, ensure_ascii=False, separators=(",", ":"), sort_keys=True), flush=True)
