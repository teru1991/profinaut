from __future__ import annotations

import gzip
import json

from services.marketdata.app.bronze_store import BronzeStore, RawMetaRepository
from services.marketdata.app.object_store import InMemoryObjectStore


def test_ingest_writes_gzip_jsonl_and_records_meta_fields() -> None:
    object_store = InMemoryObjectStore()
    meta_repo = RawMetaRepository()
    bronze = BronzeStore(object_store, meta_repo, gzip_enabled=True)

    meta = bronze.ingest_json({"symbol": "BTC_JPY", "last": 100.5}, quality_json={"status": "OK"})

    assert meta.object_key.endswith(".jsonl.gz")
    assert meta.content_encoding == "gzip"
    assert meta.content_type == "application/x-ndjson"
    assert isinstance(meta.object_size, int)
    assert meta.object_size > 0

    blob = object_store.get_object(meta.object_key)
    lines = gzip.decompress(blob).decode("utf-8").splitlines()
    assert len(lines) == 1
    assert json.loads(lines[0]) == {"last": 100.5, "symbol": "BTC_JPY"}


def test_replay_reads_both_gzip_and_plain_jsonl() -> None:
    object_store = InMemoryObjectStore()
    meta_repo = RawMetaRepository()
    bronze = BronzeStore(object_store, meta_repo, gzip_enabled=True)

    gz_meta = bronze.ingest_json({"source": "gmo", "kind": "ticker"})

    plain_key = "bronze/raw/2026/02/16/plain.jsonl"
    plain_payload = b'{"source":"gmo","kind":"trade"}\n'
    object_store.put_object(plain_key, plain_payload, content_type="application/x-ndjson")

    assert bronze.replay_payload(gz_meta.object_key) == [{"kind": "ticker", "source": "gmo"}]
    assert bronze.replay_payload(plain_key) == [{"kind": "trade", "source": "gmo"}]
