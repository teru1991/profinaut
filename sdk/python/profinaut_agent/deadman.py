from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta


@dataclass(slots=True)
class DeadmanTransition:
    reason_code: str
    stale_for_seconds: int
    consecutive_failures: int


@dataclass(slots=True)
class DeadmanSwitch:
    stale_seconds: int = 90
    unreachable_since: datetime | None = None
    last_success_at: datetime | None = None
    consecutive_failures: int = 0
    safe_mode: bool = False

    def register_failure(self, now: datetime, reason_code: str) -> DeadmanTransition | None:
        self.consecutive_failures += 1
        if self.unreachable_since is None:
            self.unreachable_since = now

        stale_start = self.last_success_at or self.unreachable_since
        stale_for = now - stale_start

        if self.safe_mode:
            return None

        if stale_for >= timedelta(seconds=self.stale_seconds):
            self.safe_mode = True
            return DeadmanTransition(
                reason_code=reason_code,
                stale_for_seconds=max(0, int(stale_for.total_seconds())),
                consecutive_failures=self.consecutive_failures,
            )

        return None

    def register_success(self, now: datetime) -> None:
        self.last_success_at = now
        self.unreachable_since = None
        self.consecutive_failures = 0
