from __future__ import annotations

import os
import time
from dataclasses import dataclass

from prometheus_client import CollectorRegistry, Counter, Gauge, Histogram, generate_latest

from libs.observability.metrics_guard import validate_labels

_PROCESS_START = time.time()
_HANDLES_BY_SERVICE: dict[str, "MetricsHandles"] = {}


@dataclass
class MetricsHandles:
    registry: CollectorRegistry
    build_info: Gauge
    uptime_seconds: Gauge
    health_status: Gauge
    capabilities_present: Gauge
    http_requests_total: Counter
    http_request_duration: Histogram
    execution_orders_total: Counter
    marketdata_frames_total: Counter


def _service_version() -> str:
    return (os.getenv("PROFINAUT_SERVICE_VERSION") or "unknown").strip() or "unknown"


def _git_sha() -> str:
    return (os.getenv("PROFINAUT_GIT_SHA") or "unknown").strip() or "unknown"


def ensure_metrics_initialized(service_name: str) -> MetricsHandles:
    if service_name in _HANDLES_BY_SERVICE:
        return _HANDLES_BY_SERVICE[service_name]

    registry = CollectorRegistry(auto_describe=True)

    validate_labels(["service", "version", "git_sha"])
    build_info = Gauge(
        "profinaut_build_info",
        "Build and version information (constant 1).",
        ["service", "version", "git_sha"],
        registry=registry,
    )
    build_info.labels(service=service_name, version=_service_version(), git_sha=_git_sha()).set(1)

    validate_labels(["service"])
    uptime_seconds = Gauge(
        "profinaut_uptime_seconds",
        "Process uptime in seconds.",
        ["service"],
        registry=registry,
    )
    uptime_seconds.labels(service=service_name).set(max(0.0, time.time() - _PROCESS_START))

    validate_labels(["service", "status"])
    health_status = Gauge(
        "profinaut_health_status",
        "Health state one-hot gauge by status label.",
        ["service", "status"],
        registry=registry,
    )
    for status in ["OK", "DEGRADED", "FAILED", "UNKNOWN"]:
        health_status.labels(service=service_name, status=status).set(1 if status == "UNKNOWN" else 0)

    capabilities_present = Gauge(
        "profinaut_capabilities_present",
        "Capabilities endpoint present and contract-valid (1=yes).",
        ["service"],
        registry=registry,
    )
    capabilities_present.labels(service=service_name).set(1)

    validate_labels(["service", "op", "method", "status_class"])
    http_requests_total = Counter(
        "profinaut_http_requests_total",
        "Total HTTP requests.",
        ["service", "op", "method", "status_class"],
        registry=registry,
    )

    validate_labels(["service", "op", "method"])
    http_request_duration = Histogram(
        "profinaut_http_request_duration_seconds",
        "HTTP request duration in seconds.",
        ["service", "op", "method"],
        registry=registry,
    )

    validate_labels(["service", "result"])
    execution_orders_total = Counter(
        "profinaut_execution_orders_total",
        "Execution orders observed/processed (result fixed set).",
        ["service", "result"],
        registry=registry,
    )

    validate_labels(["service", "venue", "result"])
    marketdata_frames_total = Counter(
        "profinaut_marketdata_frames_total",
        "Marketdata frames processed (venue fixed set; symbol forbidden).",
        ["service", "venue", "result"],
        registry=registry,
    )

    handles = MetricsHandles(
        registry=registry,
        build_info=build_info,
        uptime_seconds=uptime_seconds,
        health_status=health_status,
        capabilities_present=capabilities_present,
        http_requests_total=http_requests_total,
        http_request_duration=http_request_duration,
        execution_orders_total=execution_orders_total,
        marketdata_frames_total=marketdata_frames_total,
    )
    _HANDLES_BY_SERVICE[service_name] = handles
    return handles


def observe_http_request(service: str, op: str, method: str, status_code: int, duration_ms: int) -> None:
    handles = ensure_metrics_initialized(service)
    status_class = f"{int(status_code / 100)}xx"
    handles.http_requests_total.labels(
        service=service,
        op=op,
        method=method,
        status_class=status_class,
    ).inc()
    handles.http_request_duration.labels(service=service, op=op, method=method).observe(
        max(0.0, duration_ms / 1000.0)
    )
    handles.uptime_seconds.labels(service=service).set(max(0.0, time.time() - _PROCESS_START))


def expose_metrics_text(service: str) -> str:
    handles = ensure_metrics_initialized(service)
    handles.uptime_seconds.labels(service=service).set(max(0.0, time.time() - _PROCESS_START))
    return generate_latest(handles.registry).decode("utf-8")
