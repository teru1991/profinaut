from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import time

from libs.safety_core.errors import E_SECRET_PROVIDER_DISABLED, E_SECRET_RESOLVE_FAILED, SecError, err
from libs.safety_core.redaction import safe_str
from libs.safety_core.secrets_providers.env import EnvProvider
from libs.safety_core.secrets_providers.fileenc import FileencProvider
from libs.safety_core.secrets_providers.vault import VaultProvider
from libs.safety_core.secrets_ref import SecretRef, parse_secretref
from libs.safety_core.secrets_registry import AssetRegistry


@dataclass(frozen=True, slots=True)
class SecretValue:
    value: str
    expires_at: float


class Secrets:
    def __init__(
        self,
        *,
        mode: str,
        fileenc_base_dir: Path | None = None,
        registry: AssetRegistry | None = None,
        vault_enabled: bool = False,
        default_ttl_seconds: int = 30,
        enabled: bool = True,
    ) -> None:
        self._mode = mode
        self._enabled = enabled
        self._default_ttl = int(default_ttl_seconds)
        self._registry = registry or AssetRegistry()
        self._fileenc = FileencProvider(mode=mode, base_dir=fileenc_base_dir or Path("."))
        self._env = EnvProvider(mode=mode)
        self._vault = VaultProvider(enabled=vault_enabled)
        self._cache: dict[str, SecretValue] = {}

    def _now(self) -> float:
        return time.time()

    def parse(self, s: str) -> SecretRef:
        return parse_secretref(s)

    def resolve(self, ref: SecretRef, *, ttl_seconds: int | None = None) -> str:
        if not self._enabled:
            raise err(E_SECRET_PROVIDER_DISABLED, "secrets resolver disabled")

        ttl = int(ttl_seconds if ttl_seconds is not None else self._default_ttl)
        if ttl <= 0:
            ttl = 1

        self._registry.assert_allowed(
            registry_id=ref.registry_id,
            scheme=ref.scheme,
            scope=ref.scope,
            ttl_seconds=ttl,
        )

        key = ref.display()
        now = self._now()
        cached = self._cache.get(key)
        if cached and cached.expires_at > now:
            return cached.value

        try:
            if ref.scheme == "env":
                v = self._env.resolve(field=ref.field)
            elif ref.scheme == "fileenc":
                v = self._fileenc.resolve(
                    path=ref.path,
                    field=ref.field,
                    registry_id=ref.registry_id,
                    scope=ref.scope,
                    version_hint=ref.version_hint,
                )
            elif ref.scheme == "vault":
                v = self._vault.resolve(path=ref.path, field=ref.field, version_hint=ref.version_hint)
            else:
                raise err(E_SECRET_RESOLVE_FAILED, "unsupported scheme", scheme=ref.scheme)
        except SecError as e:
            raise e
        except Exception as e:
            raise err(E_SECRET_RESOLVE_FAILED, "secret resolve failed", ref=ref.display(), error=safe_str(str(e))) from None

        self._cache[key] = SecretValue(value=v, expires_at=now + ttl)
        return v

    def purge(self) -> None:
        self._cache.clear()

    def stats(self) -> dict[str, object]:
        return {"cache_items": len(self._cache), "mode": self._mode, "default_ttl_seconds": self._default_ttl}
