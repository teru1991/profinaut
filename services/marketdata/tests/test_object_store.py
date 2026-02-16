from __future__ import annotations

from dataclasses import dataclass
from urllib.parse import parse_qs, urlparse

from services.marketdata.app.object_store import S3ObjectStore, S3ObjectStoreConfig, build_object_store_from_env


@dataclass
class _FakeResponse:
    payload: bytes

    def read(self) -> bytes:
        return self.payload

    def __enter__(self) -> _FakeResponse:
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        return None


def test_build_object_store_from_env_reports_missing_s3_config(monkeypatch) -> None:
    monkeypatch.setenv("OBJECT_STORE_BACKEND", "s3")
    monkeypatch.delenv("S3_ENDPOINT", raising=False)
    monkeypatch.delenv("S3_BUCKET", raising=False)
    monkeypatch.delenv("S3_ACCESS_KEY", raising=False)
    monkeypatch.delenv("S3_SECRET_KEY", raising=False)
    monkeypatch.delenv("S3_REGION", raising=False)

    store, status = build_object_store_from_env()

    assert store is None
    assert status.backend == "s3"
    assert status.ready is False
    assert "OBJECT_STORE_S3_MISSING_CONFIG:S3_ENDPOINT" in status.degraded_reasons


def test_s3_put_get_list_roundtrip_with_minio_style_paths(monkeypatch) -> None:
    storage: dict[str, bytes] = {}

    def fake_urlopen(req, timeout=5):
        assert req.headers.get("Authorization", "").startswith("AWS4-HMAC-SHA256")
        parsed = urlparse(req.full_url)
        path_parts = parsed.path.split("/")
        assert path_parts[1] == "bucket"
        key = "/".join(path_parts[2:])

        if req.get_method() == "PUT":
            storage[key] = req.data or b""
            return _FakeResponse(b"")

        query = parse_qs(parsed.query)
        if query.get("list-type") == ["2"]:
            prefix = query.get("prefix", [""])[0]
            keys = sorted(name for name in storage if name.startswith(prefix))
            items = "".join(f"<Contents><Key>{name}</Key></Contents>" for name in keys)
            return _FakeResponse(
                f"<ListBucketResult xmlns='http://s3.amazonaws.com/doc/2006-03-01/'>{items}</ListBucketResult>".encode(
                    "utf-8"
                )
            )

        if req.get_method() == "GET":
            return _FakeResponse(storage[key])

        raise AssertionError(f"Unexpected request: {req.get_method()} {req.full_url}")

    monkeypatch.setattr("urllib.request.urlopen", fake_urlopen)

    store = S3ObjectStore(
        S3ObjectStoreConfig(
            endpoint="http://127.0.0.1:9000",
            bucket="bucket",
            access_key="minio",
            secret_key="miniostorage",
            region="us-east-1",
            force_path_style=True,
        )
    )

    store.put_object("bronze/2026-01-01/part-1.jsonl", b"line-1\n")
    store.put_object("bronze/2026-01-01/part-2.jsonl", b"line-2\n")

    assert store.get_object("bronze/2026-01-01/part-1.jsonl") == b"line-1\n"
    assert store.list_objects("bronze/2026-01-01/") == [
        "bronze/2026-01-01/part-1.jsonl",
        "bronze/2026-01-01/part-2.jsonl",
    ]
