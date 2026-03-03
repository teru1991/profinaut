from __future__ import annotations

from datetime import UTC, datetime, timedelta

import pytest

from bots.simple_mm import main as simple_mm
from libs.safety_core.gate import SafetyGate, SafetyGateError
from libs.safety_core.lease import ExecutionLease
from libs.safety_core.models import SafetyMode


class _Provider:
    def __init__(self, mode: SafetyMode, lease: ExecutionLease | None, raise_mode: bool = False) -> None:
        self.mode = mode
        self.lease = lease
        self.raise_mode = raise_mode

    def current_mode(self) -> SafetyMode:
        if self.raise_mode:
            raise RuntimeError("down")
        return self.mode

    def current_lease(self, _subject_kind: str, _subject_id: str) -> ExecutionLease | None:
        return self.lease


def _lease_valid(seconds: int = 20) -> ExecutionLease:
    now = datetime.now(UTC)
    return ExecutionLease(
        subject_kind="BOT",
        subject_id="simple-mm",
        scope_kind="GLOBAL",
        selector={"venue": "bybit", "symbol": "BTCUSDT"},
        issued_by="tester",
        reason="test",
        issued_at=now.isoformat(),
        expires_at=(now + timedelta(seconds=seconds)).isoformat(),
    )


def test_gate_rejects_missing_lease() -> None:
    gate = SafetyGate(provider=_Provider(SafetyMode.NORMAL, None))
    with pytest.raises(SafetyGateError, match="missing"):
        gate.check_before_send("create_order", "BOT", "simple-mm", "bybit", "BTCUSDT", "BUY", 1, None, False, datetime.now(UTC))


def test_gate_rejects_expired_lease() -> None:
    gate = SafetyGate(provider=_Provider(SafetyMode.NORMAL, _lease_valid(seconds=-1)))
    with pytest.raises(SafetyGateError) as exc:
        gate.check_before_send("create_order", "BOT", "simple-mm", "bybit", "BTCUSDT", "BUY", 1, None, False, datetime.now(UTC))
    assert exc.value.reason_code == "LEASE_EXPIRED"


def test_gate_rejects_when_safety_unreachable() -> None:
    gate = SafetyGate(provider=_Provider(SafetyMode.NORMAL, _lease_valid(), raise_mode=True))
    with pytest.raises(SafetyGateError) as exc:
        gate.check_before_send("create_order", "BOT", "simple-mm", "bybit", "BTCUSDT", "BUY", 1, None, False, datetime.now(UTC))
    assert exc.value.reason_code == "SAFETY_UNREACHABLE"


def test_simple_mm_submit_does_not_send_without_lease(monkeypatch) -> None:
    calls: list[tuple[str, str]] = []

    def _http_json(method: str, url: str, **kwargs):
        calls.append((method, url))
        if url.startswith("http://127.0.0.1:8000/safety/lease/status"):
            return 200, {"lease": None}
        if url.endswith("/execution/order-intents"):
            pytest.fail("execution send path should not be called when lease is missing")
        return 200, {}

    monkeypatch.setattr(simple_mm, "http_json", _http_json)
    with pytest.raises(simple_mm.BotError, match="SAFETY_BLOCKED:LEASE_MISSING"):
        simple_mm.submit_order_intent(
            "http://127.0.0.1:8001",
            {
                "exchange": "bybit",
                "symbol": "BTCUSDT",
                "side": "BUY",
                "qty": 1,
            },
        )
    assert any("/safety/lease/status" in url for _, url in calls)
    assert not any(url.endswith("/execution/order-intents") for _, url in calls)


def test_gate_rejects_unknown_state_parse() -> None:
    class _BadModeProvider(_Provider):
        def current_mode(self):  # type: ignore[override]
            raise ValueError("bad mode")

    gate = SafetyGate(provider=_BadModeProvider(SafetyMode.NORMAL, _lease_valid()))
    with pytest.raises(SafetyGateError) as exc:
        gate.check_before_send("create_order", "BOT", "simple-mm", "bybit", "BTCUSDT", "BUY", 1, None, False, datetime.now(UTC))
    assert exc.value.reason_code == "SAFETY_UNREACHABLE"
