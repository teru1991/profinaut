from datetime import datetime, timedelta, timezone

from profinaut_agent.models import Command, CommandType
from profinaut_agent.processor import CommandProcessor


def make_command(command_id: str, expires_at: datetime) -> Command:
    now = datetime.now(timezone.utc)
    return Command(
        command_id=command_id,
        instance_id="inst-1",
        command_type=CommandType.SAFE_MODE,
        issued_at=now,
        expires_at=expires_at,
        payload={},
    )


def test_reject_expired_command() -> None:
    processor = CommandProcessor()
    now = datetime.now(timezone.utc)
    cmd = make_command("cmd-expired", expires_at=now - timedelta(seconds=1))

    ack = processor.process(cmd, now=now)
    assert ack.status.value == "REJECTED_EXPIRED"


def test_reject_duplicate_command_id() -> None:
    processor = CommandProcessor()
    now = datetime.now(timezone.utc)
    cmd = make_command("cmd-dup", expires_at=now + timedelta(seconds=30))

    first = processor.process(cmd, now=now)
    second = processor.process(cmd, now=now + timedelta(seconds=1))

    assert first.status.value == "COMPLETED"
    assert second.status.value == "REJECTED_DUPLICATE"
