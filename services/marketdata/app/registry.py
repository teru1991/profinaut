from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from services.marketdata.app.ucel_core import OpName


_REPO_ROOT = Path(__file__).resolve().parents[3]
_DOCS_EXCHANGES_ROOT = _REPO_ROOT / "docs" / "exchanges"


class CatalogValidationError(RuntimeError):
    pass


@dataclass(frozen=True)
class ConnectionPolicy:
    allowed_ops: frozenset[OpName] | None = None
    failover_policy: str | None = None
    key_scope: str | None = None


@dataclass(frozen=True)
class RegistryConnection:
    connection_id: str
    venue: str
    source: str
    op: OpName | None
    requires_auth: bool
    supported: bool
    allowed_ops: frozenset[OpName] | None
    failover_policy: str | None
    key_scope: str | None


@dataclass(frozen=True)
class VenueRegistry:
    venue: str
    catalog_path: str
    connections: tuple[RegistryConnection, ...]

    @property
    def capabilities(self) -> dict[str, dict[str, bool | str]]:
        rows: dict[str, dict[str, bool | str]] = {}
        for op in OpName:
            supported = any(conn.op == op and conn.supported for conn in self.connections)
            requires_auth = any(conn.op == op and conn.requires_auth for conn in self.connections)
            rows[op.value] = {
                "supported": supported,
                "requires_auth": requires_auth,
                "source": "docs_catalog",
            }
        return rows


def _validate_typed_fields(value: Any, path: str) -> None:
    if isinstance(value, dict):
        field_type = value.get("type")
        if field_type is not None and not isinstance(field_type, str):
            raise CatalogValidationError(f"{path}.type must be string")
        for key, child in value.items():
            _validate_typed_fields(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _validate_typed_fields(child, f"{path}[{index}]")


def _ensure_required(record: dict[str, Any], required: tuple[str, ...], path: str) -> None:
    for field in required:
        value = record.get(field)
        if not isinstance(value, str) or not value.strip():
            raise CatalogValidationError(f"{path}.{field} must be non-empty string")


def _map_op(source: str, record_id: str, operation: str | None) -> OpName | None:
    text = f"{record_id} {operation or ''}".lower()
    if "ticker" in text:
        return OpName.FETCH_TICKER if source == "rest" else OpName.SUBSCRIBE_TICKER
    if "trade" in text or "execution" in text:
        return OpName.FETCH_TRADES if source == "rest" else OpName.SUBSCRIBE_TRADES
    if "orderbook" in text or "orderbooks" in text:
        return OpName.FETCH_ORDERBOOK_SNAPSHOT if source == "rest" else OpName.SUBSCRIBE_ORDERBOOK
    if "balance" in text or "asset" in text:
        return OpName.FETCH_BALANCE
    return None


def _is_supported(op: OpName | None) -> bool:
    implemented = {
        OpName.FETCH_TICKER,
        OpName.SUBSCRIBE_TICKER,
        OpName.SUBSCRIBE_TRADES,
        OpName.SUBSCRIBE_ORDERBOOK,
    }
    return op in implemented


def _coerce_allowed_ops(raw_ops: list[str] | None, connection_id: str) -> frozenset[OpName] | None:
    if raw_ops is None:
        return None
    out: set[OpName] = set()
    for raw in raw_ops:
        try:
            out.add(OpName(raw))
        except ValueError as exc:
            raise CatalogValidationError(f"policy for {connection_id} has unknown op '{raw}'") from exc
    return frozenset(out)


def _load_policy_overrides() -> dict[str, ConnectionPolicy]:
    raw = os.getenv("MARKETDATA_CONNECTION_POLICIES")
    if not raw:
        return {}
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise CatalogValidationError("MARKETDATA_CONNECTION_POLICIES must be valid JSON") from exc
    if not isinstance(payload, dict):
        raise CatalogValidationError("MARKETDATA_CONNECTION_POLICIES must be an object")

    overrides: dict[str, ConnectionPolicy] = {}
    for connection_id, rule in payload.items():
        if not isinstance(rule, dict):
            raise CatalogValidationError(f"policy for {connection_id} must be object")
        allowed_ops_raw = rule.get("allowed_ops")
        if allowed_ops_raw is not None and not isinstance(allowed_ops_raw, list):
            raise CatalogValidationError(f"policy for {connection_id}.allowed_ops must be list")
        overrides[connection_id] = ConnectionPolicy(
            allowed_ops=_coerce_allowed_ops(allowed_ops_raw, connection_id),
            failover_policy=rule.get("failover_policy"),
            key_scope=rule.get("key_scope"),
        )
    return overrides


def load_venue_registry(venue: str) -> VenueRegistry:
    catalog_path = _DOCS_EXCHANGES_ROOT / venue / "catalog.json"
    if not catalog_path.exists():
        raise CatalogValidationError(f"catalog not found: {catalog_path}")

    payload = json.loads(catalog_path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise CatalogValidationError("catalog root must be object")
    _ensure_required(payload, ("exchange",), "catalog")

    seen_ids: set[str] = set()
    connections: list[RegistryConnection] = []
    policy_overrides = _load_policy_overrides()

    groups: tuple[tuple[str, str, tuple[str, ...]], ...] = (
        ("rest", "rest_endpoints", ("id", "visibility", "method", "path")),
        ("ws", "ws_channels", ("id", "visibility", "channel", "ws_url")),
        ("feed", "data_feeds", ("id",)),
    )

    for source, key, required in groups:
        records = payload.get(key)
        if not isinstance(records, list):
            raise CatalogValidationError(f"catalog.{key} must be array")
        for index, record in enumerate(records):
            path = f"catalog.{key}[{index}]"
            if not isinstance(record, dict):
                raise CatalogValidationError(f"{path} must be object")
            _ensure_required(record, required, path)
            record_id = record["id"]
            if record_id in seen_ids:
                raise CatalogValidationError(f"duplicate id: {record_id}")
            seen_ids.add(record_id)

            visibility_raw = record.get("visibility", "public" if source == "feed" else None)
            if not isinstance(visibility_raw, str) or not visibility_raw.strip():
                raise CatalogValidationError(f"{path}.visibility must be non-empty string")
            visibility = visibility_raw.lower()
            if visibility not in {"public", "private"}:
                raise CatalogValidationError(f"{path}.visibility must be public/private")

            _validate_typed_fields(record, path)
            op = _map_op(source=source, record_id=record_id, operation=record.get("operation"))
            policy = policy_overrides.get(record_id)
            default_allowed_ops = frozenset({op}) if op is not None else None

            connections.append(
                RegistryConnection(
                    connection_id=record_id,
                    venue=venue,
                    source=source,
                    op=op,
                    requires_auth=visibility == "private",
                    supported=_is_supported(op),
                    allowed_ops=policy.allowed_ops if policy else default_allowed_ops,
                    failover_policy=policy.failover_policy if policy else None,
                    key_scope=policy.key_scope if policy else None,
                )
            )

    return VenueRegistry(venue=venue, catalog_path=str(catalog_path), connections=tuple(connections))
