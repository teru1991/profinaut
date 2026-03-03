from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
import math
from typing import Protocol

from libs.safety_core.lease import ExecutionLease, LeaseInvalidError
from libs.safety_core.models import SafetyMode


class SafetyGateError(RuntimeError):
    def __init__(self, reason_code: str, message: str) -> None:
        super().__init__(message)
        self.reason_code = reason_code


class GateStateProvider(Protocol):
    def current_mode(self) -> SafetyMode: ...

    def current_lease(self, subject_kind: str, subject_id: str) -> ExecutionLease | None: ...


@dataclass(slots=True)
class SafetyGate:
    provider: GateStateProvider

    def check_before_send(
        self,
        op: str,
        subject_kind: str,
        subject_id: str,
        venue: str,
        symbol: str,
        side: str,
        qty: float,
        price: float | None,
        is_reduce_only: bool,
        now: datetime,
    ) -> None:
        try:
            mode = self.provider.current_mode()
        except Exception as exc:
            raise SafetyGateError("SAFETY_UNREACHABLE", "unable to resolve safety mode") from exc

        if mode == SafetyMode.EMERGENCY_STOP:
            raise SafetyGateError("EMERGENCY_STOP_ACTIVE", "send blocked by emergency stop")

        lease = self.provider.current_lease(subject_kind, subject_id)
        if lease is None:
            raise SafetyGateError("LEASE_MISSING", "execution lease missing")
        try:
            lease.assert_valid_or_raise(now)
        except LeaseInvalidError as exc:
            raise SafetyGateError(exc.reason_code, str(exc)) from exc

        if mode == SafetyMode.SAFE and not is_reduce_only:
            raise SafetyGateError("SAFE_CLOSE_ONLY", "safe mode only allows reduce-only operations")

        if not math.isfinite(qty) or qty <= 0:
            raise SafetyGateError("INVALID_QTY", "qty must be finite and > 0")
        if price is not None and (not math.isfinite(price) or price <= 0):
            raise SafetyGateError("INVALID_PRICE", "price must be finite and > 0 when present")
        if not venue or not symbol or not side or not op:
            raise SafetyGateError("INVALID_INPUT", "required order context is missing")
