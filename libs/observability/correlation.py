from __future__ import annotations

import os
import re
import uuid
from datetime import UTC, datetime
from typing import Mapping

from contracts.observability.contract_constants import (
    BUILD_SHA_ENV,
    HEADER_INSTANCE_ID,
    HEADER_OBS_SCHEMA,
    HEADER_OP,
    HEADER_RUN_ID,
    HEADER_TRACEPARENT,
    INSTANCE_ID_ENV,
    SCHEMA_VERSION_CORRELATION,
    SERVICE_NAME_ENV,
    SERVICE_VERSION_ENV,
)

_TRACEPARENT_RE = re.compile(r"^[0-9a-f]{2}-([0-9a-f]{32})-([0-9a-f]{16})-[0-9a-f]{2}$")
_instance_id_cached: str | None = None


def now_utc_iso() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def new_uuid() -> str:
    return str(uuid.uuid4())


def ensure_instance_id() -> str:
    global _instance_id_cached
    if _instance_id_cached is not None:
        return _instance_id_cached
    env_val = os.getenv(INSTANCE_ID_ENV)
    if env_val:
        try:
            uuid.UUID(env_val)
            _instance_id_cached = env_val
            return _instance_id_cached
        except ValueError:
            pass
    _instance_id_cached = new_uuid()
    return _instance_id_cached


def parse_traceparent(traceparent: str) -> str | None:
    if not traceparent:
        return None
    matched = _TRACEPARENT_RE.match(traceparent.strip())
    if not matched:
        return None
    return matched.group(1)


def _get_uuid_header(headers: Mapping[str, str], name: str) -> str | None:
    for key, value in headers.items():
        if key.lower() == name.lower():
            try:
                uuid.UUID(value)
                return value
            except ValueError:
                return None
    return None


def make_correlation(op: str, request_headers: Mapping[str, str]) -> dict:
    run_id = _get_uuid_header(request_headers, HEADER_RUN_ID) or new_uuid()
    instance_id = _get_uuid_header(request_headers, HEADER_INSTANCE_ID) or ensure_instance_id()

    trace_id = None
    for key, value in request_headers.items():
        if key.lower() == HEADER_TRACEPARENT.lower():
            trace_id = parse_traceparent(value)
            break

    correlation = {
        "schema_version": SCHEMA_VERSION_CORRELATION,
        "run_id": run_id,
        "instance_id": instance_id,
        "trace_id": trace_id,
        "event_uid": None,
        "op": op,
        "emitted_at": now_utc_iso(),
    }

    build: dict[str, str] = {}
    git_sha = os.getenv(BUILD_SHA_ENV)
    version = os.getenv(SERVICE_VERSION_ENV)
    if git_sha:
        build["git_sha"] = git_sha
    if version:
        build["version"] = version
    if build:
        correlation["build"] = build

    env: dict[str, str] = {}
    service_name = os.getenv(SERVICE_NAME_ENV)
    service_version = os.getenv(SERVICE_VERSION_ENV)
    if service_name:
        env["service_name"] = service_name
    if service_version:
        env["service_version"] = service_version
    if env:
        correlation["env"] = env

    return correlation


def set_correlation_response_headers(resp_headers: dict[str, str], corr: dict, op_schema_versions: list[str]) -> None:
    resp_headers[HEADER_RUN_ID] = corr["run_id"]
    resp_headers[HEADER_INSTANCE_ID] = corr["instance_id"]
    resp_headers[HEADER_OP] = corr["op"]
    resp_headers[HEADER_OBS_SCHEMA] = ",".join(op_schema_versions)
