from __future__ import annotations

import logging
import os
from dataclasses import dataclass
from typing import Any, Callable, Mapping

logger = logging.getLogger("marketdata")


class OpName:
    """Single source of truth for GMO operation names used for policy/audit/metrics."""

    # MarketData REST
    FETCH_TICKER = "fetch_ticker"
    FETCH_TRADES = "fetch_trades"
    FETCH_ORDERBOOK_SNAPSHOT = "fetch_orderbook_snapshot"
    FETCH_OHLCV = "fetch_ohlcv"
    FETCH_MARKETS = "fetch_markets"

    # MarketData WS
    SUBSCRIBE_TICKER = "subscribe_ticker"
    SUBSCRIBE_TRADES = "subscribe_trades"
    SUBSCRIBE_ORDERBOOK = "subscribe_orderbook"

    # Private REST
    FETCH_BALANCES = "fetch_balances"
    PLACE_ORDER = "place_order"
    CANCEL_ORDER = "cancel_order"
    FETCH_ORDERS = "fetch_orders"
    FETCH_FILLS = "fetch_fills"

    # Private WS
    SUBSCRIBE_ORDERS = "subscribe_orders"
    SUBSCRIBE_FILLS = "subscribe_fills"
    SUBSCRIBE_BALANCES = "subscribe_balances"


ALL_OPS: frozenset[str] = frozenset(
    value for name, value in vars(OpName).items() if name.isupper() and isinstance(value, str)
)

PRIVATE_OPS: frozenset[str] = frozenset(
    {
        OpName.FETCH_BALANCES,
        OpName.PLACE_ORDER,
        OpName.CANCEL_ORDER,
        OpName.FETCH_ORDERS,
        OpName.FETCH_FILLS,
        OpName.SUBSCRIBE_ORDERS,
        OpName.SUBSCRIBE_FILLS,
        OpName.SUBSCRIBE_BALANCES,
    }
)

DANGEROUS_EXECUTION_OPS: frozenset[str] = frozenset({OpName.PLACE_ORDER, OpName.CANCEL_ORDER})


class AdapterPolicyError(RuntimeError):
    def __init__(self, code: str, message: str, *, details: Mapping[str, Any] | None = None):
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = dict(details or {})


@dataclass(frozen=True)
class GmoCapabilities:
    """Capabilities skeleton: anything not listed is treated as NotSupported."""

    supported_ops: frozenset[str]

    def supports(self, op: str) -> bool:
        return op in self.supported_ops


@dataclass(frozen=True)
class GmoAdapterPolicy:
    allowed_ops: frozenset[str]
    policy_id: str | None = None
    execution_enabled: bool = False


@dataclass(frozen=True)
class GmoAuthContext:
    key_id: str


TransportFn = Callable[[str, Mapping[str, Any] | None], Any]
AuthResolverFn = Callable[[], GmoAuthContext | None]


def requires_auth(op: str) -> bool:
    return op in PRIVATE_OPS


class GmoAdapterFacade:
    def __init__(
        self,
        *,
        transport: TransportFn,
        policy: GmoAdapterPolicy,
        capabilities: GmoCapabilities,
        auth_resolver: AuthResolverFn | None = None,
    ):
        self._transport = transport
        self._policy = policy
        self._capabilities = capabilities
        self._auth_resolver = auth_resolver or (lambda: None)

    @staticmethod
    def policy_from_env() -> GmoAdapterPolicy:
        raw_allowed = os.getenv("GMO_ALLOWED_OPS", "")
        allowed_ops = frozenset(value.strip() for value in raw_allowed.split(",") if value.strip())
        policy_id = os.getenv("GMO_POLICY_ID") or None
        execution_enabled = os.getenv("GMO_FEATURE_EXECUTION", "0").strip() == "1"
        return GmoAdapterPolicy(allowed_ops=allowed_ops, policy_id=policy_id, execution_enabled=execution_enabled)

    def execute(self, op: str, payload: Mapping[str, Any] | None = None) -> Any:
        if not self._capabilities.supports(op):
            raise AdapterPolicyError(
                "NOT_SUPPORTED",
                f"Operation '{op}' is not supported by current GMO capabilities",
                details={"op": op},
            )

        if op not in self._policy.allowed_ops:
            self._log_preflight_reject(op, "NOT_ALLOWED_OP")
            raise AdapterPolicyError(
                "NOT_ALLOWED_OP",
                f"Operation '{op}' is not allowed by runtime policy",
                details={"op": op, "policy_id": self._policy.policy_id},
            )

        if op in DANGEROUS_EXECUTION_OPS and not self._policy.execution_enabled:
            self._log_preflight_reject(op, "FEATURE_DISABLED")
            raise AdapterPolicyError(
                "FEATURE_DISABLED",
                f"Operation '{op}' requires execution feature",
                details={"op": op, "feature": "execution", "policy_id": self._policy.policy_id},
            )

        if requires_auth(op):
            auth_ctx = self._auth_resolver()
            if auth_ctx is None:
                self._log_preflight_reject(op, "MISSING_AUTH")
                raise AdapterPolicyError(
                    "MISSING_AUTH",
                    f"Operation '{op}' requires credentials and was rejected before upstream send",
                    details={"op": op, "preflight_reject": True, "policy_id": self._policy.policy_id},
                )
            logger.info("gmo_auth_context_selected op=%s key_id=%s", op, auth_ctx.key_id)

        return self._transport(op, payload)

    def _log_preflight_reject(self, op: str, code: str) -> None:
        logger.info(
            "gmo_preflight_reject code=%s op=%s preflight_reject=true policy_id=%s",
            code,
            op,
            self._policy.policy_id,
        )


DEFAULT_GMO_CAPABILITIES = GmoCapabilities(
    supported_ops=frozenset(
        {
            OpName.FETCH_TICKER,
            OpName.FETCH_TRADES,
            OpName.FETCH_ORDERBOOK_SNAPSHOT,
            OpName.FETCH_OHLCV,
            OpName.FETCH_MARKETS,
            OpName.SUBSCRIBE_TICKER,
            OpName.SUBSCRIBE_TRADES,
            OpName.SUBSCRIBE_ORDERBOOK,
        }
    )
)
