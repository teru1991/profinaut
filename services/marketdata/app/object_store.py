from __future__ import annotations

import hashlib
import hmac
import os
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Protocol


class ObjectStoreError(RuntimeError):
    """Raised when object-store operations fail."""


class ObjectStore(Protocol):
    backend: str

    def put_object(self, key: str, data: bytes, content_type: str = "application/octet-stream") -> None: ...

    def get_object(self, key: str) -> bytes: ...

    def list_objects(self, prefix: str = "") -> list[str]: ...


@dataclass(frozen=True)
class ObjectStoreStatus:
    backend: str
    ready: bool
    degraded_reasons: list[str]


class InMemoryObjectStore:
    backend = "memory"

    def __init__(self) -> None:
        self._objects: dict[str, bytes] = {}

    def put_object(self, key: str, data: bytes, content_type: str = "application/octet-stream") -> None:
        self._objects[key] = data

    def get_object(self, key: str) -> bytes:
        if key not in self._objects:
            raise ObjectStoreError(f"Object not found: {key}")
        return self._objects[key]

    def list_objects(self, prefix: str = "") -> list[str]:
        return sorted(key for key in self._objects if key.startswith(prefix))


@dataclass(frozen=True)
class S3ObjectStoreConfig:
    endpoint: str
    bucket: str
    access_key: str
    secret_key: str
    region: str
    force_path_style: bool
    timeout_seconds: float = 5.0


