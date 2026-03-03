from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from cryptography.exceptions import InvalidTag

from libs.safety_core.crypto.aead import Aes256Gcm
from libs.safety_core.crypto.fileenc_format import FileEncContext, b64d, loads_record
from libs.safety_core.crypto.kdf import ScryptParams
from libs.safety_core.crypto.rot_passphrase import require_passphrase
from libs.safety_core.errors import E_SECRET_PROVIDER_NOT_CONFIGURED, E_SECRET_RESOLVE_FAILED, err
from libs.safety_core.redaction import safe_str


@dataclass(frozen=True, slots=True)
class FileencProvider:
    mode: str
    base_dir: Path

    def _read_file(self, p: Path) -> bytes:
        return p.read_bytes()

    def _decrypt_v1(self, *, record_bytes: bytes, ctx: FileEncContext) -> dict:
        try:
            rec = loads_record(record_bytes)
            if rec.get("magic") != "UCEL-FILEENC" or int(rec.get("v", 0)) != 1:
                raise ValueError("unsupported record")
            kdf = rec["kdf"]
            aead = rec["aead"]
            aad_obj = rec.get("aad", {})
            ct_b64 = rec["ct"]

            salt = b64d(kdf["salt"])
            params = ScryptParams(
                salt=salt,
                n=int(kdf.get("n", 2**14)),
                r=int(kdf.get("r", 8)),
                p=int(kdf.get("p", 1)),
                dklen=int(kdf.get("dklen", 32)),
            )
            passphrase = require_passphrase()
            key = params.derive(passphrase)
            nonce = b64d(aead["nonce"])
            aad = b64d(aad_obj["context"])

            if aad != ctx.to_aad_bytes():
                raise ValueError("aad mismatch (context bind)")

            a = Aes256Gcm(key=key)
            pt = a.decrypt(nonce=nonce, ciphertext=b64d(ct_b64), aad=aad)
            obj = json.loads(pt.decode("utf-8"))
            if not isinstance(obj, dict):
                raise ValueError("plaintext must be json object")
            return obj
        except InvalidTag:
            raise err(E_SECRET_RESOLVE_FAILED, "fileenc decrypt failed (invalid tag / tampered)") from None
        except Exception as e:
            raise err(E_SECRET_RESOLVE_FAILED, "fileenc decrypt failed", error=safe_str(str(e))) from None

    def resolve(
        self,
        *,
        path: str,
        field: str,
        registry_id: str | None = None,
        scope: str | None = None,
        version_hint: str | None = None,
    ) -> str:
        p = (self.base_dir / path).resolve()
        if self.base_dir.resolve() not in p.parents and p != self.base_dir.resolve():
            raise err(E_SECRET_RESOLVE_FAILED, "path traversal rejected")

        if p.suffix == ".json":
            if self.mode.lower() == "prod":
                raise err(E_SECRET_PROVIDER_NOT_CONFIGURED, "plaintext file secrets are forbidden in prod; use .enc", path=str(p))
            raw = json.loads(p.read_text(encoding="utf-8"))
            if field not in raw:
                raise err(E_SECRET_RESOLVE_FAILED, "field not found in file", field=field, path=str(p))
            v = raw[field]
            if not isinstance(v, str) or v == "":
                raise err(E_SECRET_RESOLVE_FAILED, "field must be non-empty string", field=field, path=str(p))
            return v

        if p.suffix == ".enc":
            if registry_id is None or scope is None:
                raise err(E_SECRET_RESOLVE_FAILED, "registry_id and scope are required for .enc")
            ctx = FileEncContext(path=path, field=field, registry_id=registry_id, scope=scope, version_hint=version_hint)
            obj = self._decrypt_v1(record_bytes=self._read_file(p), ctx=ctx)
            if field not in obj:
                raise err(E_SECRET_RESOLVE_FAILED, "field not found in decrypted object", field=field, path=str(p))
            v = obj[field]
            if not isinstance(v, str) or v == "":
                raise err(E_SECRET_RESOLVE_FAILED, "field must be non-empty string", field=field, path=str(p))
            return v

        raise err(E_SECRET_RESOLVE_FAILED, "unsupported secret file type", path=str(p))
