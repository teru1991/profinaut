from __future__ import annotations

import gzip
import json
from datetime import UTC, datetime
from pathlib import Path

from services.marketdata.app import cli


def _write_bronze_part(path: Path, rows: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with gzip.open(path, "wt", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row) + "\n")


def test_silver_backfill_and_diff_cli(tmp_path: Path, capsys, monkeypatch) -> None:
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

    monkeypatch.setattr(
        "sys.argv",
        [
            "marketdata",
            "silver",
            "backfill",
            "--bronze-root",
            str(bronze_root),
            "--silver-root",
            str(silver_a),
            "--from-ts",
            datetime(2026, 2, 16, 0, 0, tzinfo=UTC).isoformat().replace("+00:00", "Z"),
            "--to-ts",
            datetime(2026, 2, 16, 1, 0, tzinfo=UTC).isoformat().replace("+00:00", "Z"),
            "--venue",
            "gmo",
        ],
    )
    assert cli.main() == 0
    capsys.readouterr()

    monkeypatch.setattr(
        "sys.argv",
        [
            "marketdata",
            "silver",
            "backfill",
            "--bronze-root",
            str(bronze_root),
            "--silver-root",
            str(silver_b),
            "--from-ts",
            datetime(2026, 2, 16, 0, 0, tzinfo=UTC).isoformat().replace("+00:00", "Z"),
            "--to-ts",
            datetime(2026, 2, 16, 1, 0, tzinfo=UTC).isoformat().replace("+00:00", "Z"),
            "--venue",
            "gmo",
        ],
    )
    assert cli.main() == 0
    capsys.readouterr()

    monkeypatch.setattr(
        "sys.argv",
        [
            "marketdata",
            "silver",
            "diff",
            "--baseline-silver-root",
            str(silver_a),
            "--candidate-silver-root",
            str(silver_b),
        ],
    )
    assert cli.main() == 0
    output = capsys.readouterr().out
    payload = json.loads(output)
    assert payload["mismatches"] == {}
