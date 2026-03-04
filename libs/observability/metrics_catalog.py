from __future__ import annotations

METRICS_CATALOG_VERSION = "obs.metrics_catalog.v1"

METRICS = [
    {
        "name": "profinaut_build_info",
        "type": "gauge",
        "unit": "info",
        "help": "Build and version information (constant 1).",
        "labels": ["service", "version", "git_sha"],
    },
    {
        "name": "profinaut_uptime_seconds",
        "type": "gauge",
        "unit": "seconds",
        "help": "Process uptime in seconds.",
        "labels": ["service"],
    },
    {
        "name": "profinaut_health_status",
        "type": "gauge",
        "unit": "state",
        "help": "Health state one-hot gauge by status label.",
        "labels": ["service", "status"],
    },
    {
        "name": "profinaut_capabilities_present",
        "type": "gauge",
        "unit": "bool",
        "help": "Capabilities endpoint present and contract-valid (1=yes).",
        "labels": ["service"],
    },
    {
        "name": "profinaut_http_requests_total",
        "type": "counter",
        "unit": "total",
        "help": "Total HTTP requests.",
        "labels": ["service", "op", "method", "status_class"],
    },
    {
        "name": "profinaut_http_request_duration_seconds",
        "type": "histogram",
        "unit": "seconds",
        "help": "HTTP request duration in seconds.",
        "labels": ["service", "op", "method"],
    },
    {
        "name": "profinaut_execution_orders_total",
        "type": "counter",
        "unit": "total",
        "help": "Execution orders observed/processed (result fixed set).",
        "labels": ["service", "result"],
    },
    {
        "name": "profinaut_marketdata_frames_total",
        "type": "counter",
        "unit": "total",
        "help": "Marketdata frames processed (venue fixed set; symbol forbidden).",
        "labels": ["service", "venue", "result"],
    },
]
