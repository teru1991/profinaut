from __future__ import annotations

from dataclasses import dataclass, field
import time
from typing import Callable

from libs.safety_core.lease_client import SafetyLeaseClient


@dataclass(slots=True)
class LeaseSubject:
    subject_kind: str
    subject_id: str
    lease_id: str
    actor: str
    evidence: dict[str, str]


@dataclass(slots=True)
class SafetyLeaseRenewer:
    client: SafetyLeaseClient
    on_blocked: Callable[[LeaseSubject, str], None]
    tick_seconds: float = 5.0
    max_failures: int = 3
    _failures: dict[str, int] = field(default_factory=dict)

    def tick(self, subject: LeaseSubject) -> bool:
        try:
            self.client.renew_lease(
                {
                    "lease_id": subject.lease_id,
                    "ttl_seconds": 20,
                    "actor": subject.actor,
                    "idempotency_key": f"renew-{subject.lease_id}-{int(time.time())}",
                    "evidence": subject.evidence,
                }
            )
            self._failures[subject.lease_id] = 0
            return True
        except Exception:
            count = self._failures.get(subject.lease_id, 0) + 1
            self._failures[subject.lease_id] = count
            if count >= self.max_failures:
                self.on_blocked(subject, "EXECUTION_BLOCKED")
            return False
