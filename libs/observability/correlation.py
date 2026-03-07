from __future__ import annotations

import os
import re
import uuid
from contextvars import ContextVar
from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from typing import Any, Mapping

from contracts.observability.contract_constants import (
    BUILD_SHA_ENV,
    HEADER_EVENT_ID,
    HEADER_INSTANCE_ID,
    HEADER_OBS_SCHEMA,
    HEADER_OP,
    HEADER_REQUEST_ID,
    HEADER_RUN_ID,
    HEADER_SCHEMA_VERSION,
    HEADER_TRACEPARENT,
    HEADER_TRACE_ID,
    INSTANCE_ID_ENV,
    SCHEMA_VERSION_CORRELATION,
    SERVICE_NAME_ENV,
    SERVICE_VERSION_ENV,
)

_TRACEPARENT_RE = re.compile(r"^[0-9a-f]{2}-([0-9a-f]{32})-([0-9a-f]{16})-[0-9a-f]{2}$")
_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:-]{7,127}$")
_instance_id_cached: str | None = None
_process_run_id = f"run-{uuid.uuid4()}"
_current_correlation: ContextVar[CorrelationContext | None] = ContextVar("correlation_ctx", default=None)


@dataclass(frozen=True)
class CorrelationContext:
    trace_id: str
    run_id: str
    component: str
    source: str
    schema_version: str
    op: str
    emitted_at: str
    instance_id: str
    request_id: str | None = None
    event_id: str | None = None

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["event_uid"] = self.event_id
        return payload


def now_utc_iso() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def new_uuid() -> str:
    return str(uuid.uuid4())


def new_process_run_id() -> str:
    return f"run-{uuid.uuid4()}"


def get_process_run_id() -> str:
    return _process_run_id


def is_valid_external_id(value: str | None) -> bool:
    return bool(value and _ID_RE.match(value.strip()))


def ensure_instance_id() -> str:
    global _instance_id_cached
    if _instance_id_cached is not None:
        return _instance_id_cached
    env_val = os.getenv(INSTANCE_ID_ENV)
    if env_val and is_valid_external_id(env_val):
        _instance_id_cached = env_val
        return _instance_id_cached
    _instance_id_cached = f"instance-{uuid.uuid4()}"
    return _instance_id_cached


def parse_traceparent(traceparent: str) -> str | None:
    if not traceparent:
        return None
    matched = _TRACEPARENT_RE.match(traceparent.strip())
    if not matched:
        return None
    return matched.group(1)


def _get_header(headers: Mapping[str, str], name: str) -> str | None:
    for key, value in headers.items():
        if key.lower() == name.lower() and value:
            return value.strip()
    return None


def current_correlation() -> CorrelationContext | None:
    return _current_correlation.get()


def bind_correlation(ctx: CorrelationContext):
    return _current_correlation.set(ctx)


def reset_correlation(token) -> None:
    _current_correlation.reset(token)


def set_current_correlation(ctx: CorrelationContext | None) -> None:
    _current_correlation.set(ctx)


def ensure_request_correlation(
    request_headers: Mapping[str, str],
    *,
    component: str,
    source: str,
    op: str = "http_request",
    trusted_run_id_override: bool = False,
    schema_version: str = SCHEMA_VERSION_CORRELATION,
) -> CorrelationContext:
    inbound_request_id = _get_header(request_headers, HEADER_REQUEST_ID)
    inbound_trace_id = _get_header(request_headers, HEADER_TRACE_ID)
    inbound_run_id = _get_header(request_headers, HEADER_RUN_ID)
    inbound_event_id = _get_header(request_headers, HEADER_EVENT_ID)
    traceparent = _get_header(request_headers, HEADER_TRACEPARENT)

    request_id = inbound_request_id if is_valid_external_id(inbound_request_id) else f"req-{uuid.uuid4()}"

    trace_id_from_tp = parse_traceparent(traceparent or "")
    trace_id = (
        inbound_trace_id
        if is_valid_external_id(inbound_trace_id)
        else (trace_id_from_tp if trace_id_from_tp else f"trace-{uuid.uuid4()}")
    )

    if trusted_run_id_override and is_valid_external_id(inbound_run_id):
        run_id = inbound_run_id or get_process_run_id()
    else:
        run_id = get_process_run_id()

    event_id = inbound_event_id if is_valid_external_id(inbound_event_id) else None

    return CorrelationContext(
        trace_id=trace_id,
        request_id=request_id,
        run_id=run_id,
        event_id=event_id,
        component=component,
        source=source,
        op=op,
        schema_version=schema_version,
        emitted_at=now_utc_iso(),
        instance_id=ensure_instance_id(),
    )


