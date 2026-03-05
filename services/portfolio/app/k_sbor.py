from __future__ import annotations

from dataclasses import dataclass, field
from decimal import Decimal, InvalidOperation
from typing import Any, Dict, List, Tuple

from app.k_ledger_store import iter_payloads


def D(x: str) -> Decimal:
    try:
        return Decimal(x)
    except InvalidOperation as e:
        raise ValueError(f"invalid decimal: {x}") from e


@dataclass
class Lot:
    qty: Decimal
    cost_quote: Decimal


@dataclass
class Position:
    lots: List[Lot] = field(default_factory=list)


@dataclass
class PortfolioState:
    cash: Dict[Tuple[str, str], Decimal] = field(default_factory=dict)
    positions: Dict[Tuple[str, str], Position] = field(default_factory=dict)
    marks: Dict[str, Decimal] = field(default_factory=dict)
    fx: Dict[Tuple[str, str], Decimal] = field(default_factory=dict)
    lineage: Dict[str, List[str]] = field(default_factory=dict)


def _lin_add(state: PortfolioState, key: str, event_id: str) -> None:
    state.lineage.setdefault(key, []).append(event_id)


def apply_payload(state: PortfolioState, payload: dict[str, Any]) -> None:
    kind = payload["kind"]
    event_id = payload["event_id"]
    account = payload["account"]
    asset = payload["asset"]
    qty = D(payload["qty"])

    if kind == "DEPOSIT":
        k = (account, asset)
        state.cash[k] = state.cash.get(k, Decimal("0")) + qty
        _lin_add(state, f"cash:{account}:{asset}", event_id)
        return
    if kind == "WITHDRAW":
        k = (account, asset)
        state.cash[k] = state.cash.get(k, Decimal("0")) - qty
        _lin_add(state, f"cash:{account}:{asset}", event_id)
        return
    if kind == "TRANSFER":
        k = (account, asset)
        state.cash[k] = state.cash.get(k, Decimal("0")) + qty
        _lin_add(state, f"cash:{account}:{asset}", event_id)
        return
    if kind == "PRICE_MARK":
        price = D(payload["price"])
        state.marks[asset] = price
        _lin_add(state, f"mark:{asset}", event_id)
        return
    if kind == "FX_RATE":
        fr = payload["asset"]
        to = payload["quote"]
        rate = D(payload["price"])
        state.fx[(fr, to)] = rate
        _lin_add(state, f"fx:{fr}:{to}", event_id)
        return
    if kind == "TRADE_FILL":
        price = D(payload["price"])
        quote = payload["quote"]
        cash_k = (account, quote)
        state.cash[cash_k] = state.cash.get(cash_k, Decimal("0")) - (qty * price)
        _lin_add(state, f"cash:{account}:{quote}", event_id)

        pos_k = (account, asset)
        pos = state.positions.setdefault(pos_k, Position())
        if qty > 0:
            pos.lots.append(Lot(qty=qty, cost_quote=qty * price))
        else:
            sell_qty = -qty
            remaining = sell_qty
            while remaining > 0 and pos.lots:
                lot = pos.lots[0]
                take = min(lot.qty, remaining)
                old_qty = lot.qty
                lot.qty -= take
                lot.cost_quote -= lot.cost_quote * (take / old_qty)
                remaining -= take
                if lot.qty == 0:
                    pos.lots.pop(0)
        _lin_add(state, f"pos:{account}:{asset}", event_id)
        return
    if kind in ("FEE", "FUNDING", "INTEREST", "ADJUSTMENT"):
        tgt = payload.get("fee_asset") or asset
        k = (account, tgt)
        state.cash[k] = state.cash.get(k, Decimal("0")) + qty
        _lin_add(state, f"cash:{account}:{tgt}", event_id)
        return

    raise ValueError(f"unknown kind: {kind}")


def replay(conn) -> PortfolioState:
    state = PortfolioState()
    for payload in iter_payloads(conn):
        apply_payload(state, payload)
    return state
