from __future__ import annotations

from dataclasses import dataclass, field
from datetime import UTC, datetime, timedelta
from uuid import uuid4


class LeaseInvalidError(ValueError):
    def __init__(self, reason_code: str, message: str) -> None:
        super().__init__(message)
        self.reason_code = reason_code


@dataclass(slots=True)
class ExecutionLease:
    subject_kind: str
    subject_id: str
    scope_kind: str
    selector: dict[str, str]
    issued_by: str
    reason: str
    issued_at: str = field(default_factory=lambda: datetime.now(UTC).isoformat())
    expires_at: str = field(default_factory=lambda: (datetime.now(UTC) + timedelta(seconds=20)).isoformat())
    lease_id: str = field(default_factory=lambda: str(uuid4()))

    def remaining_seconds(self, now: datetime) -> int:
        expires = datetime.fromisoformat(self.expires_at)
        return int((expires - now).total_seconds())

    def is_valid(self, now: datetime) -> bool:
        return self.remaining_seconds(now) > 0

    def assert_valid_or_raise(self, now: datetime) -> None:
        if not self.is_valid(now):
            raise LeaseInvalidError("LEASE_EXPIRED", "execution lease has expired")
