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
                "dup_suspect_total": self._dup_suspect_count,
                "last_5m": {
                    "ingest_count": len(self._success_timestamps),
                    "ingest_fail_count": len(self._fail_timestamps),
                    "dup_suspect_count": len(self._dup_timestamps),
                    "dup_suspect_total": len(self._dup_timestamps),
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


class QualityGateMetrics:
    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._anomaly_total = 0
        self._by_code: dict[str, int] = {}

    def record_anomaly(self, *, code: str) -> None:
        key = str(code or "UNKNOWN").upper()
        with self._lock:
            self._anomaly_total += 1
            self._by_code[key] = int(self._by_code.get(key, 0)) + 1

    def summary(self) -> dict[str, object]:
        with self._lock:
            return {"anomaly_total": self._anomaly_total, "by_code": dict(self._by_code)}

    def reset_for_tests(self) -> None:
        with self._lock:
            self._anomaly_total = 0
            self._by_code.clear()


quality_gate_metrics = QualityGateMetrics()


class NormalizationMetrics:
    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._bronze_read_total = 0
        self._silver_write_total = 0
        self._parse_fail_total = 0
        self._rejection_total = 0
        self._late_arrival_total = 0
        self._seq_gap_total = 0
        self._silver_by_table: dict[str, int] = {}

    def record_bronze_read(self) -> None:
        with self._lock:
            self._bronze_read_total += 1

    def record_silver_write(self, table_name: str) -> None:
        with self._lock:
            self._silver_write_total += 1
            self._silver_by_table[table_name] = int(self._silver_by_table.get(table_name, 0)) + 1

    def record_parse_fail(self) -> None:
        with self._lock:
            self._parse_fail_total += 1

    def record_rejection(self, _: str) -> None:
        with self._lock:
            self._rejection_total += 1

    def record_late_arrival(self) -> None:
        with self._lock:
            self._late_arrival_total += 1

    def record_seq_gap(self) -> None:
        with self._lock:
            self._seq_gap_total += 1

    def summary(self) -> dict[str, object]:
        with self._lock:
            return {
                "bronze_read_total": self._bronze_read_total,
                "silver_write_total": self._silver_write_total,
                "parse_fail_total": self._parse_fail_total,
                "rejection_total": self._rejection_total,
                "late_arrival_total": self._late_arrival_total,
                "seq_gap_total": self._seq_gap_total,
                "silver_by_table": dict(self._silver_by_table),
                "anomaly_total": self._rejection_total,
            }

    def reset_for_tests(self) -> None:
        with self._lock:
            self._bronze_read_total = 0
            self._silver_write_total = 0
            self._parse_fail_total = 0
            self._rejection_total = 0
            self._late_arrival_total = 0
            self._seq_gap_total = 0
            self._silver_by_table.clear()


normalization_metrics = NormalizationMetrics()
