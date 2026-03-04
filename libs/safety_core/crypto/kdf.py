from __future__ import annotations

import os
from dataclasses import dataclass

from cryptography.hazmat.primitives.kdf.scrypt import Scrypt


@dataclass(frozen=True, slots=True)
class ScryptParams:
    salt: bytes
    n: int = 2**14
    r: int = 8
    p: int = 1
    dklen: int = 32

    @staticmethod
    def fresh() -> "ScryptParams":
        return ScryptParams(salt=os.urandom(16))

    def derive(self, passphrase: str) -> bytes:
        kdf = Scrypt(salt=self.salt, length=self.dklen, n=self.n, r=self.r, p=self.p)
        return kdf.derive(passphrase.encode("utf-8"))
