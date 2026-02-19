from __future__ import annotations

from typing import Any

import pytest

from services.marketdata.app.ucel_core import (
    Capabilities,
    CoreError,
    ErrorCode,
    Exchange,
    ExecuteContext,
    OpName,
    RuntimePolicy,
)


class DummyExchange(Exchange):
    def __init__(self, supported_ops: frozenset[OpName]) -> None:
        self._caps = Capabilities(venue="dummy", supported_ops=supported_ops)

    def capabilities(self) -> Capabilities:
        return self._caps

    def _execute_impl(self, op: OpName, params: dict[str, Any], ctx: ExecuteContext) -> Any:
        return {"op": op.value, "params": params, "run_id": ctx.run_id}


def test_operation_not_in_capabilities_returns_not_supported() -> None:
    exchange = DummyExchange(supported_ops=frozenset({OpName.FETCH_TICKER}))

    with pytest.raises(CoreError) as exc:
        exchange.execute(
            OpName.FETCH_TRADES,
            params={"symbol": "BTC_JPY"},
            ctx=ExecuteContext(),
        )

    assert exc.value.error_code == ErrorCode.NOT_SUPPORTED


def test_runtime_policy_block_returns_not_allowed_op() -> None:
    exchange = DummyExchange(supported_ops=frozenset({OpName.FETCH_TICKER, OpName.FETCH_TRADES}))

    with pytest.raises(CoreError) as exc:
        exchange.execute(
            OpName.FETCH_TRADES,
            params={"symbol": "BTC_JPY"},
            ctx=ExecuteContext(
                policy=RuntimePolicy(allowed_ops=frozenset({OpName.FETCH_TICKER}), policy_id="policy-a")
            ),
        )

    assert exc.value.error_code == ErrorCode.NOT_ALLOWED_OP


def test_private_operation_without_auth_returns_missing_auth() -> None:
    exchange = DummyExchange(supported_ops=frozenset({OpName.FETCH_BALANCE}))

    with pytest.raises(CoreError) as exc:
        exchange.execute(
            OpName.FETCH_BALANCE,
            params={},
            ctx=ExecuteContext(has_auth=False),
        )

    assert exc.value.error_code == ErrorCode.MISSING_AUTH
