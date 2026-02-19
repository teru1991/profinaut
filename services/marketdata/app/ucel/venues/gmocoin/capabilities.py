from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from services.marketdata.app.registry import load_venue_registry
from services.marketdata.app.ucel_core import Capabilities, OpName

SUPPORTED_PUBLIC_OPS: frozenset[OpName] = frozenset(
    {
        OpName.FETCH_TICKER,
        OpName.FETCH_TRADES,
        OpName.FETCH_ORDERBOOK_SNAPSHOT,
        OpName.SUBSCRIBE_TICKER,
        OpName.SUBSCRIBE_TRADES,
        OpName.SUBSCRIBE_ORDERBOOK,
    }
)


@dataclass(frozen=True)
class CatalogScope:
    rest_by_op: dict[OpName, dict[str, Any]]
    ws_by_op: dict[OpName, dict[str, Any]]
    unsupported_catalog_ids: tuple[str, ...]


def build_scope_from_catalog(catalog: dict[str, Any]) -> CatalogScope:
    registry = load_venue_registry("gmocoin")
    id_to_rest = {item["id"]: item for item in catalog.get("rest_endpoints", []) if isinstance(item, dict)}
    id_to_ws = {item["id"]: item for item in catalog.get("ws_channels", []) if isinstance(item, dict)}

    rest_by_op: dict[OpName, dict[str, Any]] = {}
    ws_by_op: dict[OpName, dict[str, Any]] = {}
    unsupported: list[str] = []
    for conn in registry.connections:
        if conn.requires_auth or conn.op is None:
            unsupported.append(conn.connection_id)
            continue
        if conn.op in SUPPORTED_PUBLIC_OPS:
            if conn.source == "rest" and conn.connection_id in id_to_rest:
                rest_by_op[conn.op] = id_to_rest[conn.connection_id]
            elif conn.source == "ws" and conn.connection_id in id_to_ws:
                ws_by_op[conn.op] = id_to_ws[conn.connection_id]
        else:
            unsupported.append(conn.connection_id)

    return CatalogScope(
        rest_by_op=rest_by_op,
        ws_by_op=ws_by_op,
        unsupported_catalog_ids=tuple(sorted(set(unsupported))),
    )


def to_capabilities(scope: CatalogScope) -> Capabilities:
    ops = set(scope.rest_by_op.keys()) | set(scope.ws_by_op.keys())
    return Capabilities(venue="gmocoin", supported_ops=frozenset(ops))
