from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from libs.safety_core.errors import E_SECRET_REGISTRY_DENY, err
from libs.safety_core.redaction import safe_str


@dataclass(frozen=True, slots=True)
class RegistryEntry:
    registry_id: str
    allowed_schemes: set[str]
    scopes: set[str]
    max_ttl_seconds: int


class AssetRegistry:
    def __init__(self, *, policy_path: Path | None = None) -> None:
        self._path = policy_path or Path("docs/policy/asset_registry.json")
        self._entries = self._load()

    def _load(self) -> dict[str, RegistryEntry]:
        if not self._path.exists():
            return {}
        raw = json.loads(self._path.read_text(encoding="utf-8"))
        items = raw.get("items", [])
        out: dict[str, RegistryEntry] = {}
        for it in items:
            rid = str(it.get("registry_id", "")).strip()
            if not rid:
                continue
            out[rid] = RegistryEntry(
                registry_id=rid,
                allowed_schemes=set(map(str, it.get("allowed_schemes", ["fileenc"]))),
                scopes=set(map(str, it.get("scopes", []))),
                max_ttl_seconds=int(it.get("max_ttl_seconds", 30)),
            )
        return out

    def require(self, registry_id: str) -> RegistryEntry:
        e = self._entries.get(registry_id)
        if not e:
            raise err(E_SECRET_REGISTRY_DENY, "registry_id not registered", registry_id=registry_id)
        return e

    def assert_allowed(self, *, registry_id: str, scheme: str, scope: str, ttl_seconds: int) -> RegistryEntry:
        e = self.require(registry_id)
        if scheme not in e.allowed_schemes:
            raise err(E_SECRET_REGISTRY_DENY, "scheme not allowed by registry", registry_id=registry_id, scheme=scheme)
        if e.scopes and scope not in e.scopes:
            raise err(E_SECRET_REGISTRY_DENY, "scope not allowed by registry", registry_id=registry_id, scope=safe_str(scope))
        if ttl_seconds > e.max_ttl_seconds:
            raise err(
                E_SECRET_REGISTRY_DENY,
                "ttl exceeds registry max",
                registry_id=registry_id,
                ttl=ttl_seconds,
                max=e.max_ttl_seconds,
            )
        return e
