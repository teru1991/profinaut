from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from enum import StrEnum
from typing import Any


class CommandType(StrEnum):
    START = "START"
    STOP = "STOP"
    RESTART = "RESTART"
    CANCEL_ALL = "CANCEL_ALL"
    FLATTEN = "FLATTEN"
    SAFE_MODE = "SAFE_MODE"
    SET_MODE = "SET_MODE"
    SYNC_BALANCE = "SYNC_BALANCE"


class AckStatus(StrEnum):
    ACCEPTED = "ACCEPTED"
    COMPLETED = "COMPLETED"
    FAILED = "FAILED"
    REJECTED_EXPIRED = "REJECTED_EXPIRED"
    REJECTED_DUPLICATE = "REJECTED_DUPLICATE"


@dataclass(slots=True)
class Command:
    command_id: str
    instance_id: str
    command_type: CommandType
    issued_at: datetime
    expires_at: datetime
    payload: dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "Command":
        return cls(
            command_id=data["command_id"],
            instance_id=data["instance_id"],
            command_type=CommandType(data["command_type"]),
            issued_at=datetime.fromisoformat(data["issued_at"]),
            expires_at=datetime.fromisoformat(data["expires_at"]),
            payload=data.get("payload") or {},
        )


@dataclass(slots=True)
class CommandAck:
    command_id: str
    instance_id: str
    status: AckStatus
    timestamp: datetime
    reason: str | None = None

    def as_dict(self) -> dict[str, Any]:
        return {
            "command_id": self.command_id,
            "instance_id": self.instance_id,
            "status": self.status.value,
            "timestamp": self.timestamp.isoformat(),
            "reason": self.reason,
        }
