from __future__ import annotations

from dataclasses import dataclass

from libs.safety_core.errors import E_SECRET_PROVIDER_NOT_CONFIGURED, err


@dataclass(frozen=True, slots=True)
class VaultProvider:
    enabled: bool = False

    def resolve(self, *, path: str, field: str, version_hint: str | None) -> str:
        if not self.enabled:
            raise err(E_SECRET_PROVIDER_NOT_CONFIGURED, "vault provider not configured")
        raise err(E_SECRET_PROVIDER_NOT_CONFIGURED, "vault provider stub")
