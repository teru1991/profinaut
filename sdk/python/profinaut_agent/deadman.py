from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta


@dataclass(slots=True)
class DeadmanSwitch:
    timeout_seconds: int = 90
    fallback_action: str = "SAFE_MODE"
    unreachable_since: datetime | None = None

    def register_failure(self, now: datetime) -> str | None:
        if self.unreachable_since is None:
            self.unreachable_since = now
            return None

        if now - self.unreachable_since >= timedelta(seconds=self.timeout_seconds):
            return self.fallback_action
        return None

    def register_success(self) -> None:
        self.unreachable_since = None
