from __future__ import annotations

import json
import os
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

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
        "fields": redact_fields(fields),
    }
    for key in list(event.keys()):
        if event[key] is None:
            event.pop(key)

    validate_required_keys(event, strict=is_strict_mode())
    return event


def emit_json(event: dict[str, Any]) -> None:
    print(json.dumps(event, ensure_ascii=False, separators=(",", ":"), sort_keys=True), flush=True)
