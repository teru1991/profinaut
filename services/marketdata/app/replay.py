from __future__ import annotations

import argparse
import json
import os
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


@dataclass(frozen=True)
class ReplayEnvelope:
    envelope: dict[str, Any]
    received_ts: str
    raw_msg_id: str
    seq: int



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


def _table_exists(repo: MarketDataMetaRepository, table_name: str) -> bool:
    row = repo._conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name = ?",
        (table_name,),
    ).fetchone()
    return row is not None


def _iter_raw_ingest_meta_rows(
    repo: MarketDataMetaRepository,
    *,
    start: datetime,
    end: datetime,
    venue: str | None,
    market: str | None,
    source_type: str | None,
) -> list[tuple[str, str, str | None, str | None, str | None]]:
    if not _table_exists(repo, "raw_ingest_meta"):
        return []

    where = ["received_ts >= ?", "received_ts < ?"]
    params: list[Any] = [start.isoformat().replace("+00:00", "Z"), end.isoformat().replace("+00:00", "Z")]
    if venue:
        where.append("LOWER(COALESCE(venue_id,'')) = ?")
        params.append(venue.lower())
    if market:
        where.append("LOWER(COALESCE(market_id,'')) = ?")
        params.append(market.lower())
    if source_type:
        where.append("LOWER(COALESCE(source_type,'')) = ?")
        params.append(source_type.lower())

    sql = f"""
        SELECT raw_msg_id, received_ts, seq, object_key, event_ts
        FROM raw_ingest_meta
        WHERE {' AND '.join(where)}
        ORDER BY received_ts ASC, raw_msg_id ASC, CAST(COALESCE(NULLIF(seq,''),'0') AS INTEGER) ASC
    """
    rows = repo._conn.execute(sql, tuple(params)).fetchall()
    return [(str(r[0]), str(r[1]), None if r[2] is None else str(r[2]), None if r[3] is None else str(r[3]), None if r[4] is None else str(r[4])) for r in rows]


def _parse_seq(value: object) -> int:
    try:
        return int(str(value))
    except (TypeError, ValueError):
        return 0


def _extract_envelopes_from_payload(payload: str, *, replay_now_ts: str) -> tuple[list[ReplayEnvelope], int]:
    envelopes: list[ReplayEnvelope] = []
    skipped = 0
    for line in payload.splitlines():
        if not line.strip():
            continue
        parsed = json.loads(line)
        if not isinstance(parsed, dict):
            skipped += 1
            continue
        if not parsed.get("received_ts"):
            parsed["received_ts"] = replay_now_ts
        if not parsed.get("event_ts"):
            # Replay can run against partially-populated envelopes; a seeded fallback keeps stale/event behavior deterministic.
            parsed["event_ts"] = replay_now_ts
        raw_msg_id = str(parsed.get("raw_msg_id") or "")
        envelopes.append(
            ReplayEnvelope(
                envelope=parsed,
                received_ts=str(parsed["received_ts"]),
                raw_msg_id=raw_msg_id,
                seq=_parse_seq(parsed.get("seq")),
            )
        )
    return envelopes, skipped


def _already_processed(repo: MarketDataMetaRepository, envelope: dict[str, Any], target: str) -> bool:
    raw_msg_id = str(envelope.get("raw_msg_id") or "")
    if not raw_msg_id:
        return False

    target_table_map = {
        "md_trades": "md_trades",
        "md_ohlcv": "md_ohlcv",
        "md_best_bid_ask": "md_best_bid_ask",
        "md_orderbook": "md_best_bid_ask",
    }
    table = target_table_map.get(target, "md_events_json")

    query = "SELECT 1 FROM " + table + " WHERE raw_msg_id = ? LIMIT 1"
    row = repo._conn.execute(query, (raw_msg_id,)).fetchone()
    return row is not None


def run_replay(
    *,
    from_ts: str,
    to_ts: str,
    db_dsn: str,
    bronze_root: str,
    venue: str | None = None,
    market: str | None = None,
    source_type: str | None = None,
    dry_run: bool = False,
    write: bool = False,
    parser_version: str = "v0.1",
) -> ReplaySummary:
    start = _parse_rfc3339(from_ts)
    end = _parse_rfc3339(to_ts)
    store = FilesystemObjectStore(bronze_root)
    repo = _connect_repo(db_dsn)

    replay_now_ts = os.getenv("REPLAY_NOW_TS", "").strip() or end.isoformat().replace("+00:00", "Z")

    keys: list[str]
    meta_rows = _iter_raw_ingest_meta_rows(
        repo,
        start=start,
        end=end,
        venue=venue,
        market=market,
        source_type=source_type,
    )
    if meta_rows:
        # Prefer DB metadata ordering when present; it includes explicit received_ts/raw_msg_id/seq ordering.
        keys = [row[3] for row in meta_rows if row[3]]
    else:
        keys = [
            key
            for key in store.list("bronze")
            if _key_in_scope(key, start=start, end=end, venue=venue, source_type=source_type)
            and (market is None or f"/market={market.lower()}/" in key)
        ]

    read_count = 0
    silver_count = 0
    events_count = 0
    skipped_count = 0

    envelopes: list[ReplayEnvelope] = []
    for key in sorted(set(keys)):
        payload = store.get_bytes(key).decode("utf-8")
        parsed, skipped = _extract_envelopes_from_payload(payload, replay_now_ts=replay_now_ts)
        envelopes.extend(parsed)
        skipped_count += skipped

    envelopes.sort(key=lambda item: (_parse_rfc3339(item.received_ts), item.raw_msg_id, item.seq))

    for item in envelopes:
        envelope = item.envelope
        if venue and str(envelope.get("venue_id") or "").lower() != venue.lower():
            continue
        if market and str(envelope.get("market_id") or "").lower() != market.lower():
            continue
        if source_type and str(envelope.get("source_type") or "").lower() != source_type.lower():
            continue

        read_count += 1
        envelope["parser_version"] = parser_version
        classified = classify_envelope(envelope)

        if dry_run or not write:
            if classified.target == "md_events_json" or classified.target == "md_orderbook":
                events_count += 1
            else:
                silver_count += 1
            continue

        if _already_processed(repo, envelope, classified.target):
            if classified.target == "md_events_json" or classified.target == "md_orderbook":
                events_count += 1
            else:
                silver_count += 1
            continue

        result = normalize_envelope(repo, envelope)
        if result.target == "md_events_json" or result.target == "md_orderbook":
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
    p.add_argument("--market", dest="market", default=None, help="Optional market filter")
    p.add_argument("--source_type", dest="source_type", default=None, help="Optional source type filter")
    p.add_argument("--parser-version", dest="parser_version", default="v0.1", help="Parser version to stamp")
    p.add_argument("--dry_run", dest="dry_run", action="store_true", help="Count only; do not write")
    p.add_argument("--write", dest="write", action="store_true", help="Write to silver/events tables")
    return p


def main() -> int:
    args = _build_parser().parse_args()
    summary = run_replay(
        from_ts=args.from_ts,
        to_ts=args.to_ts,
        db_dsn=args.db_dsn,
        bronze_root=args.bronze_root,
        venue=args.venue,
        market=args.market,
        source_type=args.source_type,
        dry_run=args.dry_run,
        write=args.write,
        parser_version=args.parser_version,
    )
    print(
        json.dumps(
            {
                "from": args.from_ts,
                "to": args.to_ts,
                "venue": args.venue,
                "market": args.market,
                "source_type": args.source_type,
                "dry_run": args.dry_run,
                "write": args.write,
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
