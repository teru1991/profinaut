from __future__ import annotations

import logging

import pytest

from services.marketdata.app.gmo_private_execution import (
    DictSecretRefResolver,
    FailoverPolicy,
    GmoPrivateExecutionAdapter,
    KeyPool,
    KeyRef,
    ResolvedSecret,
)
from services.marketdata.app.ucel_core import CoreError, ErrorCode, ExecuteContext, OpName, RuntimePolicy


@pytest.fixture
def key_pool() -> KeyPool:
    tick = {"now": 1_000}

    def _now() -> int:
        return tick["now"]

    pool = KeyPool(
        keys=[
            KeyRef(key_id="key-a", secret_ref="ref-a", scope="trade"),
            KeyRef(key_id="key-b", secret_ref="ref-b", scope="trade"),
        ],
        failover_policy=FailoverPolicy(max_attempts=1, cooldown_ms=100, respect_retry_after=True),
        now_ms_fn=_now,
    )
    pool._tick = tick  # type: ignore[attr-defined]
    return pool


def test_dry_run_default_blocks_transport_calls(key_pool: KeyPool) -> None:
    calls: list[str] = []
    adapter = GmoPrivateExecutionAdapter(
        key_pool=key_pool,
        secret_resolver=DictSecretRefResolver({"ref-a": ResolvedSecret("k", "s"), "ref-b": ResolvedSecret("k2", "s2")}),
        request_fn=lambda endpoint, _params, _api_key: calls.append(endpoint),
    )

    with pytest.raises(CoreError) as exc:
        adapter.execute(
            OpName.PLACE_ORDER,
            {"symbol": "BTC_JPY", "size": "0.01"},
            ExecuteContext(secret_ref="ref-a", features=frozenset({"execution"}), live_trading=False),
        )

    assert exc.value.error_code == ErrorCode.DRY_RUN_ONLY
    assert calls == []


def test_live_trading_requires_allowed_ops_and_auth(key_pool: KeyPool) -> None:
    calls: list[str] = []
    adapter = GmoPrivateExecutionAdapter(
        key_pool=key_pool,
        secret_resolver=DictSecretRefResolver({"ref-a": ResolvedSecret("k", "s"), "ref-b": ResolvedSecret("k2", "s2")}),
        request_fn=lambda endpoint, _params, _api_key: calls.append(endpoint) or {"status": 0},
    )

    with pytest.raises(CoreError) as exc:
        adapter.execute(
            OpName.PLACE_ORDER,
            {"symbol": "BTC_JPY"},
            ExecuteContext(
                secret_ref="ref-a",
                features=frozenset({"execution"}),
                live_trading=True,
                policy=RuntimePolicy(allowed_ops=frozenset({OpName.FETCH_ORDERS}), policy_id="p"),
            ),
        )
    assert exc.value.error_code == ErrorCode.NOT_ALLOWED_OP

    with pytest.raises(CoreError) as exc2:
        adapter.execute(
            OpName.PLACE_ORDER,
            {"symbol": "BTC_JPY"},
            ExecuteContext(features=frozenset({"execution"}), live_trading=True),
        )
    assert exc2.value.error_code == ErrorCode.MISSING_AUTH

    result = adapter.execute(
        OpName.PLACE_ORDER,
        {"symbol": "BTC_JPY"},
        ExecuteContext(
            secret_ref="ref-a",
            features=frozenset({"execution", "live-trading"}),
            live_trading=True,
            policy=RuntimePolicy(allowed_ops=frozenset({OpName.PLACE_ORDER}), policy_id="p"),
        ),
    )
    assert result["status"] == 0
    assert calls == ["crypto.private.rest.order.post"]


def test_failover_auth_failed_then_next_key_success() -> None:
    tick = {"now": 1_000}

    def _now() -> int:
        return tick["now"]

    pool = KeyPool(
        keys=[
            KeyRef(key_id="key-a", secret_ref="missing", scope="trade"),
            KeyRef(key_id="key-b", secret_ref="ref-b", scope="trade"),
        ],
        failover_policy=FailoverPolicy(max_attempts=1, cooldown_ms=100, respect_retry_after=True),
        now_ms_fn=_now,
    )
    adapter = GmoPrivateExecutionAdapter(
        key_pool=pool,
        secret_resolver=DictSecretRefResolver({"ref-b": ResolvedSecret("k2", "s2")}),
        request_fn=lambda endpoint, _params, _api_key: {"endpoint": endpoint},
    )

    with pytest.raises(CoreError):
        adapter.execute(OpName.FETCH_BALANCES, {}, ExecuteContext(secret_ref="any", features=frozenset({"execution"})))

    pool.mark_failure(key_id="key-a", error_code=ErrorCode.AUTH_FAILED)
    tick["now"] += 150
    result = adapter.execute(OpName.FETCH_BALANCES, {}, ExecuteContext(secret_ref="ref-b", features=frozenset({"execution"})))
    assert result["endpoint"] == "crypto.private.rest.assets.get"


def test_retry_after_respected_and_no_storm(key_pool: KeyPool) -> None:
    key_pool.mark_failure(key_id="key-a", error_code=ErrorCode.RATE_LIMITED, retry_after_ms=500)
    with pytest.raises(CoreError) as exc:
        key_pool.select(required_scope="withdraw")
    assert exc.value.error_code == ErrorCode.AUTH_FAILED

    selected = key_pool.select(required_scope="trade")
    assert selected.key_id == "key-b"


def test_secret_not_logged(caplog: pytest.LogCaptureFixture, key_pool: KeyPool) -> None:
    caplog.set_level(logging.INFO)
    adapter = GmoPrivateExecutionAdapter(
        key_pool=key_pool,
        secret_resolver=DictSecretRefResolver({"ref-a": ResolvedSecret("api-k", "super-secret")}),
        request_fn=lambda endpoint, _params, _api_key: {"endpoint": endpoint, "ok": True},
    )

    adapter.execute(
        OpName.FETCH_ORDERS,
        {"symbol": "BTC_JPY"},
        ExecuteContext(secret_ref="ref-a", features=frozenset({"execution"}), live_trading=True),
    )

    joined = "\n".join(rec.message for rec in caplog.records)
    assert "super-secret" not in joined
    assert "api-k" not in joined
    assert "key-a" in joined
