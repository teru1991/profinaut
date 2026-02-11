from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone

from .models import AckStatus, Command, CommandAck, CommandType


@dataclass(slots=True)
class CommandProcessor:
    seen_command_ids: set[str] = field(default_factory=set)
    mode: str = "PAPER"
    state: str = "RUNNING"

    def process(self, command: Command, now: datetime | None = None) -> CommandAck:
        ts = now or datetime.now(timezone.utc)

        if command.command_id in self.seen_command_ids:
            return CommandAck(
                command_id=command.command_id,
                instance_id=command.instance_id,
                status=AckStatus.REJECTED_DUPLICATE,
                timestamp=ts,
                reason="Duplicate command_id",
            )

        if command.expires_at <= ts:
            return CommandAck(
                command_id=command.command_id,
                instance_id=command.instance_id,
                status=AckStatus.REJECTED_EXPIRED,
                timestamp=ts,
                reason="Command TTL expired",
            )

        self.seen_command_ids.add(command.command_id)
        self._apply_command(command)
        return CommandAck(
            command_id=command.command_id,
            instance_id=command.instance_id,
            status=AckStatus.COMPLETED,
            timestamp=ts,
        )

    def _apply_command(self, command: Command) -> None:
        if command.command_type in {CommandType.SAFE_MODE, CommandType.STOP, CommandType.FLATTEN}:
            self.state = command.command_type
        elif command.command_type == CommandType.SET_MODE:
            requested_mode = command.payload.get("mode")
            if isinstance(requested_mode, str) and requested_mode:
                self.mode = requested_mode
