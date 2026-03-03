from __future__ import annotations

import os
from dataclasses import dataclass

from libs.safety_core.crypto.kdf import ScryptParams
from libs.safety_core.errors import E_SECRET_PROVIDER_NOT_CONFIGURED, err


@dataclass(frozen=True, slots=True)
class PassphraseRoT:
    params: ScryptParams

    @staticmethod
    def from_env() -> "PassphraseRoT":
        pw = os.environ.get("FILEENC_PASSPHRASE")
        if not pw:
            raise err(E_SECRET_PROVIDER_NOT_CONFIGURED, "FILEENC_PASSPHRASE is required for passphrase RoT")
        return PassphraseRoT(params=ScryptParams(salt=b""))

    def derive_key(self, *, passphrase: str, params: ScryptParams) -> bytes:
        return params.derive(passphrase)


def require_passphrase() -> str:
    pw = os.environ.get("FILEENC_PASSPHRASE")
    if not pw:
        raise err(E_SECRET_PROVIDER_NOT_CONFIGURED, "FILEENC_PASSPHRASE is required")
    return pw
