from __future__ import annotations

import json
import re
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any, Callable

from services.marketdata.app.storage.object_store import ObjectStore


def _parse_rfc3339(ts: str) -> datetime:
    return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC)


def _normalize_segment(value: object | None, fallback: str) -> str:
    if value is None:
        return fallback
    text = str(value).strip().lower()
    if not text:
        return fallback
    return re.sub(r"[^a-z0-9._-]", "_", text)


@dataclass
class _OpenPart:
    key: str
    tmp_key: str
    opened_monotonic: float
    bytes_written: int


class BronzeWriter:
    def __init__(
        self,
        store: ObjectStore,
        *,
        max_bytes: int = 5 * 1024 * 1024,
        max_seconds: float = 30,
        now_monotonic: Callable[[], float] | None = None,
    ):
        self._store = store
        self._max_bytes = max_bytes
        self._max_seconds = max_seconds
        self._now = now_monotonic or time.monotonic

        self._open_part: _OpenPart | None = None
        self._buffer = bytearray()
        self._counters: dict[str, int] = {}

    def _partition_prefix(self, envelope: dict[str, Any]) -> str:
        source = _normalize_segment(envelope.get("source_type"), "unknown")
        venue = _normalize_segment(envelope.get("venue_id"), "unknown")
        market = _normalize_segment(envelope.get("market_id"), "unknown")

        received_ts_raw = envelope.get("received_ts")
        if not isinstance(received_ts_raw, str):
            raise ValueError("envelope.received_ts must be an RFC3339 string")

        received_ts = _parse_rfc3339(received_ts_raw)
        date_s = received_ts.strftime("%Y-%m-%d")
        hour_s = received_ts.strftime("%H")

        return f"bronze/source={source}/venue={venue}/market={market}/date={date_s}/hour={hour_s}"

    def _new_part_key(self, prefix: str) -> str:
        seq = self._counters.get(prefix, 0) + 1
        self._counters[prefix] = seq
        return f"{prefix}/part-{seq:04d}.jsonl"

    def _open_for_prefix(self, prefix: str) -> None:
        key = self._new_part_key(prefix)
        self._open_part = _OpenPart(
            key=key,
            tmp_key=f"{key}.tmp",
            opened_monotonic=self._now(),
            bytes_written=0,
        )
        self._buffer = bytearray()

    def _flush_current(self) -> str | None:
        if self._open_part is None:
            return None

        part = self._open_part
        if not self._buffer:
            self._open_part = None
            return None

        self._store.put_bytes(part.tmp_key, bytes(self._buffer), content_type="application/x-ndjson")
        if hasattr(self._store, "rename"):
            getattr(self._store, "rename")(part.tmp_key, part.key)
        else:
            self._store.put_bytes(part.key, bytes(self._buffer), content_type="application/x-ndjson")

        written_key = part.key
        self._open_part = None
        self._buffer = bytearray()
        return written_key

    def append(self, envelope: dict[str, Any]) -> str:
        prefix = self._partition_prefix(envelope)
        now = self._now()

        if self._open_part is None:
            self._open_for_prefix(prefix)
        else:
            current_prefix = self._open_part.key.rsplit("/part-", 1)[0]
            if current_prefix != prefix:
                self._flush_current()
                self._open_for_prefix(prefix)

        encoded = (json.dumps(envelope, separators=(",", ":"), ensure_ascii=False) + "\n").encode("utf-8")

        assert self._open_part is not None
        if self._open_part.bytes_written > 0 and self._open_part.bytes_written + len(encoded) > self._max_bytes:
            self._flush_current()
            self._open_for_prefix(prefix)

        assert self._open_part is not None
        current_key = self._open_part.key
        self._buffer.extend(encoded)
        self._open_part.bytes_written += len(encoded)

        if (now - self._open_part.opened_monotonic) >= self._max_seconds:
            self._flush_current()

        return current_key

    def rotate_if_needed(self) -> bool:
        if self._open_part is None:
            return False

        now = self._now()
        elapsed = now - self._open_part.opened_monotonic
        if self._open_part.bytes_written >= self._max_bytes or elapsed >= self._max_seconds:
            self._flush_current()
            return True
        return False

    def close(self) -> None:
        self._flush_current()
