from __future__ import annotations

from collections import defaultdict, deque
from dataclasses import dataclass
from decimal import Decimal
from typing import Dict, List, Tuple

from app.k_ledger_store import iter_payloads
from app.k_sbor import D, PortfolioState


@dataclass(frozen=True)
class PnLComponent:
    component: str
    amount: Decimal
    refs: List[str]


@dataclass(frozen=True)
class PnLReport:
    realized: Dict[Tuple[str, str], Decimal]
    unrealized: Dict[Tuple[str, str], Decimal]
    components: List[PnLComponent]


def compute_pnl(conn, state: PortfolioState, default_quote: str = "JPY") -> PnLReport:
    realized: Dict[Tuple[str, str], Decimal] = {}
    unrealized: Dict[Tuple[str, str], Decimal] = {}
    comps: List[PnLComponent] = []

    lot_book = defaultdict(lambda: deque())
    for payload in iter_payloads(conn):
        if payload["kind"] != "TRADE_FILL":
            continue
        event_id = payload["event_id"]
        account = payload["account"]
        asset = payload["asset"]
        qty = D(payload["qty"])
        price = D(payload["price"])
        quote = payload["quote"]

        key = (account, asset)
        if qty > 0:
            lot_book[key].append([qty, price, [event_id]])
            continue

        sell_qty = -qty
        remaining = sell_qty
        sell_value = sell_qty * price
        cost_total = Decimal("0")
        refs: List[str] = [event_id]

        while remaining > 0:
            if not lot_book[key]:
                comps.append(PnLComponent("pnl_missing_lot", Decimal("0"), refs))
                cost_total = sell_value
                remaining = Decimal("0")
                break
            lot_qty, unit_cost, lot_refs = lot_book[key][0]
            take = min(lot_qty, remaining)
            cost_total += take * unit_cost
            lot_qty -= take
            remaining -= take
            refs.extend(lot_refs)
            if lot_qty == 0:
                lot_book[key].popleft()
            else:
                lot_book[key][0][0] = lot_qty

        pnl = sell_value - cost_total
        realized[(account, quote)] = realized.get((account, quote), Decimal("0")) + pnl
        comps.append(PnLComponent("trade_realized", pnl, refs))

    for payload in iter_payloads(conn):
        if payload["kind"] in ("FEE", "FUNDING", "INTEREST"):
            event_id = payload["event_id"]
            account = payload["account"]
            amt = D(payload["qty"])
            asset = payload.get("fee_asset") or payload["asset"]
            comps.append(PnLComponent(payload["kind"].lower(), amt, [event_id]))
            unrealized[(account, asset)] = unrealized.get((account, asset), Decimal("0")) + amt

    for (account, asset), pos in state.positions.items():
        mark = state.marks.get(asset)
        if mark is None:
            continue
        qty_sum = sum((lot.qty for lot in pos.lots), Decimal("0"))
        cost = sum((lot.cost_quote for lot in pos.lots), Decimal("0"))
        mkt = qty_sum * mark
        upnl = mkt - cost
        unrealized[(account, default_quote)] = unrealized.get((account, default_quote), Decimal("0")) + upnl
        comps.append(PnLComponent("mark_unrealized", upnl, state.lineage.get(f"pos:{account}:{asset}", [])))

    return PnLReport(realized=realized, unrealized=unrealized, components=comps)
