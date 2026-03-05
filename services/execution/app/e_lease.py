from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path


@dataclass(frozen=True)
class LeaseStatus:
    ok: bool
    lease_id: str | None
    expires_at_utc: datetime | None
    reason: str
    evidence: dict


def _parse_dt(s: str) -> datetime:
    return datetime.fromisoformat(s.replace("Z", "+00:00")).astimezone(timezone.utc)


class FileLeaseReader:
    def __init__(self, path: Path, skew_margin_sec: int = 2):
        self.path = path
        self.skew_margin = timedelta(seconds=skew_margin_sec)

    def read(self, now_utc: datetime | None = None) -> LeaseStatus:
        now = now_utc or datetime.now(timezone.utc)
        if not self.path.exists():
            return LeaseStatus(False, None, None, "lease_file_missing", {"path": str(self.path)})
        try:
            raw = json.loads(self.path.read_text(encoding="utf-8"))
            lease_id = str(raw["lease_id"])
            expires = _parse_dt(str(raw["expires_at_utc"]))
            ttl = int(raw.get("ttl_sec", 20))
            renew_every = int(raw.get("renew_every_sec", 5))
        except Exception as e:
            return LeaseStatus(False, None, None, "lease_file_invalid", {"path": str(self.path), "error": str(e)})

        if ttl != 20 or renew_every != 5:
            return LeaseStatus(False, lease_id, expires, "lease_contract_mismatch", {"ttl_sec": ttl, "renew_every_sec": renew_every})

        if now + self.skew_margin >= expires:
            return LeaseStatus(False, lease_id, expires, "lease_expired", {"now_utc": now.isoformat()})

        return LeaseStatus(True, lease_id, expires, "ok", {"now_utc": now.isoformat()})


def default_lease_reader() -> FileLeaseReader:
    return FileLeaseReader(Path("var/run/safety_lease.json"))
