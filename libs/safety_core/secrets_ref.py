from __future__ import annotations

from dataclasses import dataclass
from typing import Literal
from urllib.parse import parse_qs, urlparse

from libs.safety_core.errors import E_SECRETREF_INVALID, E_SECRETREF_PARSE, err
from libs.safety_core.redaction import safe_str

Scheme = Literal["fileenc", "vault", "env"]


@dataclass(frozen=True, slots=True)
class SecretRef:
    scheme: Scheme
    path: str
    field: str
    registry_id: str
    scope: str
    version_hint: str | None = None

    def display(self) -> str:
        vh = f"&version_hint={self.version_hint}" if self.version_hint else ""
        return f"{self.scheme}://{self.path}#{self.field}?registry_id={self.registry_id}&scope={self.scope}{vh}"


def parse_secretref(s: str) -> SecretRef:
    if not isinstance(s, str) or not s.strip():
        raise err(E_SECRETREF_PARSE, "secretref is empty")

    try:
        u = urlparse(s)
    except Exception as e:
        raise err(E_SECRETREF_PARSE, "secretref urlparse failed", error=str(e)) from None

    scheme = (u.scheme or "").lower()
    if scheme not in ("fileenc", "vault", "env"):
        raise err(E_SECRETREF_INVALID, "unsupported scheme", scheme=scheme)

    raw_path = (u.netloc + u.path).lstrip("/")
    if not raw_path:
        raise err(E_SECRETREF_INVALID, "missing path")

    fragment = (u.fragment or "").strip()
    fragment_query = ""
    if "?" in fragment and not u.query:
        fragment, fragment_query = fragment.split("?", 1)

    field = fragment.strip()
    if not field:
        raise err(E_SECRETREF_INVALID, "missing field (use #<field>)")

    qs = parse_qs((u.query or fragment_query), keep_blank_values=True)

    def one(name: str) -> str | None:
        v = qs.get(name)
        if not v:
            return None
        return (v[0] or "").strip() or None

    registry_id = one("registry_id")
    scope = one("scope")
    version_hint = one("version_hint")

    if not registry_id:
        raise err(E_SECRETREF_INVALID, "missing registry_id")
    if not scope:
        raise err(E_SECRETREF_INVALID, "missing scope")
    if ":" not in scope:
        raise err(E_SECRETREF_INVALID, "scope must be structured like 'venue:xxx:yyy' or 'bot:xxx:yyy'", scope=scope)

    if any(x in field for x in ("/", "\\", "..")):
        raise err(E_SECRETREF_INVALID, "invalid field", field=safe_str(field))
    if any(x in raw_path for x in ("\x00",)):
        raise err(E_SECRETREF_INVALID, "invalid path")

    return SecretRef(
        scheme=scheme,  # type: ignore[arg-type]
        path=raw_path,
        field=field,
        registry_id=registry_id,
        scope=scope,
        version_hint=version_hint,
    )
