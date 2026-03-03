from __future__ import annotations

from dataclasses import dataclass, field
from datetime import UTC, datetime, timedelta
from enum import Enum
import uuid


class SafetyMode(str, Enum):
    NORMAL = "NORMAL"
    SAFE = "SAFE"
    EMERGENCY_STOP = "EMERGENCY_STOP"


class ScopeKind(str, Enum):
    GLOBAL = "GLOBAL"
    VENUE = "VENUE"
    SYMBOL = "SYMBOL"
    BOT = "BOT"
    STRATEGY = "STRATEGY"
    ACCOUNT = "ACCOUNT"


@dataclass(slots=True)
class SafetyStateV1:
    mode: SafetyMode
    reason: str
    activated_by: str | None = None
    activated_at: str = field(default_factory=lambda: datetime.now(UTC).isoformat())
    schema_version: int = 1
    state_id: str = field(default_factory=lambda: str(uuid.uuid4()))


@dataclass(slots=True)
class SafetyDirective:
    scope_kind: ScopeKind
    selector: str
    mode: SafetyMode
    ttl_seconds: int
    reason: str
    actor: str
    evidence: dict[str, str]
    issued_at: str = field(default_factory=lambda: datetime.now(UTC).isoformat())

    def expires_at(self) -> datetime:
        issued = datetime.fromisoformat(self.issued_at)
        return issued + timedelta(seconds=self.ttl_seconds)


@dataclass(slots=True)
class SafetyDecision:
    mode: SafetyMode
    sources: list[SafetyDirective]
    latched: bool
    computed_at: str = field(default_factory=lambda: datetime.now(UTC).isoformat())
