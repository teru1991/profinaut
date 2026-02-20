from __future__ import annotations

import gzip
import hashlib
import json
from collections import defaultdict
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
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
class DiffReport:
    baseline_hashes: dict[str, str]
    candidate_hashes: dict[str, str]
    mismatches: dict[str, dict[str, str]]


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


@dataclass(frozen=True)
class BronzeScanRequest:
    start: datetime
    end: datetime
    venue: str | None = None
    symbols: tuple[str, ...] = ()
    event_types: tuple[str, ...] = ()


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


def read_bronze_events(keys: Iterable[Path], *, symbols: tuple[str, ...] = (), event_types: tuple[str, ...] = ()) -> tuple[list[BronzeEvent], list[dict[str, Any]]]:
    allowed_symbols = {_normalize_symbol(symbol) for symbol in symbols}
    allowed_event_types = {etype.lower() for etype in event_types}

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
                    rejects.append(
                        {
                            "raw_ref": _build_raw_ref(key, f"line-{line_no}"),
                            "reason_code": "PARSE_ERROR",
                            "venue": "unknown",
                            "symbol": "UNKNOWN",
                            "dt": "unknown",
                        }
                    )
                    normalization_metrics.record_parse_fail()
                    continue

                raw_ref = _build_raw_ref(key, str(item.get("source_event_id") or f"line-{line_no}"))
                ts_recv = str(item.get("ingested_at") or "")
                parsed_recv = _parse_ts(ts_recv)
                if parsed_recv is None:
                    rejects.append(
                        {
                            "raw_ref": raw_ref,
                            "reason_code": "INVALID_TS_RECV",
                            "venue": "unknown",
                            "symbol": "UNKNOWN",
                            "dt": "unknown",
                        }
                    )
                    normalization_metrics.record_rejection("INVALID_TS_RECV")
                    continue

                payload = item.get("payload") if isinstance(item.get("payload"), dict) else {}
                source = item.get("source") if isinstance(item.get("source"), dict) else {}
                symbol = _normalize_symbol(str(source.get("catalog_id") or payload.get("symbol") or "UNKNOWN"))
                event_type = _derive_event_type(item)

                if allowed_symbols and symbol not in allowed_symbols:
                    continue
                if allowed_event_types and event_type not in allowed_event_types:
                    continue

                sequence = item.get("sequence") or payload.get("sequence")
                seq_value: int | None = None
                try:
                    if sequence is not None:
                        seq_value = int(sequence)
                except (TypeError, ValueError):
                    seq_value = None

                events.append(
                    BronzeEvent(
                        raw_ref=raw_ref,
                        event_type=event_type,
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
    last_recv: dict[tuple[str, str], datetime] = {}

    for event in events:
        event_dt = _parse_ts(event.ts_event)
        recv_dt = _parse_ts(event.ts_recv)
        assert recv_dt is not None

        anomaly_flags: list[str] = []
        key = (event.venue, event.symbol)
        prior_recv = last_recv.get(key)
        if prior_recv and recv_dt < prior_recv:
            anomaly_flags.append("OUT_OF_ORDER")
        last_recv[key] = recv_dt

        sequence_gap = False
        if event.sequence is not None:
            prev = last_seq.get(key)
            if prev is not None and event.sequence > prev + 1:
                sequence_gap = True
                anomaly_flags.append("SEQ_GAP")
                normalization_metrics.record_seq_gap()
            last_seq[key] = event.sequence

        delay_ms: int | None = None
        is_stale = False
        ts_event = event.ts_event
        if event_dt:
            delay_ms = max(0, int((recv_dt - event_dt).total_seconds() * 1000))
            is_stale = delay_ms > 5_000
            if is_stale:
                normalization_metrics.record_late_arrival()
        else:
            anomaly_flags.append("TS_EVENT_MISSING")

        base = {
            "raw_ref": event.raw_ref,
            "venue": event.venue,
            "symbol": event.symbol,
            "venue_symbol": event.venue_symbol,
            "ts_event": ts_event,
            "ts_recv": event.ts_recv,
            "dt": event.ts_recv[:10],
            "hh": event.ts_recv[11:13],
            "is_stale": is_stale,
            "delay_ms": delay_ms,
            "anomaly_flags": anomaly_flags,
            "sequence_gap": sequence_gap,
        }

        if event.event_type == "trade":
            payload = event.payload
            try:
                rows["trades"].append(
                    {
                        **base,
                        "price": float(payload["price"]),
                        "qty": float(payload["qty"]),
                        "side": str(payload.get("side") or "").lower(),
                        "trade_id": str(payload.get("trade_id") or ""),
                    }
                )
                normalization_metrics.record_silver_write("trades")
            except Exception:
                rejects.append(
                    {
                        "raw_ref": event.raw_ref,
                        "reason_code": "TRADE_PARSE_ERROR",
                        "venue": event.venue,
                        "symbol": event.symbol,
                        "dt": base["dt"],
                    }
                )
                normalization_metrics.record_rejection("TRADE_PARSE_ERROR")
        elif event.event_type == "ticker":
            payload = event.payload
            try:
                rows["tickers"].append(
                    {
                        **base,
                        "bid": float(payload.get("bid") or payload.get("bid_px") or 0),
                        "ask": float(payload.get("ask") or payload.get("ask_px") or 0),
                        "last": float(payload.get("last") or payload.get("price") or 0),
                    }
                )
                normalization_metrics.record_silver_write("tickers")
            except Exception:
                rejects.append(
                    {
                        "raw_ref": event.raw_ref,
                        "reason_code": "TICKER_PARSE_ERROR",
                        "venue": event.venue,
                        "symbol": event.symbol,
                        "dt": base["dt"],
                    }
                )
                normalization_metrics.record_rejection("TICKER_PARSE_ERROR")
        elif event.event_type == "orderbook_delta":
            rows["orderbook_deltas"].append({**base, "sequence": event.sequence, "changes": event.payload.get("changes") or event.payload})
            normalization_metrics.record_silver_write("orderbook_deltas")
        elif event.event_type == "orderbook_snapshot":
            rows["orderbook_snapshots"].append(
                {**base, "sequence": event.sequence, "bids": event.payload.get("bids") or [], "asks": event.payload.get("asks") or []}
            )
            normalization_metrics.record_silver_write("orderbook_snapshots")
        else:
            rejects.append(
                {
                    "raw_ref": event.raw_ref,
                    "reason_code": "UNSUPPORTED_EVENT",
                    "venue": event.venue,
                    "symbol": event.symbol,
                    "dt": base["dt"],
                }
            )
            normalization_metrics.record_rejection("UNSUPPORTED_EVENT")

    rows["rejections"].extend(rejects)
    return rows, rejects


def write_silver_iceberg(
    rows: dict[str, list[dict[str, Any]]],
    *,
    silver_root: Path,
    namespace: str = "silver",
    batch_size: int = 10_000,
    compact: bool = False,
) -> None:
    import duckdb

    silver_root.mkdir(parents=True, exist_ok=True)
    con = duckdb.connect(str(silver_root / "silver.duckdb"))

    if compact:
        batch_size = max(batch_size, 50_000)

    for table_name, entries in rows.items():
        if table_name not in TABLES:
            continue

        sorted_entries = sorted(entries, key=lambda row: (str(row.get("ts_recv") or ""), str(row.get("raw_ref") or "")))
        table_dir = silver_root / "iceberg" / namespace / table_name
        data_dir = table_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        (table_dir / "metadata.json").write_text(
            json.dumps(
                {
                    "format": "iceberg",
                    "table": f"{namespace}.{table_name}",
                    "partition": ["dt", "hh"],
                    "updated_at": _to_rfc3339(datetime.now(UTC)),
                    "batch_size": batch_size,
                }
            ),
            encoding="utf-8",
        )

        for idx in range(0, len(sorted_entries), batch_size):
            chunk = sorted_entries[idx : idx + batch_size]
            if not chunk:
                continue

            dt_value = str(chunk[0].get("dt") or "unknown")
            hh_value = str(chunk[0].get("hh") or "unknown")
            out_file = data_dir / f"dt={dt_value}" / f"hh={hh_value}" / f"part-{idx // batch_size:05d}.parquet"
            out_file.parent.mkdir(parents=True, exist_ok=True)
            temp_json = silver_root / f".{table_name}-{idx:05d}.json"
            temp_json.write_text("\n".join(json.dumps(row, sort_keys=True, separators=(",", ":")) for row in chunk), encoding="utf-8")
            src = str(temp_json).replace("'", "''")
            dst = str(out_file).replace("'", "''")
            con.execute(f"COPY (SELECT * FROM read_json_auto('{src}')) TO '{dst}' (FORMAT PARQUET)")
            temp_json.unlink(missing_ok=True)


def _rows_digest(rows: list[dict[str, Any]]) -> str:
    canonical = "\n".join(
        json.dumps(row, sort_keys=True, separators=(",", ":"))
        for row in sorted(rows, key=lambda item: (str(item.get("ts_recv") or ""), str(item.get("raw_ref") or "")))
    )
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def build_report(
    rows: dict[str, list[dict[str, Any]]],
    rejects: list[dict[str, Any]],
    *,
    start: datetime,
    end: datetime,
    venue: str | None,
    symbols: tuple[str, ...],
    event_types: tuple[str, ...],
) -> RecomputeReport:
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
        sample_hashes[table_name] = _rows_digest(table_rows)
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


def run_recompute(
    *,
    bronze_root: Path,
    silver_root: Path,
    start: datetime,
    end: datetime,
    venue: str | None,
    symbols: tuple[str, ...],
    event_types: tuple[str, ...],
    batch_size: int = 10_000,
    compact: bool = False,
) -> RecomputeReport:
    keys = iter_bronze_keys(bronze_root, start=start, end=end, venue=venue)
    events, scan_rejects = read_bronze_events(keys, symbols=symbols, event_types=event_types)
    rows, normalize_rejects = normalize_events(events)
    rows["rejections"].extend(scan_rejects)
    write_silver_iceberg(rows, silver_root=silver_root, batch_size=batch_size, compact=compact)
    return build_report(rows, [*scan_rejects, *normalize_rejects], start=start, end=end, venue=venue, symbols=symbols, event_types=event_types)


def run_diff(*, baseline_silver_root: Path, candidate_silver_root: Path) -> DiffReport:
    baseline_hashes = _table_hashes_from_parquet(baseline_silver_root)
    candidate_hashes = _table_hashes_from_parquet(candidate_silver_root)
    mismatches: dict[str, dict[str, str]] = {}
    for table in TABLES:
        if baseline_hashes.get(table, "") != candidate_hashes.get(table, ""):
            mismatches[table] = {
                "baseline": baseline_hashes.get(table, ""),
                "candidate": candidate_hashes.get(table, ""),
            }
    return DiffReport(baseline_hashes=baseline_hashes, candidate_hashes=candidate_hashes, mismatches=mismatches)


def _table_hashes_from_parquet(silver_root: Path) -> dict[str, str]:
    import duckdb

    con = duckdb.connect()
    hashes: dict[str, str] = {}
    for table in TABLES:
        pattern = (silver_root / "iceberg" / "silver" / table / "data" / "dt=*" / "hh=*" / "*.parquet").as_posix()
        files = list((silver_root / "iceberg" / "silver" / table / "data").rglob("*.parquet"))
        if not files:
            hashes[table] = ""
            continue
        query = f"SELECT * FROM read_parquet('{pattern}')"
        result = con.execute(query)
        col_names = [desc[0] for desc in result.description]
        parsed_rows = [dict(zip(col_names, row)) for row in result.fetchall()]
        parsed_rows.sort(key=lambda row: (str(row.get("ts_recv") or ""), str(row.get("raw_ref") or "")))
        canonical_rows = [json.dumps(row, sort_keys=True, separators=(",", ":"), default=str) for row in parsed_rows]
        digest_input = "\n".join(canonical_rows)
        hashes[table] = hashlib.sha256(digest_input.encode("utf-8")).hexdigest()
    return hashes
