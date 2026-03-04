from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]


@dataclass(frozen=True)
class RedactionPolicy:
    mode: str
    mask_value: str
    max_depth: int
    max_keys: int
    value_regex: list[re.Pattern[str]]
    restricted_contains: list[str]
    trade_exact: set[str]
    infra_contains: list[str]


def _repo_root() -> Path:
    current = Path(__file__).resolve()
    for parent in [current.parent] + list(current.parents):
        if (parent / "docs").exists() and (parent / "services").exists():
            return parent
    return Path.cwd()


def _extract_forbidden_list(data: dict[str, Any]) -> list[str]:
    if isinstance(data.get("keys"), list):
        return [str(item) for item in data["keys"]]
    if isinstance(data.get("keys"), dict) and isinstance(data["keys"].get("list"), list):
        return [str(item) for item in data["keys"]["list"]]
    if isinstance(data.get("forbidden"), dict) and isinstance(data["forbidden"].get("keys"), list):
        return [str(item) for item in data["forbidden"]["keys"]]
    return []


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

    keys = _extract_forbidden_list(data)
    return {key.strip().lower() for key in keys if key.strip()}


def load_redaction_policy() -> RedactionPolicy:
    mode = "mask"
    mask_value = "***"
    max_depth = 8
    max_keys = 2000
    value_regex: list[re.Pattern[str]] = []
    restricted_contains = ["token", "secret", "password", "authorization", "cookie", "signature", "nonce"]
    trade_exact = {"client_order_id", "order_id", "price", "qty", "size", "amount", "notional"}
    infra_contains = ["host", "hostname", "ip", "internal_url", "endpoint", "base_url"]

    if tomllib is not None:
        path = _repo_root() / "docs" / "policy" / "redaction.toml"
        if path.exists():
            try:
                data = tomllib.loads(path.read_text(encoding="utf-8"))
                redaction_section = data.get("redaction") if isinstance(data.get("redaction"), dict) else {}
                patterns_section = data.get("patterns") if isinstance(data.get("patterns"), dict) else {}
                restricted_section = (
                    data.get("restricted_fields") if isinstance(data.get("restricted_fields"), dict) else {}
                )

                mode = str(redaction_section.get("mode", mode)).strip() or mode
                mask_value = str(redaction_section.get("mask_value", mask_value))
                max_depth = int(redaction_section.get("max_depth", max_depth))
                max_keys = int(redaction_section.get("max_keys", max_keys))

                if isinstance(patterns_section.get("value_regex"), list):
                    value_regex = [re.compile(str(pattern)) for pattern in patterns_section["value_regex"]]
                if isinstance(restricted_section.get("contains"), list):
                    restricted_contains = [str(item).lower() for item in restricted_section["contains"]]
                if isinstance(restricted_section.get("trade_exact"), list):
                    trade_exact = {str(item).lower() for item in restricted_section["trade_exact"]}
                if isinstance(restricted_section.get("infra_contains"), list):
                    infra_contains = [str(item).lower() for item in restricted_section["infra_contains"]]
            except (OSError, ValueError, TypeError, re.error):
                pass

    return RedactionPolicy(
        mode=mode,
        mask_value=mask_value,
        max_depth=max(1, max_depth),
        max_keys=max(1, max_keys),
        value_regex=value_regex,
        restricted_contains=restricted_contains,
        trade_exact=trade_exact,
        infra_contains=infra_contains,
    )


_POLICY_CACHE: RedactionPolicy | None = None
_FORBIDDEN_CACHE: set[str] | None = None


def policy() -> RedactionPolicy:
    global _POLICY_CACHE
    if _POLICY_CACHE is None:
        _POLICY_CACHE = load_redaction_policy()
    return _POLICY_CACHE


def forbidden_keys() -> set[str]:
    global _FORBIDDEN_CACHE
    if _FORBIDDEN_CACHE is None:
        _FORBIDDEN_CACHE = load_forbidden_keys()
    return _FORBIDDEN_CACHE


def classify_key(key: str) -> str:
    normalized = str(key).lower()
    if normalized in forbidden_keys():
        return "RESTRICTED"

    redaction_policy = policy()
    if normalized in redaction_policy.trade_exact:
        return "RESTRICTED"
    if any(token and token in normalized for token in redaction_policy.restricted_contains):
        return "RESTRICTED"
    if any(token and token in normalized for token in redaction_policy.infra_contains):
        return "RESTRICTED"
    return "INTERNAL"


def sanitize_text(value: str) -> tuple[str, bool]:
    sanitized = value
    violated = False
    redaction_policy = policy()
    for regex in redaction_policy.value_regex:
        if regex.search(sanitized):
            violated = True
            sanitized = regex.sub(redaction_policy.mask_value, sanitized)
    return sanitized, violated


def sanitize(obj: Any, *, _depth: int = 0, _state: dict[str, int] | None = None) -> tuple[Any, list[dict[str, Any]]]:
    redaction_policy = policy()
    state = _state if _state is not None else {"count": 0}
    violations: list[dict[str, Any]] = []

    if _depth > redaction_policy.max_depth:
        return {"TRUNCATED": True}, [{"kind": "limit", "where": "depth", "action": "truncate"}]

    if state["count"] >= redaction_policy.max_keys:
        return {"TRUNCATED": True}, [{"kind": "limit", "where": "keys", "action": "truncate"}]

    if isinstance(obj, dict):
        out: dict[str, Any] = {}
        for key, value in obj.items():
            state["count"] += 1
            if state["count"] > redaction_policy.max_keys:
                out["TRUNCATED"] = True
                violations.append({"kind": "limit", "where": "keys", "action": "truncate"})
                break

            key_name = str(key)
            classification = classify_key(key_name)
            if classification == "RESTRICTED":
                violations.append(
                    {
                        "kind": "key",
                        "key": key_name,
                        "class": classification,
                        "action": redaction_policy.mode,
                    }
                )
                if redaction_policy.mode == "drop":
                    continue
                out[key_name] = redaction_policy.mask_value
                continue

            sanitized_value, child_violations = sanitize(value, _depth=_depth + 1, _state=state)
            out[key_name] = sanitized_value
            violations.extend(child_violations)
        return out, violations

    if isinstance(obj, list):
        out_list: list[Any] = []
        for item in obj:
            state["count"] += 1
            if state["count"] > redaction_policy.max_keys:
                out_list.append({"TRUNCATED": True})
                violations.append({"kind": "limit", "where": "keys", "action": "truncate"})
                break
            sanitized_item, child_violations = sanitize(item, _depth=_depth + 1, _state=state)
            out_list.append(sanitized_item)
            violations.extend(child_violations)
        return out_list, violations

    if isinstance(obj, str):
        sanitized_text, violated = sanitize_text(obj)
        if violated:
            violations.append({"kind": "value_pattern", "action": "mask"})
        return sanitized_text, violations

    return obj, violations
