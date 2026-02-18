from __future__ import annotations

from typing import Any, Mapping

from services.marketdata.app.gmo_adapter import (
    DEFAULT_GMO_CAPABILITIES,
    AdapterPolicyError,
    GmoAdapterFacade,
    GmoAdapterPolicy,
    OpName,
    requires_auth,
)


class _TransportSpy:
    def __init__(self) -> None:
        self.calls: list[tuple[str, Mapping[str, Any] | None]] = []

    def __call__(self, op: str, payload: Mapping[str, Any] | None) -> dict[str, object]:
        self.calls.append((op, payload))
        return {"ok": True, "op": op}


def test_allowed_ops_rejects_and_does_not_call_upstream() -> None:
    spy = _TransportSpy()
    adapter = GmoAdapterFacade(
        transport=spy,
        policy=GmoAdapterPolicy(allowed_ops=frozenset({OpName.FETCH_TICKER}), policy_id="policy-test"),
        capabilities=DEFAULT_GMO_CAPABILITIES,
    )

    try:
        adapter.execute(OpName.FETCH_TRADES)
        assert False, "expected AdapterPolicyError"
    except AdapterPolicyError as exc:
        assert exc.code == "NOT_ALLOWED_OP"

    assert spy.calls == []


def test_private_op_without_auth_is_preflight_rejected_and_no_upstream_call() -> None:
    spy = _TransportSpy()
    private_caps = DEFAULT_GMO_CAPABILITIES.supported_ops | {OpName.FETCH_BALANCES}
    adapter = GmoAdapterFacade(
        transport=spy,
        policy=GmoAdapterPolicy(allowed_ops=frozenset({OpName.FETCH_BALANCES})),
        capabilities=DEFAULT_GMO_CAPABILITIES.__class__(supported_ops=frozenset(private_caps)),
        auth_resolver=lambda: None,
    )

    try:
        adapter.execute(OpName.FETCH_BALANCES)
        assert False, "expected AdapterPolicyError"
    except AdapterPolicyError as exc:
        assert exc.code == "MISSING_AUTH"
        assert exc.details.get("preflight_reject") is True

    assert spy.calls == []


def test_capabilities_missing_op_returns_not_supported() -> None:
    spy = _TransportSpy()
    adapter = GmoAdapterFacade(
        transport=spy,
        policy=GmoAdapterPolicy(allowed_ops=frozenset({OpName.FETCH_BALANCES})),
        capabilities=DEFAULT_GMO_CAPABILITIES,
    )

    try:
        adapter.execute(OpName.FETCH_BALANCES)
        assert False, "expected AdapterPolicyError"
    except AdapterPolicyError as exc:
        assert exc.code == "NOT_SUPPORTED"

    assert spy.calls == []


def test_public_ops_do_not_require_auth() -> None:
    assert requires_auth(OpName.FETCH_TICKER) is False
    assert requires_auth(OpName.SUBSCRIBE_ORDERBOOK) is False


def test_private_ops_require_auth() -> None:
    assert requires_auth(OpName.FETCH_BALANCES) is True
    assert requires_auth(OpName.CANCEL_ORDER) is True
