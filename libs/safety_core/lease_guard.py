from __future__ import annotations

import time
from dataclasses import dataclass

from libs.safety_core.errors import err

E_LEASE_CONFLICT = "E_LEASE_CONFLICT"


@dataclass(frozen=True, slots=True)
class Lease:
    scope: str
    owner_id: str
    acquired_at: float


class LeaseGuard:
    def __init__(self) -> None:
        self._leases: dict[str, Lease] = {}

    def acquire(self, *, scope: str, owner_id: str) -> Lease:
        existing = self._leases.get(scope)
        if existing and existing.owner_id != owner_id:
            raise err(E_LEASE_CONFLICT, "lease conflict (split brain); refusing to proceed", scope=scope)
        l = Lease(scope=scope, owner_id=owner_id, acquired_at=time.time())
        self._leases[scope] = l
        return l

    def release(self, *, scope: str, owner_id: str) -> None:
        existing = self._leases.get(scope)
        if not existing:
            return
        if existing.owner_id != owner_id:
            raise err(E_LEASE_CONFLICT, "lease owned by different owner; refusing release", scope=scope)
        self._leases.pop(scope, None)
