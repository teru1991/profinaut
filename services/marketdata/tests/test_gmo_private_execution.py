from __future__ import annotations

import logging

import pytest

from services.marketdata.app.gmo_private_execution import FailoverPolicy, GmoPrivateExecutionAdapter, KeyPool, KeyRef
from services.marketdata.app.ucel_core import (
    CoreError,
    DictSecretRefResolver,
    ErrorCode,
    ExecuteContext,
    OpName,
    OrderIntent,
    ResolvedSecret,
    RuntimePolicy,
)


@pytest.fixture
def key_pool() -> KeyPool:
    tick = {"now": 1_000}

    def _now() -> int:
        return tick["now"]

    pool = KeyPool(
        keys=[
            KeyRef(key_id="key-read", secret_ref="ref-read", scope="read_only"),
            KeyRef(key_id="key-trade", secret_ref="ref-trade", scope="trade"),
            KeyRef(key_id="key-trade-2", secret_ref="ref-trade-2", scope="trade"),
        ],
        failover_policy=FailoverPolicy(max_attempts=2, cooldown_ms=100, respect_retry_after=True),
        now_ms_fn=_now,
    )
    pool._tick = tick  # type: ignore[attr-defined]
    return pool


def test_preflight_guards_block_transport_calls(key_pool: KeyPool) -> None:
    calls: list[str] = []
    adapter = GmoPrivateExecutionAdapter(
        key_pool=key_pool,
        secret_resolver=DictSecretRefResolver(
            {
                "ref-read": ResolvedSecret("rk", "rs"),
                "ref-trade": ResolvedSecret("tk", "ts"),
                "ref-trade-2": ResolvedSecret("tk2", "ts2"),
            }
        ),
        request_fn=lambda endpoint, _params, _api_key: calls.append(endpoint),
    )

    with pytest.raises(CoreError) as blocked_op:
        adapter.execute(
            OpName.PLACE_ORDER,
            {"symbol": "BTC_JPY"},
            ExecuteContext(
                secret_ref="ref-trade",
                features=frozenset({"execution", "live-trading"}),
                live_trading=True,
                policy=RuntimePolicy(allowed_ops=frozenset({OpName.FETCH_ORDERS}), policy_id="p"),
            ),
        )
    assert blocked_op.value.error_code == ErrorCode.NOT_ALLOWED_OP

    with pytest.raises(CoreError) as dry_run_only:
        adapter.execute(
            OpName.PLACE_ORDER,
            {"symbol": "BTC_JPY"},
            ExecuteContext(secret_ref="ref-trade", features=frozenset({"execution"}), live_trading=False),
        )
    assert dry_run_only.value.error_code == ErrorCode.DRY_RUN_ONLY

    with pytest.raises(CoreError) as missing_auth:
        adapter.execute(
            OpName.PLACE_ORDER,
            {"symbol": "BTC_JPY"},
            ExecuteContext(features=frozenset({"execution", "live-trading"}), live_trading=True),
        )
    assert missing_auth.value.error_code == ErrorCode.MISSING_AUTH
    assert calls == []


def test_live_trading_allowed_path_calls_transport(key_pool: KeyPool) -> None:
    calls: list[str] = []
    adapter = GmoPrivateExecutionAdapter(
        key_pool=key_pool,
        secret_resolver=DictSecretRefResolver(
            {
                "ref-read": ResolvedSecret("rk", "rs"),
                "ref-trade": ResolvedSecret("tk", "ts"),
                "ref-trade-2": ResolvedSecret("tk2", "ts2"),
            }
        ),
        request_fn=lambda endpoint, _params, _api_key: calls.append(endpoint) or {"status": 0, "data": {"orderId": "o-1"}},
    )

    result = adapter.place_order(
        intent=OrderIntent(symbol="BTC_JPY", side="BUY", order_type="LIMIT", size=0.01, price=1_000_000),
        ctx=ExecuteContext(
            secret_ref="ref-trade",
            features=frozenset({"execution", "live-trading"}),
            live_trading=True,
            policy=RuntimePolicy(allowed_ops=frozenset({OpName.PLACE_ORDER}), policy_id="p"),
        ),
    )

    assert result.status == "accepted"
    assert calls == ["crypto.private.rest.order.post"]


