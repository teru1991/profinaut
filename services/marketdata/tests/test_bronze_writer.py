from __future__ import annotations

import json
from pathlib import Path

from services.marketdata.app.bronze.writer import BronzeWriter
from services.marketdata.app.storage.fs_store import FilesystemObjectStore


def _envelope(received_ts: str, *, seq: int = 1) -> dict[str, object]:
    return {
        "raw_msg_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "tenant_id": "tenant-a",
        "source_type": "WS_PUBLIC",
        "venue_id": "gmo",
        "market_id": "spot",
        "stream_name": "ticker",
        "endpoint": None,
        "symbol_raw": "BTC_JPY",
        "instrument_id": None,
        "event_ts": None,
        "received_ts": received_ts,
        "seq": seq,
        "source_msg_key": None,
        "payload_json": {"n": seq},
        "payload_hash": f"h-{seq}",
        "quality_json": {},
        "parser_version": "v0.1",
    }


def test_writer_generates_partitioned_jsonl_path(tmp_path: Path) -> None:
    store = FilesystemObjectStore(tmp_path)
    writer = BronzeWriter(store, max_bytes=1024 * 1024, max_seconds=300)

    writer.append(_envelope("2026-01-02T03:04:05Z"))
    writer.close()

    keys = store.list("bronze")
    assert keys == [
        "bronze/source=ws_public/venue=gmo/market=spot/date=2026-01-02/hour=03/part-0001.jsonl"
    ]

    payload = store.get_bytes(keys[0]).decode("utf-8").strip().splitlines()
    assert len(payload) == 1
    decoded = json.loads(payload[0])
    assert decoded["received_ts"] == "2026-01-02T03:04:05Z"


def test_writer_rotates_on_size_boundary(tmp_path: Path) -> None:
    store = FilesystemObjectStore(tmp_path)
    writer = BronzeWriter(store, max_bytes=120, max_seconds=300)

    writer.append(_envelope("2026-01-02T03:04:05Z", seq=1))
    writer.append(_envelope("2026-01-02T03:04:06Z", seq=2))
    writer.append(_envelope("2026-01-02T03:04:07Z", seq=3))
    writer.close()

    keys = store.list("bronze/source=ws_public/venue=gmo/market=spot/date=2026-01-02/hour=03")
    assert len(keys) >= 2
    assert keys[0].endswith("part-0001.jsonl")
    assert keys[1].endswith("part-0002.jsonl")


def test_writer_rotates_on_time_boundary(tmp_path: Path) -> None:
    store = FilesystemObjectStore(tmp_path)

    now = {"value": 1000.0}

    def fake_now() -> float:
        return now["value"]

    writer = BronzeWriter(store, max_bytes=1024 * 1024, max_seconds=10, now_monotonic=fake_now)

    writer.append(_envelope("2026-01-02T03:04:05Z", seq=1))
    now["value"] += 11
    rotated = writer.rotate_if_needed()
    assert rotated is True

    writer.append(_envelope("2026-01-02T03:04:06Z", seq=2))
    writer.close()

    keys = store.list("bronze/source=ws_public/venue=gmo/market=spot/date=2026-01-02/hour=03")
    assert len(keys) == 2
    first = store.get_bytes(keys[0]).decode("utf-8").strip().splitlines()
    second = store.get_bytes(keys[1]).decode("utf-8").strip().splitlines()
    assert len(first) == 1
    assert len(second) == 1
