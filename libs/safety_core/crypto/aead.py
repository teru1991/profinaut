from __future__ import annotations

import os
from dataclasses import dataclass

from cryptography.hazmat.primitives.ciphers.aead import AESGCM


@dataclass(frozen=True, slots=True)
class Aes256Gcm:
    key: bytes

    @staticmethod
    def fresh_nonce() -> bytes:
        return os.urandom(12)

    def encrypt(self, *, nonce: bytes, plaintext: bytes, aad: bytes) -> bytes:
        a = AESGCM(self.key)
        return a.encrypt(nonce, plaintext, aad)

    def decrypt(self, *, nonce: bytes, ciphertext: bytes, aad: bytes) -> bytes:
        a = AESGCM(self.key)
        return a.decrypt(nonce, ciphertext, aad)
