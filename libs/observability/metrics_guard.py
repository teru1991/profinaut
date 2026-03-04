from __future__ import annotations

from typing import Iterable

ALLOWED_LABELS = {
    "service",
    "op",
    "method",
    "status_class",
    "result",
    "venue",
    "status",
    "version",
    "git_sha",
}

FORBIDDEN_LABELS = {
    "symbol",
    "order_id",
    "client_order_id",
    "price",
    "qty",
    "size",
    "amount",
    "notional",
    "ip",
    "host",
    "hostname",
    "endpoint",
    "url",
    "trace_id",
    "run_id",
}


def validate_labels(labelnames: Iterable[str]) -> None:
    for raw in labelnames:
        label = str(raw)
        if label in FORBIDDEN_LABELS:
            raise ValueError(f"forbidden label detected: {label}")
        if label not in ALLOWED_LABELS:
            raise ValueError(f"unknown label (not allowed by SSOT): {label}")


def validate_catalog(metrics: list[dict]) -> None:
    for metric in metrics:
        name = metric.get("name", "")
        if not isinstance(name, str) or not name.startswith("profinaut_"):
            raise ValueError(f"metric name must start with profinaut_: {name}")
        labels = metric.get("labels", [])
        if not isinstance(labels, list):
            raise ValueError(f"labels must be list: {name}")
        validate_labels(labels)
