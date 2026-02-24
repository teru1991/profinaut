"""
Fail-closed redaction utilities for contracts bridge.

Scans objects for forbidden keys and regex patterns.
On any detection, returns a redaction placeholder instead of the original data.
"""
from __future__ import annotations

import re
from typing import Any

DEFAULT_FORBIDDEN_KEYS: list[str] = [
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "private_key",
    "privatekey",
    "credential",
    "credentials",
    "auth_token",
    "access_token",
    "refresh_token",
    "session_key",
    "secret_key",
    "signing_key",
    "encryption_key",
]

DEFAULT_FORBIDDEN_REGEX: list[re.Pattern[str]] = [
    re.compile(
        r"(?i)(password|passwd|secret|token|api[_\-]?key|private[_\-]?key|credential)\s*[:=]\s*\S+"
    ),
    re.compile(r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*"),
]


def _collect_forbidden_paths(
    obj: Any,
    forbidden_keys: list[str],
    forbidden_regex: list[re.Pattern[str]],
    path: str,
    found_paths: list[str],
) -> None:
    """Recursively collect paths of forbidden keys and regex-matched values."""
    if isinstance(obj, dict):
        for k, v in obj.items():
            current_path = f"{path}.{k}" if path else k
            if k.lower() in {fk.lower() for fk in forbidden_keys}:
                found_paths.append(current_path)
            else:
                _collect_forbidden_paths(v, forbidden_keys, forbidden_regex, current_path, found_paths)
    elif isinstance(obj, (list, tuple)):
        for i, item in enumerate(obj):
            _collect_forbidden_paths(item, forbidden_keys, forbidden_regex, f"{path}[{i}]", found_paths)
    elif isinstance(obj, str):
        for pattern in forbidden_regex:
            if pattern.search(obj):
                found_paths.append(path)
                break


def redact_obj_fail_closed(
    obj: Any,
    forbidden_keys: list[str] | None = None,
    forbidden_regex: list[re.Pattern[str]] | None = None,
) -> tuple[Any, list[str]]:
    """
    Recursively scan obj for forbidden keys/patterns (fail-closed).

    Returns:
        (redacted_obj, forbidden_paths)

    If any forbidden key or pattern is detected, the entire obj is replaced
    with a redaction placeholder dict and forbidden_paths is non-empty.
    Otherwise returns (obj, []).
    """
    if forbidden_keys is None:
        forbidden_keys = DEFAULT_FORBIDDEN_KEYS
    if forbidden_regex is None:
        forbidden_regex = DEFAULT_FORBIDDEN_REGEX

    found_paths: list[str] = []
    _collect_forbidden_paths(obj, forbidden_keys, forbidden_regex, "", found_paths)

    if found_paths:
        return {"redaction_failed": True, "forbidden_paths": found_paths}, found_paths

    return obj, []