def merge_event_correlation(
    base: CorrelationContext | None,
    *,
    component: str,
    source: str,
    event_id: str | None,
    schema_version: str = SCHEMA_VERSION_CORRELATION,
    op: str = "event",
) -> CorrelationContext:
    resolved_event_id = event_id if is_valid_external_id(event_id) else f"evt-{uuid.uuid4()}"
    if base is None:
        return CorrelationContext(
            trace_id=f"trace-{uuid.uuid4()}",
            run_id=get_process_run_id(),
            component=component,
            source=source,
            schema_version=schema_version,
            op=op,
            emitted_at=now_utc_iso(),
            instance_id=ensure_instance_id(),
            event_id=resolved_event_id,
        )

    return CorrelationContext(
        trace_id=base.trace_id,
        run_id=base.run_id,
        component=base.component,
        source=base.source,
        schema_version=schema_version,
        op=op,
        emitted_at=now_utc_iso(),
        instance_id=base.instance_id,
        request_id=base.request_id,
        event_id=resolved_event_id,
    )


def to_response_headers(ctx: CorrelationContext, op_schema_versions: list[str] | None = None) -> dict[str, str]:
    headers = {
        HEADER_REQUEST_ID: ctx.request_id or "",
        HEADER_TRACE_ID: ctx.trace_id,
        HEADER_RUN_ID: ctx.run_id,
        HEADER_SCHEMA_VERSION: ctx.schema_version,
        HEADER_INSTANCE_ID: ctx.instance_id,
        HEADER_OP: ctx.op,
    }
    if ctx.event_id:
        headers[HEADER_EVENT_ID] = ctx.event_id
    if op_schema_versions:
        headers[HEADER_OBS_SCHEMA] = ",".join(op_schema_versions)
    return {k: v for k, v in headers.items() if v}


def set_correlation_response_headers(resp_headers: dict[str, str], corr: Mapping[str, Any], op_schema_versions: list[str]) -> None:
    ctx = CorrelationContext(
        trace_id=str(corr.get("trace_id") or f"trace-{uuid.uuid4()}"),
        run_id=str(corr.get("run_id") or get_process_run_id()),
        component=str(corr.get("component") or "unknown"),
        source=str(corr.get("source") or "unknown"),
        schema_version=str(corr.get("schema_version") or SCHEMA_VERSION_CORRELATION),
        op=str(corr.get("op") or "http_request"),
        emitted_at=str(corr.get("emitted_at") or now_utc_iso()),
        instance_id=str(corr.get("instance_id") or ensure_instance_id()),
        request_id=str(corr.get("request_id")) if corr.get("request_id") else None,
        event_id=str(corr.get("event_id")) if corr.get("event_id") else None,
    )
    resp_headers.update(to_response_headers(ctx, op_schema_versions))


def make_correlation(op: str, request_headers: Mapping[str, str]) -> dict[str, Any]:
    ctx = ensure_request_correlation(
        request_headers,
        component=os.getenv(SERVICE_NAME_ENV) or "unknown",
        source=os.getenv(SERVICE_NAME_ENV) or "unknown",
        op=op,
    )
    correlation = ctx.to_dict()

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
