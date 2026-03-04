from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol

from libs.safety_core.errors import E_SECRET_PROVIDER_NOT_CONFIGURED, err


class RootOfTrust(Protocol):
    def get_wrapping_key(self) -> bytes: ...


@dataclass(frozen=True, slots=True)
class RotUnavailable:
    reason: str

    def get_wrapping_key(self) -> bytes:
        raise err(E_SECRET_PROVIDER_NOT_CONFIGURED, "root-of-trust unavailable", reason=self.reason)
