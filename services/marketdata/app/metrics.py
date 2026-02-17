from __future__ import annotations

import threading
from collections import deque
from datetime import UTC, datetime, timedelta


class IngestMetrics:
    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._ingest_count = 0
        self._ingest_fail_count = 0
        self._dup_suspect_count = 0
        self._dup_suspect_total = 0
        self._success_timestamps: deque[datetime] = deque()
        self._fail_timestamps: deque[datetime] = deque()
        self._dup_timestamps: deque[datetime] = deque()

    def _prune(self, now: datetime) -> None:
        threshold = now - timedelta(minutes=5)
        for queue in (self._success_timestamps, self._fail_timestamps, self._dup_timestamps):
            while queue and queue[0] < threshold:
                queue.popleft()

    def record_success(self, *, dup_suspect: bool) -> None:
        now = datetime.now(UTC)
        with self._lock:
            self._ingest_count += 1
            self._success_timestamps.append(now)
            if dup_suspect:
                self._dup_suspect_count += 1
                self._dup_suspect_total += 1
                self._dup_timestamps.append(now)
            self._prune(now)


    def record_trade_duplicate(self) -> None:
        now = datetime.now(UTC)
        with self._lock:
            self._dup_suspect_count += 1
            self._dup_suspect_total += 1
            self._dup_timestamps.append(now)
            self._prune(now)

    def record_failure(self) -> None:
        now = datetime.now(UTC)
        with self._lock:
            self._ingest_fail_count += 1
            self._fail_timestamps.append(now)
            self._prune(now)

    def summary(self) -> dict[str, object]:
        now = datetime.now(UTC)
        with self._lock:
            self._prune(now)
            return {
                "ingest_count": self._ingest_count,
                "ingest_fail_count": self._ingest_fail_count,
                "dup_suspect_count": self._dup_suspect_count,
                "dup_suspect_total": self._dup_suspect_total,
                "last_5m": {
                    "ingest_count": len(self._success_timestamps),
                    "ingest_fail_count": len(self._fail_timestamps),
                    "dup_suspect_count": len(self._dup_timestamps),
                },
            }

    def reset_for_tests(self) -> None:
        with self._lock:
            self._ingest_count = 0
            self._ingest_fail_count = 0
            self._dup_suspect_count = 0
            self._dup_suspect_total = 0
            self._success_timestamps.clear()
            self._fail_timestamps.clear()
            self._dup_timestamps.clear()


ingest_metrics = IngestMetrics()