def test_read_only_scope_for_balance_does_not_require_live_mode(key_pool: KeyPool) -> None:
    calls: list[str] = []
    adapter = GmoPrivateExecutionAdapter(
        key_pool=key_pool,
        secret_resolver=DictSecretRefResolver({"ref-read": ResolvedSecret("rk", "rs")}),
        request_fn=lambda endpoint, _params, _api_key: calls.append(endpoint) or {"status": 0, "data": [{"symbol": "JPY", "amount": "1"}]},
    )

    balances = adapter.fetch_balances(ctx=ExecuteContext(secret_ref="ref-read", features=frozenset({"execution"}), live_trading=False))

    assert balances[0].symbol == "JPY"
    assert calls == ["crypto.private.rest.assets.get"]


def test_failover_auth_failed_then_next_key_success() -> None:
    tick = {"now": 1_000}

    def _now() -> int:
        return tick["now"]

    pool = KeyPool(
        keys=[
            KeyRef(key_id="key-a", secret_ref="missing", scope="trade"),
            KeyRef(key_id="key-b", secret_ref="ref-b", scope="trade"),
        ],
        failover_policy=FailoverPolicy(max_attempts=2, cooldown_ms=100, respect_retry_after=True),
        now_ms_fn=_now,
    )
    adapter = GmoPrivateExecutionAdapter(
        key_pool=pool,
        secret_resolver=DictSecretRefResolver({"ref-b": ResolvedSecret("k2", "s2")}),
        request_fn=lambda endpoint, _params, _api_key: {"status": 0, "data": {"orderId": "x"}, "endpoint": endpoint},
    )

    result = adapter.execute(
        OpName.PLACE_ORDER,
        {"symbol": "BTC_JPY"},
        ExecuteContext(secret_ref="any", features=frozenset({"execution", "live-trading"}), live_trading=True),
    )

    assert result.order_id == "x"
    assert pool.health["key-a"].exhausted is False
    assert pool.health["key-a"].consecutive_failures == 1


def test_retry_after_respected_and_no_storm(key_pool: KeyPool) -> None:
    key_pool.mark_failure(key_id="key-trade", error_code=ErrorCode.RATE_LIMITED, retry_after_ms=500)
    selected = key_pool.select(required_scope="trade")
    assert selected.key_id == "key-trade-2"


def test_all_keys_exhausted_returns_clear_error() -> None:
    pool = KeyPool(
        keys=[KeyRef(key_id="key-a", secret_ref="ra", scope="trade")],
        failover_policy=FailoverPolicy(max_attempts=1, cooldown_ms=100, respect_retry_after=True, ban_risk_guard=True),
    )
    pool.mark_failure(key_id="key-a", error_code=ErrorCode.AUTH_FAILED)

    with pytest.raises(CoreError) as exc:
        pool.select(required_scope="trade")

    assert exc.value.error_code == ErrorCode.PERMISSION_DENIED


def test_secret_not_logged_or_exposed_in_exception(caplog: pytest.LogCaptureFixture, key_pool: KeyPool) -> None:
    caplog.set_level(logging.INFO)
    secret = "SENSITIVE_TEST_TOKEN_123"
    adapter = GmoPrivateExecutionAdapter(
        key_pool=key_pool,
        secret_resolver=DictSecretRefResolver(
            {
                "ref-read": ResolvedSecret("rk", "rs"),
                "ref-trade": ResolvedSecret("api-k", secret),
                "ref-trade-2": ResolvedSecret("api-k2", "other"),
            }
        ),
        request_fn=lambda _endpoint, _params, _api_key: (_ for _ in ()).throw(RuntimeError(f"boom {secret}")),
    )

    with pytest.raises(CoreError) as exc:
        adapter.execute(
            OpName.PLACE_ORDER,
            {"symbol": "BTC_JPY"},
            ExecuteContext(secret_ref="ref-trade", features=frozenset({"execution", "live-trading"}), live_trading=True),
        )

    joined = "\n".join(rec.message for rec in caplog.records)
    assert secret not in joined
    assert "api-k" not in joined
    assert secret not in str(exc.value)
    assert secret not in repr(ResolvedSecret("x", secret))
    assert exc.value.error_code == ErrorCode.AUTH_FAILED
