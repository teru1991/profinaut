from __future__ import annotations

import gzip
import hashlib
import json
import os
import queue
import random
import re
import sqlite3
import threading
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Callable

from services.marketdata.app.storage.object_store import ObjectStore

_ALLOWED_EVENT_TYPES = {
    "trade",
    "orderbook_delta",
    "orderbook_snapshot",
    "ticker",
    "execution",
    "balance",
    "funding_rate",
    "mark_price",
}
_ALLOWED_ASSET_CLASS = {"crypto", "fx", "equity", "commodity", "unknown"}
_ALLOWED_CHANNEL = {"ws", "rest", "fix", "grpc", "file"}

_DENYLIST_KEY_PATTERNS = [
    re.compile(r"authorization", re.IGNORECASE),
    re.compile(r"api[-_]?key", re.IGNORECASE),
    re.compile(r"secret", re.IGNORECASE),
    re.compile(r"signature|\\bsig\\b", re.IGNORECASE),
    re.compile(r"token", re.IGNORECASE),
    re.compile(r"passphrase", re.IGNORECASE),
    re.compile(r"private_?key", re.IGNORECASE),
]
_DENYLIST_VALUE_PATTERNS = [
    re.compile(r"(?i)bearer\\s+[a-z0-9\\-\\._~\\+\\/]+=*"),
    re.compile(r"(?i)aws(.{0,20})?(secret|token)"),
    re.compile(r"(?i)(ed25519|rsa)?_?private_?key"),
]


def _parse_rfc3339(ts: str) -> datetime:
    return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC)


def _normalize_segment(value: object | None, fallback: str) -> str:
    if value is None:
        return fallback
    text = str(value).strip().lower()
    if not text:
        return fallback
    return re.sub(r"[^a-z0-9._-]", "_", text)


@dataclass(frozen=True)
class BronzeRecord:
    event_type: str
    source: dict[str, Any]
    source_event_id: str
    canonical_id: str
    idempotency_key: str
    event_time: str
    ingested_at: str
    payload: dict[str, Any]
    meta: dict[str, Any]

    def as_dict(self) -> dict[str, Any]:
        return {
            "event_type": self.event_type,
            "source": self.source,
            "source_event_id": self.source_event_id,
            "canonical_id": self.canonical_id,
            "idempotency_key": self.idempotency_key,
            "event_time": self.event_time,
            "ingested_at": self.ingested_at,
            "payload": self.payload,
            "meta": self.meta,
        }


class IdempotencyStore:
    def __init__(self, db_path: str, *, max_rows: int = 200_000, ttl_seconds: int = 3 * 24 * 3600) -> None:
        self._db_path = db_path
        self._max_rows = max_rows
        self._ttl_seconds = ttl_seconds
        Path(db_path).parent.mkdir(parents=True, exist_ok=True)
        self._conn = sqlite3.connect(db_path, check_same_thread=False)
        self._conn.execute(
            "CREATE TABLE IF NOT EXISTS dedupe (idempotency_key TEXT PRIMARY KEY, first_seen_ts TEXT NOT NULL)"
        )
        self._conn.commit()
        self._lock = threading.Lock()

    def first_seen(self, key: str, ts: str) -> bool:
        with self._lock:
            inserted = self._conn.execute(
                "INSERT OR IGNORE INTO dedupe(idempotency_key, first_seen_ts) VALUES(?,?)", (key, ts)
            ).rowcount
            self._conn.commit()
            return inserted == 1

    def compact(self, now: datetime) -> None:
        cutoff = (now.timestamp() - self._ttl_seconds)
        cutoff_s = datetime.fromtimestamp(cutoff, tz=UTC).isoformat().replace("+00:00", "Z")
        with self._lock:
            self._conn.execute("DELETE FROM dedupe WHERE first_seen_ts < ?", (cutoff_s,))
            over = self._conn.execute("SELECT COUNT(*) FROM dedupe").fetchone()[0] - self._max_rows
            if over > 0:
                self._conn.execute(
                    "DELETE FROM dedupe WHERE idempotency_key IN (SELECT idempotency_key FROM dedupe ORDER BY first_seen_ts ASC LIMIT ?)",
                    (over,),
                )
            self._conn.commit()


