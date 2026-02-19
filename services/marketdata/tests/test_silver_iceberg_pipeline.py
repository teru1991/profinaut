from __future__ import annotations

import gzip
import json
from datetime import UTC, datetime
from pathlib import Path

from services.marketdata.app.silver.iceberg_pipeline import (
    _normalize_symbol,
    iter_bronze_keys,
    run_recompute,
)


def _write_bronze_part(path: Path, rows: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with gzip.open(path, "wt", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row) + "\n")


def test_iter_bronze_keys_range_scan(tmp_path: Path) -> None:
    root = tmp_path / "lake"
    _write_bronze_part(root / "bronze/crypto/gmo/2026/02/16/00/part-00001.jsonl.gz", [])
    _write_bronze_part(root / "bronze/crypto/gmo/2026/02/17/00/part-00001.jsonl.gz", [])

    keys = iter_bronze_keys(
        root,
        start=datetime(2026, 2, 16, 0, 0, tzinfo=UTC),
        end=datetime(2026, 2, 16, 23, 59, tzinfo=UTC),
        venue="gmo",
    )
    assert len(keys) == 1
    assert "2026/02/16" in keys[0].as_posix()


def test_recompute_preserves_raw_ref_and_rejections(tmp_path: Path) -> None:
    bronze_root = tmp_path / "bronze_root"
    silver_root = tmp_path / "silver_root"
    part = bronze_root / "bronze/crypto/gmo/2026/02/16/00/part-00001.jsonl.gz"
    _write_bronze_part(
        part,
        [
            {
                "event_type": "trade",
                "source_event_id": "t-1",
                "event_time": "2026-02-16T00:00:00Z",
                "ingested_at": "2026-02-16T00:00:01Z",
                "source": {"exchange": "gmo", "catalog_id": "btc_jpy"},
                "payload": {"price": "100", "qty": "0.2", "side": "buy", "symbol": "BTC_JPY"},
            },
            {
                "event_type": "trade",
                "source_event_id": "bad-1",
                "event_time": "2026-02-16T00:00:00Z",
                "ingested_at": "2026-02-16T00:00:02Z",
                "source": {"exchange": "gmo", "catalog_id": "btc_jpy"},
                "payload": {"side": "buy"},
            },
        ],
    )

    report = run_recompute(
        bronze_root=bronze_root,
        silver_root=silver_root,
        start=datetime(2026, 2, 16, 0, 0, tzinfo=UTC),
        end=datetime(2026, 2, 16, 1, 0, tzinfo=UTC),
        venue="gmo",
        symbols=(),
        event_types=(),
    )

    assert report.row_counts["trades"] == 1
    assert report.row_counts["rejections"] >= 1
    parquet_files = list((silver_root / "iceberg/silver/trades/data").rglob("*.parquet"))
    assert parquet_files
    assert (silver_root / "iceberg/silver/trades/metadata.json").exists()


def test_symbol_normalization() -> None:
    assert _normalize_symbol("btc-jpy") == "BTC_JPY"
