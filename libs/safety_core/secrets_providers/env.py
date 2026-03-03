from __future__ import annotations

import os
from dataclasses import dataclass

from libs.safety_core.errors import E_SECRET_ENV_FORBIDDEN, E_SECRET_RESOLVE_FAILED, err


@dataclass(frozen=True, slots=True)
class EnvProvider:
    mode: str

    def resolve(self, *, field: str) -> str:
        if self.mode.lower() == "prod":
            raise err(E_SECRET_ENV_FORBIDDEN, "env provider is forbidden in prod")
        v = os.environ.get(field)
        if v is None or v == "":
            raise err(E_SECRET_RESOLVE_FAILED, "env var not set", field=field)
        return v