class S3ObjectStore:
    backend = "s3"

    def __init__(self, config: S3ObjectStoreConfig):
        self._config = config

    @staticmethod
    def _sha256_hex(data: bytes) -> str:
        return hashlib.sha256(data).hexdigest()

    def _bucket_host(self) -> str:
        endpoint = urllib.parse.urlparse(self._config.endpoint)
        base_host = endpoint.netloc
        if self._config.force_path_style:
            return base_host
        return f"{self._config.bucket}.{base_host}"

    def _canonical_uri(self, key: str) -> str:
        encoded_key = urllib.parse.quote(key, safe="/-_.~")
        if self._config.force_path_style:
            return f"/{self._config.bucket}/{encoded_key}"
        return f"/{encoded_key}"

    def _url(self, key: str, query: dict[str, str] | None = None) -> str:
        endpoint = urllib.parse.urlparse(self._config.endpoint)
        host = self._bucket_host()
        uri = self._canonical_uri(key)
        query_string = self._canonical_query(query)
        return urllib.parse.urlunparse((endpoint.scheme or "http", host, uri, "", query_string, ""))

    @staticmethod
    def _canonical_query(query: dict[str, str] | None) -> str:
        if not query:
            return ""
        items = sorted((k, v) for k, v in query.items())
        return "&".join(
            f"{urllib.parse.quote(k, safe='-_.~')}={urllib.parse.quote(v, safe='-_.~')}" for k, v in items
        )

    def _signing_key(self, date_stamp: str) -> bytes:
        key = ("AWS4" + self._config.secret_key).encode("utf-8")
        k_date = hmac.new(key, date_stamp.encode("utf-8"), hashlib.sha256).digest()
        k_region = hmac.new(k_date, self._config.region.encode("utf-8"), hashlib.sha256).digest()
        k_service = hmac.new(k_region, b"s3", hashlib.sha256).digest()
        return hmac.new(k_service, b"aws4_request", hashlib.sha256).digest()

    def _request(
        self,
        method: str,
        key: str,
        body: bytes = b"",
        content_type: str | None = None,
        query: dict[str, str] | None = None,
    ) -> bytes:
        now = datetime.now(UTC)
        amz_date = now.strftime("%Y%m%dT%H%M%SZ")
        date_stamp = now.strftime("%Y%m%d")

        host = self._bucket_host()
        canonical_uri = self._canonical_uri(key)
        canonical_query = self._canonical_query(query)
        payload_hash = self._sha256_hex(body)

        canonical_headers = (
            f"host:{host}\n"
            f"x-amz-content-sha256:{payload_hash}\n"
            f"x-amz-date:{amz_date}\n"
        )
        signed_headers = "host;x-amz-content-sha256;x-amz-date"
        canonical_request = (
            f"{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        )

        algorithm = "AWS4-HMAC-SHA256"
        credential_scope = f"{date_stamp}/{self._config.region}/s3/aws4_request"
        string_to_sign = (
            f"{algorithm}\n{amz_date}\n{credential_scope}\n{self._sha256_hex(canonical_request.encode('utf-8'))}"
        )
        signature = hmac.new(
            self._signing_key(date_stamp),
            string_to_sign.encode("utf-8"),
            hashlib.sha256,
        ).hexdigest()

        auth_header = (
            f"{algorithm} "
            f"Credential={self._config.access_key}/{credential_scope}, "
            f"SignedHeaders={signed_headers}, "
            f"Signature={signature}"
        )

        url = self._url(key, query=query)
        headers = {
            "Host": host,
            "x-amz-date": amz_date,
            "x-amz-content-sha256": payload_hash,
            "Authorization": auth_header,
        }
        if content_type:
            headers["Content-Type"] = content_type

        req = urllib.request.Request(url, data=body if body else None, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=self._config.timeout_seconds) as response:
                return response.read()
        except (urllib.error.URLError, TimeoutError) as exc:  # pragma: no cover - normalized wrapper
            raise ObjectStoreError(f"S3 request failed: {method} {key}: {exc}") from exc

    def put_object(self, key: str, data: bytes, content_type: str = "application/octet-stream") -> None:
        self._request("PUT", key=key, body=data, content_type=content_type)

    def get_object(self, key: str) -> bytes:
        return self._request("GET", key=key)

    def list_objects(self, prefix: str = "") -> list[str]:
        # key is unused for list operation; keep root path to allow consistent signer path handling.
        payload = self._request("GET", key="", query={"list-type": "2", "prefix": prefix})
        root = ET.fromstring(payload)
        keys: list[str] = []
        for node in root.findall("{*}Contents"):
            key_node = node.find("{*}Key")
            if key_node is not None and key_node.text:
                keys.append(key_node.text)
        return keys


def _env_bool(name: str, default: bool = False) -> bool:
    value = os.getenv(name)
    if value is None:
        return default
    return value.strip().lower() in {"1", "true", "yes", "on"}


def build_object_store_from_env() -> tuple[ObjectStore | None, ObjectStoreStatus]:
    backend = os.getenv("OBJECT_STORE_BACKEND", "memory").strip().lower()

    if backend == "memory":
        store = InMemoryObjectStore()
        return store, ObjectStoreStatus(backend=store.backend, ready=True, degraded_reasons=[])

    if backend != "s3":
        return None, ObjectStoreStatus(
            backend=backend,
            ready=False,
            degraded_reasons=[f"OBJECT_STORE_UNSUPPORTED_BACKEND:{backend}"],
        )

    required_envs = {
        "S3_ENDPOINT": os.getenv("S3_ENDPOINT"),
        "S3_BUCKET": os.getenv("S3_BUCKET"),
        "S3_ACCESS_KEY": os.getenv("S3_ACCESS_KEY"),
        "S3_SECRET_KEY": os.getenv("S3_SECRET_KEY"),
        "S3_REGION": os.getenv("S3_REGION"),
    }
    missing = [name for name, value in required_envs.items() if not value]
    if missing:
        return None, ObjectStoreStatus(
            backend="s3",
            ready=False,
            degraded_reasons=[f"OBJECT_STORE_S3_MISSING_CONFIG:{name}" for name in missing],
        )

    config = S3ObjectStoreConfig(
        endpoint=required_envs["S3_ENDPOINT"],
        bucket=required_envs["S3_BUCKET"],
        access_key=required_envs["S3_ACCESS_KEY"],
        secret_key=required_envs["S3_SECRET_KEY"],
        region=required_envs["S3_REGION"],
        force_path_style=_env_bool("S3_FORCE_PATH_STYLE", default=True),
    )
    return S3ObjectStore(config), ObjectStoreStatus(backend="s3", ready=True, degraded_reasons=[])
