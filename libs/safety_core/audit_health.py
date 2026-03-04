from __future__ import annotations

import time
from dataclasses import dataclass

from libs.safety_core.errors import err

E_AUDIT_DOWN = "E_AUDIT_DOWN"


@dataclass(frozen=True, slots=True)
class AuditHealthStatus:
    ok: bool
    last_ok_ts: float | None
    last_err_ts: float | None
    last_err_code: str | None


class AuditHealth:
    def __init__(self) -> None:
        self._ok_ts: float | None = None
        self._err_ts: float | None = None
        self._err_code: str | None = None

    def mark_ok(self) -> None:
        self._ok_ts = time.time()

    def mark_err(self, code: str) -> None:
        self._err_ts = time.time()
        self._err_code = code

    def status(self) -> AuditHealthStatus:
        ok = (self._ok_ts is not None) and ((self._err_ts is None) or (self._ok_ts >= self._err_ts))
        return AuditHealthStatus(ok=ok, last_ok_ts=self._ok_ts, last_err_ts=self._err_ts, last_err_code=self._err_code)

    def require_ok_for_danger_ops(self) -> None:
        st = self.status()
        if not st.ok:
            raise err(E_AUDIT_DOWN, "audit health is down; refusing dangerous operations", last_err_code=st.last_err_code)
