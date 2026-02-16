from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Literal

StorageBackend = Literal["fs", "s3"]


@dataclass(frozen=True)
class ServiceSettings:
    db_dsn: str | None
    object_store_backend: StorageBackend | None
    ingest_raw_enabled: bool
    silver_enabled: bool
    degraded: bool
    degraded_reasons: list[str]


_ALLOWED_STORAGE_BACKENDS = {"fs", "s3"}


def _normalized_env(name: str) -> str | None:
    value = os.getenv(name)
    if value is None:
        return None
    value = value.strip()
    return value or None


def load_settings() -> ServiceSettings:
    db_dsn = _normalized_env("DB_DSN")
    storage_backend_raw = _normalized_env("OBJECT_STORE_BACKEND")

    object_store_backend: StorageBackend | None = None
    degraded_reasons: list[str] = []

    if storage_backend_raw is None:
        degraded_reasons.append("STORAGE_NOT_CONFIGURED")
    else:
        storage_backend = storage_backend_raw.lower()
        if storage_backend not in _ALLOWED_STORAGE_BACKENDS:
            degraded_reasons.append("STORAGE_BACKEND_INVALID")
        else:
            object_store_backend = storage_backend  # type: ignore[assignment]

    if db_dsn is None:
        degraded_reasons.append("DB_NOT_CONFIGURED")

    ingest_raw_enabled = object_store_backend is not None and db_dsn is not None
    silver_enabled = bool(_normalized_env("SILVER_ENABLED") == "1")
    degraded = len(degraded_reasons) > 0

    return ServiceSettings(
        db_dsn=db_dsn,
        object_store_backend=object_store_backend,
        ingest_raw_enabled=ingest_raw_enabled,
        silver_enabled=silver_enabled,
        degraded=degraded,
        degraded_reasons=degraded_reasons,
    )
