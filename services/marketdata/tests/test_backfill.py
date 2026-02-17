from __future__ import annotations

import json
import sqlite3
from pathlib import Path

from services.marketdata.app.backfill import BackfillPage, run_backfill_ohlcv


class _FakePager:
    def __init__(self, pages: list[list[dict[str, object]]]):
        self.pages = pages

    def __call__(self, _symbol: str, _tf: str, page: int) -> BackfillPage:
        idx = page - 1
        if idx < 0 or idx >= len(self.pages):
            return BackfillPage(candles=[], has_more=False)
        candles = self.pages[idx]
        return BackfillPage(candles=[dict(c) for c in candles], has_more=(idx + 1) < len(self.pages))


def test_backfill_writes_ohlcv_and_resume_from_cursor_file(tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    cursor_file = tmp_path / "cursor.json"

    first = run_backfill_ohlcv(
        venue="gmo",
        market="spot",
        tf="1m",
        from_ts="2026-02-16T00:00:00Z",
        to_ts="2026-02-16T00:03:00Z",
        db_dsn=f"sqlite:///{db_file}",
        max_pages_per_run=1,
        symbol="BTC_JPY",
        fetch_page=_FakePager(
            [
                [
                    {"openTime": "2026-02-16T00:00:00Z", "open": 100, "high": 101, "low": 99, "close": 100.5, "volume": 10},
                    {"openTime": "2026-02-16T00:01:00Z", "open": 100.5, "high": 102, "low": 100, "close": 101, "volume": 12},
                ],
                [
                    {"openTime": "2026-02-16T00:02:00Z", "open": 101, "high": 103, "low": 100.5, "close": 102, "volume": 8},
                ],
            ]
        ),
        cursor_file=str(cursor_file),
    )
    assert first.completed is False
    assert first.candles_written == 2

    second = run_backfill_ohlcv(
        venue="gmo",
        market="spot",
        tf="1m",
        from_ts="2026-02-16T00:00:00Z",
        to_ts="2026-02-16T00:03:00Z",
        db_dsn=f"sqlite:///{db_file}",
        max_pages_per_run=5,
        symbol="BTC_JPY",
        fetch_page=_FakePager(
            [
                [
                    {"openTime": "2026-02-16T00:00:00Z", "open": 100, "high": 101, "low": 99, "close": 100.5, "volume": 10},
                    {"openTime": "2026-02-16T00:01:00Z", "open": 100.5, "high": 102, "low": 100, "close": 101, "volume": 12},
                ],
                [
                    {"openTime": "2026-02-16T00:02:00Z", "open": 101, "high": 103, "low": 100.5, "close": 102, "volume": 8},
                ],
            ]
        ),
        cursor_file=str(cursor_file),
    )

    assert second.resumed_from_page == 2
    conn = sqlite3.connect(db_file)
    count = conn.execute("SELECT COUNT(*) FROM md_ohlcv").fetchone()[0]
    assert count == 3

    state = json.loads(cursor_file.read_text(encoding="utf-8"))
    assert state == {}


def test_backfill_honors_max_pages_per_run(tmp_path: Path) -> None:
    db_file = tmp_path / "md.sqlite3"
    summary = run_backfill_ohlcv(
        venue="gmo",
        market="spot",
        tf="1m",
        from_ts="2026-02-16T00:00:00Z",
        to_ts="2026-02-16T00:05:00Z",
        db_dsn=f"sqlite:///{db_file}",
        max_pages_per_run=1,
        symbol="BTC_JPY",
        fetch_page=_FakePager(
            [
                [{"openTime": "2026-02-16T00:00:00Z", "open": 1, "high": 2, "low": 1, "close": 2, "volume": 1}],
                [{"openTime": "2026-02-16T00:01:00Z", "open": 2, "high": 3, "low": 2, "close": 3, "volume": 1}],
            ]
        ),
    )
    assert summary.pages_processed == 1
    assert summary.completed is False
    conn = sqlite3.connect(db_file)
    assert conn.execute("SELECT COUNT(*) FROM md_ohlcv").fetchone()[0] == 1
