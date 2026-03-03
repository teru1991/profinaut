from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from libs.safety_core.errors import E_SECRET_PROVIDER_NOT_CONFIGURED, E_SECRET_RESOLVE_FAILED, err


@dataclass(frozen=True, slots=True)
class FileencProvider:
    mode: str
    base_dir: Path

    def resolve(self, *, path: str, field: str) -> str:
        base = self.base_dir.resolve()
        p = (self.base_dir / path).resolve()
        if base not in p.parents and p != base:
            raise err(E_SECRET_RESOLVE_FAILED, "path traversal rejected")

        if p.suffix == ".enc":
            raise err(E_SECRET_PROVIDER_NOT_CONFIGURED, "encrypted fileenc not implemented yet (handled in STEP3)", path=str(p))

        if p.suffix == ".json":
            if self.mode.lower() == "prod":
                raise err(
                    E_SECRET_PROVIDER_NOT_CONFIGURED,
                    "plaintext file secrets are forbidden in prod; use encrypted fileenc",
                    path=str(p),
                )
            raw = json.loads(p.read_text(encoding="utf-8"))
            if field not in raw:
                raise err(E_SECRET_RESOLVE_FAILED, "field not found in file", field=field, path=str(p))
            v = raw[field]
            if not isinstance(v, str) or v == "":
                raise err(E_SECRET_RESOLVE_FAILED, "field must be non-empty string", field=field, path=str(p))
            return v

        raise err(E_SECRET_RESOLVE_FAILED, "unsupported secret file type", path=str(p))
