from __future__ import annotations

from fastapi.testclient import TestClient

from services.marketdata.app.bronze_store import BronzeStore, RawMetaRepository
from services.marketdata.app.hash_util import compute_payload_hash
from services.marketdata.app.main import app
from services.marketdata.app.object_store import InMemoryObjectStore, ObjectStoreStatus


def test_raw_meta_lookup_returns_storage_metadata(monkeypatch) -> None:
    from services.marketdata.app import main as app_main

    object_store = InMemoryObjectStore()
    repo = RawMetaRepository()
    bronze = BronzeStore(object_store, repo, gzip_enabled=True)
    meta = bronze.ingest_json({"symbol": "BTC_JPY", "kind": "ticker"}, quality_json={"status": "OK"})

    monkeypatch.setattr(app_main, "_raw_meta_repo", repo)
    monkeypatch.setattr(app_main, "_bronze_store", bronze)
    monkeypatch.setattr(app_main, "_object_store_status", ObjectStoreStatus(backend="s3", ready=True, degraded_reasons=[]))

    with TestClient(app) as client:
        response = client.get(f"/raw/meta/{meta.raw_msg_id}")

    assert response.status_code == 200
    body = response.json()
    assert body["raw_msg_id"] == meta.raw_msg_id
    assert body["object_key"] == meta.object_key
    assert body["payload_hash"] == compute_payload_hash({"symbol": "BTC_JPY", "kind": "ticker"})
    assert body["content_encoding"] == "gzip"
    assert body["content_type"] == "application/x-ndjson"
    assert body["object_size"] == meta.object_size


def test_raw_meta_returns_503_when_dependency_unavailable(monkeypatch) -> None:
    from services.marketdata.app import main as app_main

    monkeypatch.setattr(app_main, "_bronze_store", None)
    monkeypatch.setattr(
        app_main,
        "_object_store_status",
        ObjectStoreStatus(backend="s3", ready=False, degraded_reasons=["OBJECT_STORE_S3_MISSING_CONFIG:S3_BUCKET"]),
    )

    with TestClient(app) as client:
        response = client.get("/raw/meta/does-not-matter")

    assert response.status_code == 503
    body = response.json()
    assert body["code"] == "RAW_DEPENDENCY_UNAVAILABLE"


def test_raw_download_returns_ndjson_and_size_guard(monkeypatch) -> None:
    from services.marketdata.app import main as app_main

    object_store = InMemoryObjectStore()
    repo = RawMetaRepository()
    bronze = BronzeStore(object_store, repo, gzip_enabled=True)
    meta = bronze.ingest_json({"symbol": "BTC_JPY", "kind": "ticker"})

    monkeypatch.setattr(app_main, "_raw_meta_repo", repo)
    monkeypatch.setattr(app_main, "_bronze_store", bronze)

    with TestClient(app) as client:
        ok_response = client.get(f"/raw/download/{meta.raw_msg_id}")

    assert ok_response.status_code == 200
    assert ok_response.headers["content-type"].startswith("application/x-ndjson")
    assert '"kind":"ticker"' in ok_response.text

    too_large_meta = bronze.ingest_json({"payload": "x" * 4096})
    too_large_meta.object_size = 9999999
    repo.save(too_large_meta)

    monkeypatch.setenv("RAW_DOWNLOAD_MAX_BYTES", "1024")
    with TestClient(app) as client:
        large_response = client.get(f"/raw/download/{too_large_meta.raw_msg_id}")

    assert large_response.status_code == 413
    body = large_response.json()
    assert body["code"] == "RAW_DOWNLOAD_TOO_LARGE"
