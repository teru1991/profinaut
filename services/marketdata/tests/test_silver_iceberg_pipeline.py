from __future__ import annotations

import gzip
import json
from datetime import UTC, datetime
from pathlib import Path

import duckdb

from services.marketdata.app.silver.iceberg_pipeline import (
    _normalize_symbol,
    iter_bronze_keys,
    run_diff,
    run_recompute,
)


def _write_bronze_part(path: Path, rows: list[dict] | list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with gzip.open(path, "wt", encoding="utf-8") as fh:
        for row in rows:
            if isinstance(row, str):
                fh.write(row + "\n")
            else:
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


def test_recompute_event_types_edges_and_rejections(tmp_path: Path) -> None:
    bronze_root = tmp_path / "bronze_root"
    silver_root = tmp_path / "silver_root"
    part = bronze_root / "bronze/crypto/gmo/2026/02/16/00/part-00001.jsonl.gz"
    _write_bronze_part(
        part,
        [
            # valid trade
            {
                "event_type": "trade",
                "source_event_id": "t-1",
                "event_time": "2026-02-16T00:00:00Z",
                "ingested_at": "2026-02-16T00:00:01Z",
                "source": {"exchange": "gmo", "catalog_id": "btc_jpy"},
                "payload": {"price": "100", "qty": "0.2", "side": "buy", "symbol": "BTC_JPY", "sequence": 1},
            },
            # missing ts_event (allowed)
            {
                "event_type": "ticker",
                "source_event_id": "tk-1",
                "ingested_at": "2026-02-16T00:00:02Z",
                "source": {"exchange": "gmo", "catalog_id": "btc_jpy"},
                "payload": {"bid": "100", "ask": "101", "last": "100.5", "symbol": "BTC_JPY", "sequence": 2},
            },
            # seq gap
            {
                "event_type": "orderbook_delta",
                "source_event_id": "obd-1",
                "event_time": "2026-02-16T00:00:02Z",
                "ingested_at": "2026-02-16T00:00:03Z",
                "source": {"exchange": "gmo", "catalog_id": "btc_jpy"},
                "payload": {"changes": [{"side": "bid", "price": "99", "qty": "1"}], "symbol": "BTC_JPY", "sequence": 4},
            },
            # snapshot
            {
                "event_type": "orderbook_snapshot",
                "source_event_id": "obs-1",
                "event_time": "2026-02-16T00:00:03Z",
                "ingested_at": "2026-02-16T00:00:04Z",
                "source": {"exchange": "gmo", "catalog_id": "btc_jpy"},
                "payload": {"bids": [["99", "1"]], "asks": [["101", "1"]], "symbol": "BTC_JPY"},
            },
            # rejected trade parse
            {
                "event_type": "trade",
                "source_event_id": "bad-1",
                "event_time": "2026-02-16T00:00:04Z",
                "ingested_at": "2026-02-16T00:00:05Z",
                "source": {"exchange": "gmo", "catalog_id": "btc_jpy"},
                "payload": {"side": "buy"},
            },
            # rejected parse
            "{",
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
    assert report.row_counts["tickers"] == 1
    assert report.row_counts["orderbook_deltas"] == 1
    assert report.row_counts["orderbook_snapshots"] == 1
    assert report.row_counts["rejections"] == 2
    assert report.rejection_counts["TRADE_PARSE_ERROR"] == 1
    assert report.rejection_counts["PARSE_ERROR"] == 1

    trade_file = list((silver_root / "iceberg/silver/trades/data").rglob("*.parquet"))[0]
    ticker_file = list((silver_root / "iceberg/silver/tickers/data").rglob("*.parquet"))[0]
    delta_file = list((silver_root / "iceberg/silver/orderbook_deltas/data").rglob("*.parquet"))[0]
    assert "dt=2026-02-16/hh=00" in trade_file.as_posix()

    con = duckdb.connect()
    trade_raw_ref = con.execute(f"SELECT raw_ref FROM read_parquet('{trade_file.as_posix()}')").fetchone()[0]
    ticker_flags = con.execute(f"SELECT anomaly_flags FROM read_parquet('{ticker_file.as_posix()}')").fetchone()[0]
    delta_gap = con.execute(f"SELECT sequence_gap FROM read_parquet('{delta_file.as_posix()}')").fetchone()[0]
    assert trade_raw_ref.startswith("raw://")
    assert "TS_EVENT_MISSING" in ticker_flags
    assert delta_gap is True


def test_recompute_is_deterministic_and_diff_is_stable(tmp_path: Path) -> None:
    bronze_root = tmp_path / "bronze_root"
    silver_a = tmp_path / "silver_a"
    silver_b = tmp_path / "silver_b"
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
            }
        ],
    )

    report_a = run_recompute(
        bronze_root=bronze_root,
        silver_root=silver_a,
        start=datetime(2026, 2, 16, 0, 0, tzinfo=UTC),
        end=datetime(2026, 2, 16, 1, 0, tzinfo=UTC),
        venue="gmo",
        symbols=(),
        event_types=(),
    )
    report_b = run_recompute(
        bronze_root=bronze_root,
        silver_root=silver_b,
        start=datetime(2026, 2, 16, 0, 0, tzinfo=UTC),
        end=datetime(2026, 2, 16, 1, 0, tzinfo=UTC),
        venue="gmo",
        symbols=(),
        event_types=(),
    )

    assert report_a.row_counts == report_b.row_counts
    assert report_a.sample_hashes == report_b.sample_hashes

    diff = run_diff(baseline_silver_root=silver_a, candidate_silver_root=silver_b)
    assert diff.mismatches == {}


def test_symbol_normalization() -> None:
    assert _normalize_symbol("btc-jpy") == "BTC_JPY"