@dataclass
class _OpenPart:
    key: str
    opened_monotonic: float
    bytes_written: int
    records_written: int


class BronzeWriter:
    def __init__(
        self,
        store: ObjectStore,
        *,
        bucket: str = "local",
        max_bytes: int = 5 * 1024 * 1024,
        max_records: int = 5_000,
        max_seconds: float = 30,
        queue_maxsize: int = 1024,
        now_monotonic: Callable[[], float] | None = None,
        idempotency_store_path: str | None = None,
        spool_dir: str | None = None,
        max_payload_bytes: int = 512 * 1024,
        max_headers_bytes: int = 16 * 1024,
        compact_interval_seconds: float = 60,
        spool_max_bytes: int = 64 * 1024 * 1024,
        write_retry_max_attempts: int = 4,
        write_retry_base_seconds: float = 0.2,
        circuit_breaker_threshold: int = 3,
        circuit_breaker_cooldown_seconds: float = 10.0,
    ):
        self._store = store
        self._bucket = bucket
        self._max_bytes = max_bytes
        self._max_records = max_records
        self._max_seconds = max_seconds
        self._now = now_monotonic or time.monotonic
        self._open_part: _OpenPart | None = None
        self._buffer = bytearray()
        self._counters: dict[str, int] = {}
        self._queue: queue.Queue[tuple[BronzeRecord, queue.Queue[str]] | None] = queue.Queue(maxsize=queue_maxsize)
        self._degraded_reason: str | None = None
        self._last_success_ts: str | None = None
        self._spool_dir = Path(spool_dir or os.getenv("BRONZE_SPOOL_DIR", "data/bronze_spool"))
        self._spool_dir.mkdir(parents=True, exist_ok=True)
        self._spool_max_bytes = spool_max_bytes
        self._max_payload_bytes = max_payload_bytes
        self._max_headers_bytes = max_headers_bytes
        self._compact_interval_seconds = compact_interval_seconds
        self._last_compact_monotonic = self._now()
        self._write_retry_max_attempts = max(write_retry_max_attempts, 1)
        self._write_retry_base_seconds = max(write_retry_base_seconds, 0.01)
        self._circuit_breaker_threshold = max(circuit_breaker_threshold, 1)
        self._circuit_breaker_cooldown_seconds = max(circuit_breaker_cooldown_seconds, 0.1)
        self._consecutive_write_failures = 0
        self._circuit_opened_monotonic: float | None = None
        dedupe_path = idempotency_store_path or os.getenv("BRONZE_IDEMPOTENCY_DB", "data/bronze/idempotency.sqlite3")
        self._dedupe = IdempotencyStore(dedupe_path)
        self.metrics: dict[str, float] = {
            "ingest_total": 0,
            "persisted_total": 0,
            "reject_secret_total": 0,
            "reject_schema_total": 0,
            "dedupe_drop_total": 0,
            "write_latency_ms": 0,
        }
        self._lock = threading.Lock()
        self._worker = threading.Thread(target=self._run_worker, daemon=True)
        self._worker.start()

    def _partition_prefix(self, record: BronzeRecord) -> str:
        source = _normalize_segment(record.source.get("asset_class"), "unknown")
        venue = _normalize_segment(record.source.get("exchange") or record.source.get("venue"), "unknown")
        ts = _parse_rfc3339(record.ingested_at)
        return f"bronze/{source}/{venue}/dt={ts.strftime('%Y-%m-%d')}/hh={ts.strftime('%H')}"

    def _new_part_key(self, prefix: str) -> str:
        seq = self._counters.get(prefix, 0) + 1
        self._counters[prefix] = seq
        return f"{prefix}/part-{seq:05d}.jsonl.gz"

    def _open_for_prefix(self, prefix: str) -> None:
        self._open_part = _OpenPart(key=self._new_part_key(prefix), opened_monotonic=self._now(), bytes_written=0, records_written=0)
        self._buffer = bytearray()

    def _raw_ref(self, record: BronzeRecord, key: str) -> str:
        ts = _parse_rfc3339(record.ingested_at)
        exchange = _normalize_segment(record.source.get("exchange"), "unknown")
        part = key.split("/")[-1]
        return f"raw://bronze/{exchange}/{record.event_type}/dt={ts.strftime('%Y-%m-%d')}/hh={ts.strftime('%H')}/{part}#{record.idempotency_key}"

    def _headers_bytes(self, record: BronzeRecord) -> int:
        headers = record.meta.get("headers") or {}
        http_info = record.meta.get("http") or {}
        return len(json.dumps({"headers": headers, "http": http_info}, separators=(",", ":"), ensure_ascii=False).encode("utf-8"))

    def _derive_idempotency_key(
        self,
        envelope: dict[str, Any],
        source: dict[str, Any],
        payload: dict[str, Any],
        event_type: str,
        event_time: str,
    ) -> str:
        explicit = envelope.get("idempotency_key")
        if isinstance(explicit, str) and explicit.strip():
            return explicit.strip()
        for key in ("canonical_id", "source_event_id", "raw_msg_id"):
            value = envelope.get(key)
            if isinstance(value, str) and value.strip():
                return f"{source.get('exchange', 'unknown')}:{value.strip()}"
        canonical = json.dumps(
            {
                "event_type": event_type,
                "exchange": str(source.get("exchange") or "unknown"),
                "symbol": str(envelope.get("market_id") or payload.get("symbol") or "unknown"),
                "event_time": event_time,
                "payload": payload,
            },
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
        )
        return f"sha256:{hashlib.sha256(canonical.encode('utf-8')).hexdigest()}"

    def _check_secret(self, value: Any, path: str = "") -> bool:
        if isinstance(value, dict):
            for k, v in value.items():
                key_path = f"{path}.{k}" if path else str(k)
                lower_k = str(k).lower()
                if any(p.search(lower_k) for p in _DENYLIST_KEY_PATTERNS):
                    return True
                if "query" in key_path and any(token in lower_k for token in ("token", "sig", "signature")):
                    return True
                if self._check_secret(v, key_path):
                    return True
            return False
        if isinstance(value, list):
            return any(self._check_secret(item, path) for item in value)
        if isinstance(value, str):
            return any(p.search(value) for p in _DENYLIST_VALUE_PATTERNS)
        return False

    def _validate_schema(self, record: BronzeRecord) -> None:
        if record.event_type not in _ALLOWED_EVENT_TYPES:
            raise ValueError("event_type invalid")
        for field_name in ("source_event_id", "canonical_id", "idempotency_key"):
            if not getattr(record, field_name):
                raise ValueError(f"{field_name} required")
        _parse_rfc3339(record.event_time)
        _parse_rfc3339(record.ingested_at)
        if not isinstance(record.payload, dict):
            raise ValueError("payload must be object")
        if len(json.dumps(record.payload, separators=(",", ":"), ensure_ascii=False).encode("utf-8")) > self._max_payload_bytes:
            raise ValueError("payload too large")
        if self._headers_bytes(record) > self._max_headers_bytes:
            raise ValueError("headers too large")
        src = record.source
        if str(src.get("asset_class", "unknown")).lower() not in _ALLOWED_ASSET_CLASS:
            raise ValueError("source.asset_class invalid")
        if str(src.get("channel", "")).lower() not in _ALLOWED_CHANNEL:
            raise ValueError("source.channel invalid")

    def _to_record(self, envelope: dict[str, Any]) -> BronzeRecord:
        payload = envelope.get("payload_json") or envelope.get("payload") or {}
        if not isinstance(payload, dict):
            payload = {"value": payload}
        source = {
            "exchange": str(envelope.get("venue_id") or envelope.get("source", {}).get("exchange") or "unknown"),
            "venue": str(envelope.get("venue_id") or "unknown"),
            "channel": str(envelope.get("source", {}).get("channel") or envelope.get("source_type") or "ws").lower().split("_")[0],
            "transport": str(envelope.get("source", {}).get("transport") or "ws"),
            "asset_class": str(envelope.get("source", {}).get("asset_class") or "crypto"),
            "op_name": envelope.get("stream_name"),
            "catalog_id": envelope.get("instrument_id"),
        }
        event_type = str(envelope.get("event_type") or envelope.get("stream_name") or "trade").lower()
        if event_type not in _ALLOWED_EVENT_TYPES:
            event_type = "trade"
        source_event_id = str(envelope.get("source_event_id") or envelope.get("raw_msg_id") or "")
        event_time = str(envelope.get("event_ts") or envelope.get("event_time") or envelope.get("received_ts"))
        ingested_at = str(envelope.get("received_ts") or envelope.get("ingested_at"))
        idempotency_key = self._derive_idempotency_key(envelope, source, payload, event_type, event_time)
        meta = {
            "schema_version": "bronze.v1",
            "collector": "marketdata",
            "quality": envelope.get("quality_json") or {},
            "headers": envelope.get("headers") or {},
            "http": envelope.get("http") or {},
            "query": envelope.get("query") or {},
            "ws": envelope.get("ws") or {},
            "run_id": envelope.get("run_id"),
            "trace_id": envelope.get("trace_id"),
            "record_id": envelope.get("record_id") or source_event_id,
            "raw_ref": "",
        }
        return BronzeRecord(
            event_type=event_type,
            source=source,
            source_event_id=source_event_id,
            canonical_id=str(envelope.get("canonical_id") or source_event_id),
            idempotency_key=idempotency_key,
            event_time=event_time,
            ingested_at=ingested_at,
            payload=payload,
            meta=meta,
        )

    def _flush_current(self) -> str | None:
        if self._open_part is None or not self._buffer:
            self._open_part = None
            self._buffer = bytearray()
            return None
        part = self._open_part
        started = time.perf_counter()
        compressed = gzip.compress(bytes(self._buffer))
        try:
            if self._circuit_opened_monotonic is not None:
                elapsed = self._now() - self._circuit_opened_monotonic
                if elapsed < self._circuit_breaker_cooldown_seconds:
                    raise RuntimeError("circuit breaker open")

            for attempt in range(1, self._write_retry_max_attempts + 1):
                try:
                    self._store.put_bytes(part.key, compressed, content_type="application/gzip")
                    self._last_success_ts = datetime.now(UTC).isoformat().replace("+00:00", "Z")
                    self._degraded_reason = None
                    self._consecutive_write_failures = 0
                    self._circuit_opened_monotonic = None
                    break
                except Exception:
                    self._consecutive_write_failures += 1
                    if self._consecutive_write_failures >= self._circuit_breaker_threshold:
                        self._circuit_opened_monotonic = self._now()
                    if attempt >= self._write_retry_max_attempts:
                        raise
                    time.sleep(min(self._write_retry_base_seconds * (2 ** (attempt - 1)), 5.0))
        except Exception as exc:
            self._degraded_reason = str(exc)
            spool = self._spool_dir / f"{int(time.time() * 1000)}-{random.randint(1000,9999)}.jsonl"
            spool.write_bytes(bytes(self._buffer))
            self._trim_spool()
        finally:
            self.metrics["write_latency_ms"] = max((time.perf_counter() - started) * 1000.0, 0.0)
        key = part.key
        self._open_part = None
        self._buffer = bytearray()
        return key

    def _write_record(self, record: BronzeRecord) -> str:
        prefix = self._partition_prefix(record)
        now = self._now()
        if self._open_part is None:
            self._open_for_prefix(prefix)
        else:
            current_prefix = self._open_part.key.rsplit("/part-", 1)[0]
            if current_prefix != prefix:
                self._flush_current()
                self._open_for_prefix(prefix)

        assert self._open_part is not None
        raw_ref = self._raw_ref(record, self._open_part.key)
        payload = record.as_dict()
        payload["meta"] = dict(payload["meta"])
        payload["meta"]["raw_ref"] = raw_ref
        encoded = (json.dumps(payload, separators=(",", ":"), ensure_ascii=False) + "\n").encode("utf-8")

        if (
            self._open_part.bytes_written > 0
            and (self._open_part.bytes_written + len(encoded) > self._max_bytes or self._open_part.records_written >= self._max_records)
        ):
            self._flush_current()
            self._open_for_prefix(prefix)

        assert self._open_part is not None
        current_key = self._open_part.key
        self._buffer.extend(encoded)
        self._open_part.bytes_written += len(encoded)
        self._open_part.records_written += 1

        if (now - self._open_part.opened_monotonic) >= self._max_seconds:
            self._flush_current()
        return current_key

    def append(self, envelope: dict[str, Any]) -> str:
        with self._lock:
            self.metrics["ingest_total"] += 1
        record = self._to_record(envelope)
        try:
            self._validate_schema(record)
        except Exception:
            with self._lock:
                self.metrics["reject_schema_total"] += 1
            raise
        if (
            self._check_secret(record.payload)
            or self._check_secret(record.meta.get("headers") or {})
            or self._check_secret(record.meta.get("http") or {})
            or self._check_secret(record.meta.get("query") or {})
        ):
            with self._lock:
                self.metrics["reject_secret_total"] += 1
            raise ValueError("secret detected")

        if not self._dedupe.first_seen(record.idempotency_key, record.ingested_at):
            with self._lock:
                self.metrics["dedupe_drop_total"] += 1
            return "dedupe://dropped"

        if (self._now() - self._last_compact_monotonic) >= self._compact_interval_seconds:
            self._dedupe.compact(datetime.now(UTC))
            self._last_compact_monotonic = self._now()

        result_q: queue.Queue[str] = queue.Queue(maxsize=1)
        self._queue.put((record, result_q))
        return result_q.get()

    def _run_worker(self) -> None:
        while True:
            item = self._queue.get()
            if item is None:
                self._flush_current()
                return
            record, result_q = item
            key = self._write_record(record)
            self._flush_current()
            if key:
                with self._lock:
                    self.metrics["persisted_total"] += 1
            result_q.put(key)
            self._queue.task_done()

    def rotate_if_needed(self) -> bool:
        if self._open_part is None:
            return False
        elapsed = self._now() - self._open_part.opened_monotonic
        if self._open_part.bytes_written >= self._max_bytes or self._open_part.records_written >= self._max_records or elapsed >= self._max_seconds:
            self._flush_current()
            return True
        return False

    def health(self) -> dict[str, Any]:
        spool_bytes = sum(path.stat().st_size for path in self._spool_dir.glob("*.jsonl"))
        return {
            "degraded": self._degraded_reason is not None,
            "degraded_reason": self._degraded_reason,
            "last_success_ts": self._last_success_ts,
            "queue_depth": self._queue.qsize(),
            "spool_bytes": spool_bytes,
            "circuit_open": self._circuit_opened_monotonic is not None
            and (self._now() - self._circuit_opened_monotonic) < self._circuit_breaker_cooldown_seconds,
        }

    def _trim_spool(self) -> None:
        files = sorted(self._spool_dir.glob("*.jsonl"), key=lambda p: p.stat().st_mtime)
        total = sum(path.stat().st_size for path in files)
        while total > self._spool_max_bytes and files:
            oldest = files.pop(0)
            size = oldest.stat().st_size
            oldest.unlink(missing_ok=True)
            total -= size

    def close(self) -> None:
        self._queue.put(None)
        self._worker.join(timeout=5)
        self._flush_current()
