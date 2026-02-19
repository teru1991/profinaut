from __future__ import annotations

import gzip
import json
from pathlib import Path

from services.marketdata.app.bronze.writer import BronzeWriter
from services.marketdata.app.storage.fs_store import FilesystemObjectStore


def _envelope(received_ts: str, *, seq: int = 1) -> dict[str, object]:
    return {
        "raw_msg_id": f"msg-{seq}",
        "source_type": "WS_PUBLIC",
        "venue_id": "gmo",
        "market_id": "BTC_JPY",
        "stream_name": "trade",
        "received_ts": received_ts,
        "event_ts": received_ts,
        "payload_json": {"price": "100", "qty": "1", "side": "buy", "trade_id": seq},
        "source_event_id": f"source-{seq}",
        "idempotency_key": f"gmo:BTC_JPY:{seq}",
    }


def _read_jsonl_gz(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in gzip.decompress(path.read_bytes()).decode("utf-8").splitlines() if line.strip()]


def test_writer_generates_partitioned_jsonl_gz_and_raw_ref(tmp_path: Path) -> None:
    store = FilesystemObjectStore(tmp_path)
    writer = BronzeWriter(store, max_bytes=1024 * 1024, max_seconds=300, idempotency_store_path=str(tmp_path / "dedupe.sqlite"))

    key = writer.append(_envelope("2026-01-02T03:04:05Z"))
    writer.close()

    assert key.startswith("bronze/crypto/gmo/2026/01/02/03/part-")
    assert key.endswith(".jsonl.gz")

    stored = _read_jsonl_gz(tmp_path / key)[0]
    assert stored["event_type"] == "trade"
    assert stored["meta"]["raw_ref"].startswith("raw://bronze/gmo/trade/dt=2026-01-02/hh=03/")


def test_denylist_rejects_secret_payload(tmp_path: Path) -> None:
    writer = BronzeWriter(FilesystemObjectStore(tmp_path), idempotency_store_path=str(tmp_path / "dedupe.sqlite"))
    envelope = _envelope("2026-01-02T03:04:05Z")
    envelope["payload_json"] = {"authorization": "Bearer token-value"}

    try:
        writer.append(envelope)
        assert False, "expected secret rejection"
    except ValueError as exc:
        assert "secret" in str(exc)


def test_schema_invalid_rejected(tmp_path: Path) -> None:
    writer = BronzeWriter(FilesystemObjectStore(tmp_path), idempotency_store_path=str(tmp_path / "dedupe.sqlite"))
    envelope = _envelope("2026-01-02T03:04:05Z")
    envelope["received_ts"] = "not-a-ts"

    try:
        writer.append(envelope)
        assert False, "expected schema rejection"
    except ValueError:
        pass


def test_idempotency_drops_duplicates_across_restart(tmp_path: Path) -> None:
    store = FilesystemObjectStore(tmp_path)
    db = tmp_path / "dedupe.sqlite"
    writer1 = BronzeWriter(store, idempotency_store_path=str(db))
    key1 = writer1.append(_envelope("2026-01-02T03:04:05Z", seq=1))
    writer1.close()

    writer2 = BronzeWriter(store, idempotency_store_path=str(db))
    key2 = writer2.append(_envelope("2026-01-02T03:04:05Z", seq=1))
    writer2.close()

    assert key1.endswith(".jsonl.gz")
    assert key2 == "dedupe://dropped"
