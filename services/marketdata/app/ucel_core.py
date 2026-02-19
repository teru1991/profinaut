from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Generic, TypeVar


SCHEMA_VERSION_V114 = "1.1.4"


class ErrorCode(str, Enum):
    NOT_SUPPORTED = "NOT_SUPPORTED"
    NOT_ALLOWED_OP = "NOT_ALLOWED_OP"
    MISSING_AUTH = "MISSING_AUTH"
    FEATURE_DISABLED = "FEATURE_DISABLED"
    DRY_RUN_ONLY = "DRY_RUN_ONLY"
    RATE_LIMITED = "RATE_LIMITED"
    AUTH_FAILED = "AUTH_FAILED"
    PERMISSION_DENIED = "PERMISSION_DENIED"
    INVALID_PARAMS = "INVALID_PARAMS"
    INVALID_ORDER = "INVALID_ORDER"
    TRANSPORT_ERROR = "TRANSPORT_ERROR"
    INTERNAL_ERROR = "INTERNAL_ERROR"


class CoreError(Exception):
    def __init__(self, code: ErrorCode, message: str, *, details: dict[str, Any] | None = None) -> None:
        super().__init__(message)
        self.error_code = code
        self.message = message
        self.details = details or {}

    def to_dict(self) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "error_code": self.error_code.value,
            "message": self.message,
        }
        if self.details:
            payload["details"] = self.details
        return payload


@dataclass(frozen=True)
class Quality:
    source: str = "exchange"
    is_replay: bool = False
    is_partial: bool = False
    latency_ms: int | None = None


@dataclass(frozen=True)
class Meta:
    venue: str
    symbol: str
    venue_symbol: str
    ts_event: datetime
    ts_recv: datetime
    schema_version: str = SCHEMA_VERSION_V114
    trace_id: str | None = None
    request_id: str | None = None
    run_id: str | None = None
    quality: Quality = field(default_factory=Quality)


T = TypeVar("T")


@dataclass(frozen=True)
class Envelope(Generic[T]):
    meta: Meta
    payload: T


@dataclass(frozen=True)
class TickerSnapshot:
    bid: float | None
    ask: float | None
    last: float | None
    volume_24h: float | None = None


@dataclass(frozen=True)
class TradeEvent:
    trade_id: str
    side: str
    price: float
    amount: float
    ts_event: datetime


@dataclass(frozen=True)
class OrderBookLevel:
    price: float
    amount: float


@dataclass(frozen=True)
class OrderBookSnapshot:
    bids: tuple[OrderBookLevel, ...]
    asks: tuple[OrderBookLevel, ...]
    sequence: int | None = None


@dataclass(frozen=True)
class OrderBookDelta:
    bids: tuple[OrderBookLevel, ...]
    asks: tuple[OrderBookLevel, ...]
    sequence: int | None = None


class OpName(str, Enum):
    FETCH_TICKER = "fetch_ticker"
    FETCH_TRADES = "fetch_trades"
    FETCH_ORDERBOOK_SNAPSHOT = "fetch_orderbook_snapshot"
    SUBSCRIBE_TICKER = "subscribe_ticker"
    SUBSCRIBE_TRADES = "subscribe_trades"
    SUBSCRIBE_ORDERBOOK = "subscribe_orderbook"
    FETCH_BALANCE = "fetch_balance"
    FETCH_BALANCES = "fetch_balances"
    PLACE_ORDER = "place_order"
    CANCEL_ORDER = "cancel_order"
    FETCH_ORDERS = "fetch_orders"
    FETCH_FILLS = "fetch_fills"


@dataclass(frozen=True)
class OpSpec:
    requires_auth: bool = False


OP_SPECS: dict[OpName, OpSpec] = {
    OpName.FETCH_TICKER: OpSpec(requires_auth=False),
    OpName.FETCH_TRADES: OpSpec(requires_auth=False),
    OpName.FETCH_ORDERBOOK_SNAPSHOT: OpSpec(requires_auth=False),
    OpName.SUBSCRIBE_TICKER: OpSpec(requires_auth=False),
    OpName.SUBSCRIBE_TRADES: OpSpec(requires_auth=False),
    OpName.SUBSCRIBE_ORDERBOOK: OpSpec(requires_auth=False),
    OpName.FETCH_BALANCE: OpSpec(requires_auth=True),
    OpName.FETCH_BALANCES: OpSpec(requires_auth=True),
    OpName.PLACE_ORDER: OpSpec(requires_auth=True),
    OpName.CANCEL_ORDER: OpSpec(requires_auth=True),
    OpName.FETCH_ORDERS: OpSpec(requires_auth=True),
    OpName.FETCH_FILLS: OpSpec(requires_auth=True),
}


@dataclass(frozen=True)
class Capabilities:
    venue: str
    supported_ops: frozenset[OpName]
    schema_version: str = SCHEMA_VERSION_V114


@dataclass(frozen=True)
class RuntimePolicy:
    allowed_ops: frozenset[OpName] | None = None
    policy_id: str | None = None
    failover_policy: str | None = None


@dataclass(frozen=True)
class ExecuteContext:
    trace_id: str | None = None
    request_id: str | None = None
    run_id: str | None = None
    has_auth: bool = False
    secret_ref: str | None = None
    actor_id: str | None = None
    features: frozenset[str] = field(default_factory=frozenset)
    live_trading: bool = False
    policy: RuntimePolicy | None = None


class Exchange(ABC):
    @abstractmethod
    def capabilities(self) -> Capabilities:
        raise NotImplementedError

    @abstractmethod
    def _execute_impl(self, op: OpName, params: dict[str, Any], ctx: ExecuteContext) -> Any:
        raise NotImplementedError

    def execute(self, op: OpName, params: dict[str, Any], ctx: ExecuteContext) -> Any:
        self._validate_preflight(op=op, ctx=ctx)
        return self._execute_impl(op=op, params=params, ctx=ctx)

    def fetch_ticker(self, *, symbol: str, ctx: ExecuteContext) -> Envelope[TickerSnapshot]:
        return self.execute(OpName.FETCH_TICKER, {"symbol": symbol}, ctx)

    def fetch_trades(self, *, symbol: str, limit: int, ctx: ExecuteContext) -> Envelope[tuple[TradeEvent, ...]]:
        return self.execute(OpName.FETCH_TRADES, {"symbol": symbol, "limit": limit}, ctx)

    def fetch_orderbook_snapshot(self, *, symbol: str, depth: int, ctx: ExecuteContext) -> Envelope[OrderBookSnapshot]:
        return self.execute(OpName.FETCH_ORDERBOOK_SNAPSHOT, {"symbol": symbol, "depth": depth}, ctx)

    def _validate_preflight(self, *, op: OpName, ctx: ExecuteContext) -> None:
        caps = self.capabilities()
        if op not in caps.supported_ops:
            raise CoreError(
                ErrorCode.NOT_SUPPORTED,
                f"Operation '{op.value}' is not declared in capabilities for venue '{caps.venue}'",
            )

        if ctx.policy and ctx.policy.allowed_ops is not None and op not in ctx.policy.allowed_ops:
            raise CoreError(
                ErrorCode.NOT_ALLOWED_OP,
                f"Operation '{op.value}' is blocked by runtime policy",
                details={"policy_id": ctx.policy.policy_id, "failover_policy": ctx.policy.failover_policy},
            )

        if OP_SPECS[op].requires_auth and not (ctx.has_auth or ctx.secret_ref):
            raise CoreError(
                ErrorCode.MISSING_AUTH,
                f"Operation '{op.value}' requires authentication",
            )
