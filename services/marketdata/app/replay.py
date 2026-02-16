from __future__ import annotations

import argparse
import json
import sqlite3
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

from services.marketdata.app.db.repository import MarketDataMetaRepository
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.silver.normalizer import classify_envelope, normalize_envelope
from services.marketdata.app.storage.fs_store import FilesystemObjectStore


def _parse_rfc3339(ts: str) -> datetime:
    return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC)


def _format_hour(dt: datetime) -> str:
    return dt.astimezone(UTC).strftime("%Y-%m-%d/%H")


def _iter_hours(start: datetime, end: datetime) -> list[datetime]:
    if end <= start:
        return []
    cur = start.astimezone(UTC).replace(minute=0, second=0, microsecond=0)
    out: list[datetime] = []
    while cur < end:
        out.append(cur)
        cur += timedelta(hours=1)
    return out


def _key_in_scope(
    key: str,
    *,
    start: datetime,
    end: datetime,
    venue: str | None,
    source_type: str | None,
) -> bool:
    if not key.startswith("bronze/source=") or "/date=" not in key or "/hour=" not in key:
        return False

    parts = key.split("/")
    try:
        source = parts[1].split("=", 1)[1]
        venue_id = parts[2].split("=", 1)[1]
        date_s = parts[4].split("=", 1)[1]
        hour_s = parts[5].split("=", 1)[1]
        dt = _parse_rfc3339(f"{date_s}T{hour_s}:00:00Z")
    except Exception:
        return False

    if not (start <= dt < end):
        return False
    if venue and venue_id != venue.lower():
        return False
    if source_type and source != source_type.lower():
        return False
    return key.endswith(".jsonl")


@dataclass(frozen=True)
class ReplaySummary:
    read_count: int
    silver_count: int
    events_count: int
    skipped_count: int



def _connect_repo(db_dsn: str) -> MarketDataMetaRepository:
    if not db_dsn.startswith("sqlite:///"):
        raise ValueError("DP-009 v0.1 supports sqlite:/// DSN")
    db_path = db_dsn.removeprefix("sqlite:///")
    if db_path != ":memory:":
        Path(db_path).parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn)
    return MarketDataMetaRepository(conn)


def run_replay(
    *,
    from_ts: str,
    to_ts: str,
    db_dsn: str,
    bronze_root: str,
    venue: str | None = None,
    source_type: str | None = None,
    dry_run: bool = False,
    parser_version: str = "v0.1",
) -> ReplaySummary:
    start = _parse_rfc3339(from_ts)
    end = _parse_rfc3339(to_ts)
    store = FilesystemObjectStore(bronze_root)
    repo = _connect_repo(db_dsn)

    keys = [
        key
        for key in store.list("bronze")
        if _key_in_scope(key, start=start, end=end, venue=venue, source_type=source_type)
    ]

    read_count = 0
    silver_count = 0
    events_count = 0
    skipped_count = 0

    for key in keys:
        payload = store.get_bytes(key).decode("utf-8")
        for line in payload.splitlines():
            if not line.strip():
                continue
            envelope = json.loads(line)
            if not isinstance(envelope, dict):
                skipped_count += 1
                continue
            read_count += 1
            envelope["parser_version"] = parser_version

            if dry_run:
                classified = classify_envelope(envelope)
                if classified.target == "md_events_json":
                    events_count += 1
                else:
                    silver_count += 1
                continue

            result = normalize_envelope(repo, envelope)
            if result.target == "md_events_json":
                events_count += 1
            else:
                silver_count += 1

    return ReplaySummary(
        read_count=read_count,
        silver_count=silver_count,
        events_count=events_count,
        skipped_count=skipped_count,
    )


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Replay Bronze raw envelopes into Silver/events")
    p.add_argument("--from", dest="from_ts", required=True, help="RFC3339 start timestamp")
    p.add_argument("--to", dest="to_ts", required=True, help="RFC3339 end timestamp")
    p.add_argument("--db-dsn", dest="db_dsn", required=True, help="DB DSN (sqlite:///... for v0.1)")
    p.add_argument("--bronze-root", dest="bronze_root", default="./data/bronze", help="Bronze FS root")
    p.add_argument("--venue", dest="venue", default=None, help="Optional venue filter")
    p.add_argument("--source_type", dest="source_type", default=None, help="Optional source type filter")
    p.add_argument("--parser-version", dest="parser_version", default="v0.1", help="Parser version to stamp")
    p.add_argument("--dry_run", dest="dry_run", action="store_true", help="Count only; do not write")
    return p


def main() -> int:
    args = _build_parser().parse_args()
    summary = run_replay(
        from_ts=args.from_ts,
        to_ts=args.to_ts,
        db_dsn=args.db_dsn,
        bronze_root=args.bronze_root,
        venue=args.venue,
        source_type=args.source_type,
        dry_run=args.dry_run,
        parser_version=args.parser_version,
    )
    print(
        json.dumps(
            {
                "from": args.from_ts,
                "to": args.to_ts,
                "venue": args.venue,
                "source_type": args.source_type,
                "dry_run": args.dry_run,
                "parser_version": args.parser_version,
                "read_count": summary.read_count,
                "silver_count": summary.silver_count,
                "events_count": summary.events_count,
                "skipped_count": summary.skipped_count,
            },
            separators=(",", ":"),
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
