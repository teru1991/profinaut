from __future__ import annotations

import gzip
import json
import os
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any
from uuid import uuid4

from services.marketdata.app.hash_util import compute_payload_hash
from services.marketdata.app.object_store import ObjectStore


@dataclass
class RawIngestMeta:
    raw_msg_id: str
    object_key: str
    payload_hash: str | None
    received_ts: str
    quality_json: dict[str, Any] | None
    content_encoding: str | None
    content_type: str | None
    object_size: int | None


class RawMetaRepository:
    """In-memory meta repository used by this service version."""

    def __init__(self) -> None:
        self._rows: dict[str, RawIngestMeta] = {}

    def save(self, row: RawIngestMeta) -> None:
        self._rows[row.raw_msg_id] = row

    def get(self, raw_msg_id: str) -> RawIngestMeta | None:
        return self._rows.get(raw_msg_id)


class BronzeStore:
    def __init__(
        self,
        object_store: ObjectStore,
        meta_repository: RawMetaRepository,
        *,
        gzip_enabled: bool | None = None,
    ) -> None:
        self._object_store = object_store
        self._meta = meta_repository
        if gzip_enabled is None:
            gzip_enabled = os.getenv("BRONZE_GZIP_ENABLED", "true").strip().lower() in {"1", "true", "yes", "on"}
        self._gzip_enabled = gzip_enabled

    @staticmethod
    def _object_key(raw_msg_id: str, received_at: datetime, gzip_enabled: bool) -> str:
        base = f"bronze/raw/{received_at.strftime('%Y/%m/%d')}/{raw_msg_id}.jsonl"
        return f"{base}.gz" if gzip_enabled else base

    def ingest_json(self, payload: dict[str, Any], quality_json: dict[str, Any] | None = None) -> RawIngestMeta:
        raw_msg_id = str(uuid4())
        received_at = datetime.now(UTC)
        line = (json.dumps(payload, separators=(",", ":"), sort_keys=True) + "\n").encode("utf-8")

        if self._gzip_enabled:
            body = gzip.compress(line)
            content_encoding = "gzip"
            content_type = "application/x-ndjson"
        else:
            body = line
            content_encoding = None
            content_type = "application/x-ndjson"

        object_key = self._object_key(raw_msg_id=raw_msg_id, received_at=received_at, gzip_enabled=self._gzip_enabled)
        self._object_store.put_object(object_key, body, content_type=content_type)

        meta = RawIngestMeta(
            raw_msg_id=raw_msg_id,
            object_key=object_key,
            payload_hash=compute_payload_hash(payload),
            received_ts=received_at.isoformat(),
            quality_json=quality_json,
            content_encoding=content_encoding,
            content_type=content_type,
            object_size=len(body),
        )
        self._meta.save(meta)
        return meta

    def replay_payload(self, object_key: str) -> list[dict[str, Any]]:
        data = self._object_store.get_object(object_key)
        if object_key.endswith(".gz"):
            decoded = gzip.decompress(data).decode("utf-8")
        else:
            decoded = data.decode("utf-8")

        items: list[dict[str, Any]] = []
        for line in decoded.splitlines():
            stripped = line.strip()
            if not stripped:
                continue
            items.append(json.loads(stripped))
        return items
