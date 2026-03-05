from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from app.i_events import append_event


@dataclass(frozen=True)
class ReconcileReport:
    severity: str
    diffs: list[dict[str, Any]]
    suggested_mode: str


def reconcile_open_orders(conn, local_orders: dict[str, dict[str, Any]], snapshot_open_client_ids: set[str]) -> ReconcileReport:
    diffs: list[dict[str, Any]] = []
    for cid, order in local_orders.items():
        if order.get("state") == "UNKNOWN" and cid not in snapshot_open_client_ids:
            append_event(conn, "RECONCILE_CONVERGE", {"client_order_id": cid, "to": "CANCELED"})
            diffs.append({"client_order_id": cid, "from": "UNKNOWN", "to": "CANCELED"})
    severity = "INFO" if not diffs else "WARN"
    suggested = "SAFE" if not diffs else "CANCEL_ONLY"
    return ReconcileReport(severity=severity, diffs=diffs, suggested_mode=suggested)
