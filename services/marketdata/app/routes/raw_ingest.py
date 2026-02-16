from __future__ import annotations

import hashlib
import json
import os
import random
import sqlite3
import time
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

from fastapi import APIRouter, Request
from fastapi.responses import JSONResponse

from services.marketdata.app.bronze.writer import BronzeWriter
from services.marketdata.app.db.repository import MarketDataMetaRepository, RawIngestMeta
from services.marketdata.app.db.schema import apply_migrations
from services.marketdata.app.logging import scrub_sensitive_fields
from services.marketdata.app.metrics import ingest_metrics
from services.marketdata.app.settings import load_settings
from services.marketdata.app.silver.normalizer import normalize_envelope
from services.marketdata.app.storage.fs_store import FilesystemObjectStore

router = APIRouter()

_CROCKFORD32 = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"


def _encode_crockford(value: int, length: int) -> str:
    chars: list[str] = []
    for _ in range(length):
        chars.append(_CROCKFORD32[value & 31])
        value >>= 5
    return "".join(reversed(chars))


def _new_ulid() -> str:
    timestamp_ms = int(time.time() * 1000)
    random_80 = random.getrandbits(80)
    return f"{_encode_crockford(timestamp_ms, 10)}{_encode_crockford(random_80, 16)}"


def _canonical_payload_hash(payload_json: dict[str, Any]) -> str:
    canonical = json.dumps(payload_json, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def _parse_rfc3339(ts: str) -> datetime:
    return datetime.fromisoformat(ts.replace("Z", "+00:00")).astimezone(UTC)


def _derive_event_ts_quality(event_ts: object, received_ts: str) -> tuple[str, float | None]:
    received = _parse_rfc3339(received_ts)
    if isinstance(event_ts, str) and event_ts.strip():
        try:
            event = _parse_rfc3339(event_ts)
            lag_ms = max((received - event).total_seconds() * 1000, 0.0)
            return "exchange_provided", lag_ms
        except ValueError:
            return "derived", 0.0
    if event_ts is None:
        return "missing", None
    return "derived", 0.0


def _resolve_sqlite_path(db_dsn: str) -> str:
    if not db_dsn.startswith("sqlite:///"):
        raise ValueError("Only sqlite:/// DSN is supported in v0.1")
    return db_dsn.removeprefix("sqlite:///")


def _connect_repo(db_dsn: str) -> MarketDataMetaRepository:
    db_path = _resolve_sqlite_path(db_dsn)
    if db_path != ":memory:":
        Path(db_path).parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA foreign_keys = ON")
    apply_migrations(conn)
    return MarketDataMetaRepository(conn)


@router.post("/raw/ingest")
async def raw_ingest(request: Request) -> JSONResponse:
    settings = load_settings()

    if settings.object_store_backend is None:
        ingest_metrics.record_failure()
        return JSONResponse(
            status_code=503,
            content={"error": "INGEST_DISABLED", "reason": "STORAGE_NOT_CONFIGURED"},
        )
    if settings.db_dsn is None:
        ingest_metrics.record_failure()
        return JSONResponse(
            status_code=503,
            content={"error": "INGEST_DISABLED", "reason": "DB_NOT_CONFIGURED"},
        )

    body = await request.json()
    if not isinstance(body, dict):
        ingest_metrics.record_failure()
        return JSONResponse(status_code=400, content={"error": "INVALID_REQUEST", "reason": "BODY_MUST_BE_OBJECT"})

    for required in ("tenant_id", "source_type", "received_ts", "payload_json"):
        if required not in body:
            ingest_metrics.record_failure()
            return JSONResponse(status_code=400, content={"error": "INVALID_REQUEST", "reason": f"MISSING_{required.upper()}"})

    if not isinstance(body["payload_json"], dict):
        ingest_metrics.record_failure()
        return JSONResponse(
            status_code=400,
            content={"error": "INVALID_REQUEST", "reason": "PAYLOAD_JSON_MUST_BE_OBJECT"},
        )

    envelope = dict(body)
    envelope.setdefault("raw_msg_id", _new_ulid())
    envelope["payload_hash"] = _canonical_payload_hash(envelope["payload_json"])
    envelope.setdefault("quality_json", {})
    envelope.setdefault("parser_version", "v0.1")
    envelope.setdefault("event_ts", None)
    envelope.setdefault("source_msg_key", None)
    envelope.setdefault("seq", None)
    envelope.setdefault("venue_id", None)
    envelope.setdefault("market_id", None)
    envelope.setdefault("stream_name", None)
    envelope.setdefault("endpoint", None)
    envelope.setdefault("symbol_raw", None)
    envelope.setdefault("instrument_id", None)

    try:
        _ = _parse_rfc3339(str(envelope["received_ts"]))
    except ValueError:
        ingest_metrics.record_failure()
        return JSONResponse(
            status_code=400,
            content={"error": "INVALID_REQUEST", "reason": "RECEIVED_TS_INVALID_RFC3339"},
        )

    repo = _connect_repo(settings.db_dsn)

    duplicate_reasons: list[str] = []
    window_start = (_parse_rfc3339(str(envelope["received_ts"])) - timedelta(minutes=5)).isoformat().replace("+00:00", "Z")
    if repo.count_payload_hash_since(str(envelope["payload_hash"]), since_ts=window_start) > 0:
        duplicate_reasons.append("payload_hash_seen")

    source_msg_key = envelope.get("source_msg_key")
    if isinstance(source_msg_key, str) and source_msg_key:
        if repo.exists_source_msg_key_since(
            venue_id=None if envelope.get("venue_id") is None else str(envelope.get("venue_id")),
            market_id=None if envelope.get("market_id") is None else str(envelope.get("market_id")),
            source_msg_key=source_msg_key,
            since_ts=window_start,
        ):
            duplicate_reasons.append("source_msg_key_seen")

    event_ts_quality, lag_ms = _derive_event_ts_quality(envelope.get("event_ts"), str(envelope["received_ts"]))
    quality_json = dict(envelope.get("quality_json") or {})
    quality_json["dup_suspect"] = len(duplicate_reasons) > 0
    quality_json["dup_reason"] = duplicate_reasons[0] if duplicate_reasons else None
    quality_json["event_ts_quality"] = event_ts_quality
    quality_json["lag_ms"] = lag_ms
    envelope["quality_json"] = quality_json

    fs_root = os.getenv("BRONZE_FS_ROOT", "./data/bronze")
object_key = _BRONZE_WRITER.append(envelope)

    repo.insert_raw_ingest_meta(
        RawIngestMeta(
            raw_msg_id=envelope["raw_msg_id"],
            tenant_id=str(envelope["tenant_id"]),
            source_type=envelope.get("source_type"),
            venue_id=envelope.get("venue_id"),
            market_id=envelope.get("market_id"),
            stream_name=envelope.get("stream_name"),
            endpoint=envelope.get("endpoint"),
            event_ts=envelope.get("event_ts"),
            received_ts=str(envelope["received_ts"]),
            seq=None if envelope.get("seq") is None else str(envelope.get("seq")),
            source_msg_key=envelope.get("source_msg_key"),
            payload_hash=envelope.get("payload_hash"),
            payload_size=len(json.dumps(envelope["payload_json"], separators=(",", ":"), ensure_ascii=False).encode("utf-8")),
            object_key=object_key,
            quality_json=dict(envelope.get("quality_json") or {}),
            parser_version=envelope.get("parser_version"),
        )
    )

    normalized_target = None
    normalized_event_type = None
    if settings.silver_enabled:
        normalized = normalize_envelope(repo, envelope)
        normalized_target = normalized.target
        normalized_event_type = normalized.event_type

    # Deliberately scrub payload body from logs.
    _ = scrub_sensitive_fields(envelope)

    dup_suspect = bool(quality_json.get("dup_suspect"))
    ingest_metrics.record_success(dup_suspect=dup_suspect)

    return JSONResponse(
        status_code=200,
        content={
            "raw_msg_id": envelope["raw_msg_id"],
            "object_key": object_key,
            "stored": True,
            "dup_suspect": dup_suspect,
            "degraded": False,
            "normalized_target": normalized_target,
            "normalized_event_type": normalized_event_type,
        },
    )
