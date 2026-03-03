from __future__ import annotations

from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
import json
from pathlib import Path
from typing import Protocol

from libs.safety_core.redaction import redact, safe_str, scan_obj


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


class AuditLeakError(RuntimeError):
    """
    Raised when a payload is about to leak secrets/near-secrets into audit output.
    IMPORTANT: never include raw sensitive values in this exception message.
    """

    def __init__(self, message: str, *, finding_kinds: list[str] | None = None) -> None:
        super().__init__(message)
        self.finding_kinds = finding_kinds or []


class JsonlAuditWriter:
    def __init__(self, log_dir: Path | None = None) -> None:
        self._log_dir = log_dir or Path("libs/safety_core/_audit_log")
        self._log_dir.mkdir(parents=True, exist_ok=True)
        self._log_file = self._log_dir / "audit.jsonl"

    def _prepare_payload(self, event: AuditEvent) -> dict:
        payload = asdict(event)

        # 1) Redact everything (safe default)
        payload = redact(payload)  # type: ignore[assignment]

        # 2) Scan for any remaining secret indicators (Fail-closed trigger)
        findings = scan_obj(payload)
        if findings:
            kinds = sorted({f.kind for f in findings})
            # Do NOT log payload here; it may still be sensitive.
            raise AuditLeakError("audit payload contains secret indicators; refusing to write", finding_kinds=kinds)

        # 3) Keep minimal metadata for forensic without leaking
        # (Optional field; backward compatible)
        payload["redaction_guard"] = {"ok": True, "ts": datetime.now(UTC).isoformat()}
        return payload

    def write_event(self, event: AuditEvent) -> None:
        try:
            payload = self._prepare_payload(event)
            line = json.dumps(payload, ensure_ascii=False) + "\n"
        except AuditLeakError:
            # propagate as-is; caller will decide fail-closed behavior.
            raise
        except Exception as e:
            # Do not leak exception internals
            raise AuditLeakError(f"audit payload serialization failed: {safe_str(str(e))}") from None

        with self._log_file.open("a", encoding="utf-8") as f:
            f.write(line)
            f.flush()
