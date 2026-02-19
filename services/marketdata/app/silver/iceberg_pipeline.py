from __future__ import annotations

import gzip
import hashlib
import json
from collections import defaultdict
from dataclasses import dataclass
from datetime import UTC, date, datetime, timedelta
from pathlib import Path
from typing import Any, Iterable

from services.marketdata.app.metrics import normalization_metrics

TABLES = (
    "trades",
    "tickers",
    "orderbook_deltas",
    "orderbook_snapshots",
    "rejections",
)


@dataclass(frozen=True)
class RecomputeReport:
    range_start: str
    range_end: str
    venue: str | None
    symbols: tuple[str, ...]
    event_types: tuple[str, ...]
    row_counts: dict[str, int]
    rejection_counts: dict[str, int]
    sample_hashes: dict[str, str]
    latency_ms: dict[str, float]


@dataclass(frozen=True)
class BronzeEvent:
    raw_ref: str
    event_type: str
    venue: str
    symbol: str
    venue_symbol: str
    ts_event: str | None
    ts_recv: str
    payload: dict[str, Any]
    sequence: int | None


def _parse_ts(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(str(value).replace("Z", "+00:00")).astimezone(UTC)
    except ValueError:
        return None


def _to_rfc3339(value: datetime) -> str:
    return value.astimezone(UTC).isoformat().replace("+00:00", "Z")


def _normalize_symbol(raw: str | None) -> str:
    if not raw:
        return "UNKNOWN"
    return str(raw).strip().upper().replace("-", "_")


def _derive_event_type(record: dict[str, Any]) -> str:
    event_type = str(record.get("event_type") or "").lower()
    if event_type in {"ticker", "trade", "orderbook_snapshot", "orderbook_delta"}:
        return event_type
    payload = record.get("payload") if isinstance(record.get("payload"), dict) else {}
    if {"price", "qty", "side"}.issubset(payload.keys()):
        return "trade"
    if {"bid", "ask"}.intersection(payload.keys()):
        return "ticker"
    if "changes" in payload:
        return "orderbook_delta"
    if "bids" in payload and "asks" in payload:
        return "orderbook_snapshot"
    return "unknown"


def iter_bronze_keys(bronze_root: Path, *, start: datetime, end: datetime, venue: str | None) -> list[Path]:
    keys: list[Path] = []
    cur = start.astimezone(UTC).replace(minute=0, second=0, microsecond=0)
    while cur < end:
        prefix = bronze_root / "bronze" / "crypto"
        if venue:
            candidates = [prefix / venue.lower()]
        else:
            candidates = [p for p in prefix.iterdir() if p.is_dir()] if prefix.exists() else []
        for vdir in candidates:
            hour_dir = vdir / cur.strftime("%Y/%m/%d/%H")
            if hour_dir.exists():
                keys.extend(sorted(hour_dir.glob("*.jsonl.gz")))
        cur += timedelta(hours=1)
    return keys


def _build_raw_ref(path: Path, source_event_id: str) -> str:
    return f"raw://{path.as_posix()}#{source_event_id}"


def read_bronze_events(keys: Iterable[Path]) -> tuple[list[BronzeEvent], list[dict[str, Any]]]:
    events: list[BronzeEvent] = []
    rejects: list[dict[str, Any]] = []
    for key in keys:
        with gzip.open(key, "rt", encoding="utf-8") as fh:
            for line_no, line in enumerate(fh, start=1):
                if not line.strip():
                    continue
                try:
                    item = json.loads(line)
                    if not isinstance(item, dict):
                        raise ValueError("record_not_object")
                except Exception:
                    rejects.append({
                        "raw_ref": _build_raw_ref(key, f"line-{line_no}"),
                        "reason_code": "PARSE_ERROR",
                        "detail": "invalid_json",
                    })
                    normalization_metrics.record_parse_fail()
                    continue

                ts_recv = str(item.get("ingested_at") or "")
                if _parse_ts(ts_recv) is None:
                    rejects.append({
                        "raw_ref": _build_raw_ref(key, str(item.get("source_event_id") or f"line-{line_no}")),
                        "reason_code": "INVALID_TS_RECV",
                        "detail": ts_recv,
                    })
                    normalization_metrics.record_rejection("INVALID_TS_RECV")
                    continue

                payload = item.get("payload") if isinstance(item.get("payload"), dict) else {}
                source = item.get("source") if isinstance(item.get("source"), dict) else {}
                symbol = _normalize_symbol(str(source.get("catalog_id") or payload.get("symbol") or "UNKNOWN"))
                sequence = item.get("sequence") or payload.get("sequence")
                seq_value: int | None = None
                try:
                    if sequence is not None:
                        seq_value = int(sequence)
                except (TypeError, ValueError):
                    seq_value = None

                events.append(
                    BronzeEvent(
                        raw_ref=_build_raw_ref(key, str(item.get("source_event_id") or f"line-{line_no}")),
                        event_type=_derive_event_type(item),
                        venue=str(source.get("exchange") or source.get("venue") or "unknown").lower(),
                        symbol=symbol,
                        venue_symbol=str(payload.get("symbol") or symbol),
                        ts_event=str(item.get("event_time")) if item.get("event_time") else None,
                        ts_recv=ts_recv,
                        payload=payload,
                        sequence=seq_value,
                    )
                )
                normalization_metrics.record_bronze_read()
    events.sort(key=lambda r: (r.ts_recv, r.raw_ref))
    return events, rejects


def normalize_events(events: Iterable[BronzeEvent]) -> tuple[dict[str, list[dict[str, Any]]], list[dict[str, Any]]]:
    rows: dict[str, list[dict[str, Any]]] = {name: [] for name in TABLES}
    rejects: list[dict[str, Any]] = []
    last_seq: dict[tuple[str, str], int] = {}
    for event in events:
        base = {
            "raw_ref": event.raw_ref,
            "venue": event.venue,
            "symbol": event.symbol,
            "venue_symbol": event.venue_symbol,
            "ts_event": event.ts_event,
            "ts_recv": event.ts_recv,
            "dt": event.ts_recv[:10],
            "is_stale": False,
            "delay_ms": None,
            "anomaly_flags": [],
            "sequence_gap": False,
        }
        event_dt = _parse_ts(event.ts_event)
        recv_dt = _parse_ts(event.ts_recv)
        if event_dt and recv_dt:
            delay_ms = max(0, int((recv_dt - event_dt).total_seconds() * 1000))
            base["delay_ms"] = delay_ms
            base["is_stale"] = delay_ms > 5_000
            if base["is_stale"]:
                normalization_metrics.record_late_arrival()

        key = (event.venue, event.symbol)
        if event.sequence is not None:
            prev = last_seq.get(key)
            if prev is not None and event.sequence > prev + 1:
                base["sequence_gap"] = True
                base["anomaly_flags"] = ["SEQ_GAP"]
                normalization_metrics.record_seq_gap()
            last_seq[key] = event.sequence

        if event.event_type == "trade":
            payload = event.payload
            try:
                rows["trades"].append({
                    **base,
                    "price": float(payload["price"]),
                    "qty": float(payload["qty"]),
                    "side": str(payload.get("side") or "").lower(),
                    "trade_id": str(payload.get("trade_id") or ""),
                })
                normalization_metrics.record_silver_write("trades")
            except Exception:
                rejects.append({"raw_ref": event.raw_ref, "reason_code": "TRADE_PARSE_ERROR", "detail": "missing_fields"})
                normalization_metrics.record_rejection("TRADE_PARSE_ERROR")
        elif event.event_type == "ticker":
            payload = event.payload
            rows["tickers"].append(
                {
                    **base,
                    "bid": float(payload.get("bid") or payload.get("bid_px") or 0),
                    "ask": float(payload.get("ask") or payload.get("ask_px") or 0),
                    "last": float(payload.get("last") or payload.get("price") or 0),
                }
            )
            normalization_metrics.record_silver_write("tickers")
        elif event.event_type == "orderbook_delta":
            rows["orderbook_deltas"].append({**base, "sequence": event.sequence, "changes": event.payload.get("changes") or event.payload})
            normalization_metrics.record_silver_write("orderbook_deltas")
        elif event.event_type == "orderbook_snapshot":
            rows["orderbook_snapshots"].append({**base, "sequence": event.sequence, "bids": event.payload.get("bids") or [], "asks": event.payload.get("asks") or []})
            normalization_metrics.record_silver_write("orderbook_snapshots")
        else:
            rejects.append({"raw_ref": event.raw_ref, "reason_code": "UNSUPPORTED_EVENT", "detail": event.event_type})
            normalization_metrics.record_rejection("UNSUPPORTED_EVENT")

    rows["rejections"].extend(rejects)
    return rows, rejects


def write_silver_iceberg(rows: dict[str, list[dict[str, Any]]], *, silver_root: Path, namespace: str = "silver", batch_size: int = 10_000) -> None:
    import duckdb

    silver_root.mkdir(parents=True, exist_ok=True)
    con = duckdb.connect(str(silver_root / "silver.duckdb"))
    for table_name, entries in rows.items():
        if table_name not in TABLES:
            continue
        table_dir = silver_root / "iceberg" / namespace / table_name
        data_dir = table_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        (table_dir / "metadata.json").write_text(
            json.dumps({"format": "iceberg", "table": f"{namespace}.{table_name}", "partition": ["dt"], "updated_at": _to_rfc3339(datetime.now(UTC))}),
            encoding="utf-8",
        )
        if not entries:
            continue
        for idx in range(0, len(entries), batch_size):
            chunk = entries[idx : idx + batch_size]
            dt_value = str(chunk[0].get("dt") or str(chunk[0].get("ts_recv") or "")[0:10] or "unknown")
            out_file = data_dir / f"dt={dt_value}" / f"part-{idx // batch_size:05d}.parquet"
            out_file.parent.mkdir(parents=True, exist_ok=True)
            temp_json = silver_root / f".{table_name}-{idx:05d}.json"
            temp_json.write_text("\n".join(json.dumps(row, sort_keys=True, separators=(",",":")) for row in chunk), encoding="utf-8")
            src = str(temp_json).replace("'", "''")
            dst = str(out_file).replace("'", "''")
            con.execute(f"COPY (SELECT * FROM read_json_auto('{src}') ORDER BY raw_ref) TO '{dst}' (FORMAT PARQUET)")
            temp_json.unlink(missing_ok=True)


def build_report(rows: dict[str, list[dict[str, Any]]], rejects: list[dict[str, Any]], *, start: datetime, end: datetime, venue: str | None, symbols: tuple[str, ...], event_types: tuple[str, ...]) -> RecomputeReport:
    row_counts = {k: len(v) for k, v in rows.items()}
    rejection_counts: dict[str, int] = defaultdict(int)
    for item in rejects:
        rejection_counts[str(item.get("reason_code") or "UNKNOWN")] += 1

    sample_hashes: dict[str, str] = {}
    latency_ms: dict[str, float] = {}
    for table_name, table_rows in rows.items():
        if not table_rows:
            sample_hashes[table_name] = ""
            latency_ms[table_name] = 0.0
            continue
        canonical = json.dumps(sorted(table_rows, key=lambda r: (str(r.get("ts_recv")), str(r.get("raw_ref"))))[:100], sort_keys=True, separators=(",", ":"))
        sample_hashes[table_name] = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
        delays = [int(r.get("delay_ms") or 0) for r in table_rows if r.get("delay_ms") is not None]
        latency_ms[table_name] = float(sum(delays) / len(delays)) if delays else 0.0

    return RecomputeReport(
        range_start=_to_rfc3339(start),
        range_end=_to_rfc3339(end),
        venue=venue,
        symbols=symbols,
        event_types=event_types,
        row_counts=row_counts,
        rejection_counts=dict(rejection_counts),
        sample_hashes=sample_hashes,
        latency_ms=latency_ms,
    )


def run_recompute(*, bronze_root: Path, silver_root: Path, start: datetime, end: datetime, venue: str | None, symbols: tuple[str, ...], event_types: tuple[str, ...]) -> RecomputeReport:
    keys = iter_bronze_keys(bronze_root, start=start, end=end, venue=venue)
    events, scan_rejects = read_bronze_events(keys)
    if symbols:
        allowed = {_normalize_symbol(s) for s in symbols}
        events = [e for e in events if e.symbol in allowed]
    if event_types:
        allowed_types = {e.lower() for e in event_types}
        events = [e for e in events if e.event_type in allowed_types]

    rows, normalize_rejects = normalize_events(events)
    rows["rejections"].extend(scan_rejects)
    write_silver_iceberg(rows, silver_root=silver_root)
    return build_report(rows, [*scan_rejects, *normalize_rejects], start=start, end=end, venue=venue, symbols=symbols, event_types=event_types)
