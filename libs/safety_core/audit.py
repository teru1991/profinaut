from __future__ import annotations

from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
import json
from pathlib import Path
from typing import Protocol

from libs.safety_core.redaction import redact


@dataclass(slots=True)
class AuditEvent:
    event_type: str
    actor: str
    scope: str
    mode_from: str
    mode_to: str
    reason: str
    ttl: int
    evidence_ref: dict[str, str]
    ts: str = field(default_factory=lambda: datetime.now(UTC).isoformat())


class AuditWriter(Protocol):
    def write_event(self, event: AuditEvent) -> None: ...


class JsonlAuditWriter:
    def __init__(self, log_dir: Path | None = None) -> None:
        self._log_dir = log_dir or Path("libs/safety_core/_audit_log")
        self._log_dir.mkdir(parents=True, exist_ok=True)
        self._log_file = self._log_dir / "audit.jsonl"

    def write_event(self, event: AuditEvent) -> None:
        payload = asdict(event)
        payload["evidence_ref"] = redact(payload.get("evidence_ref", {}))
        with self._log_file.open("a", encoding="utf-8") as f:
            f.write(json.dumps(payload, ensure_ascii=False) + "\n")
            f.flush()
