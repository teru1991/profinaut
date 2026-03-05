from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from typing import Any, Literal, Optional

Kind = Literal[
    "DEPOSIT",
    "WITHDRAW",
    "TRANSFER",
    "TRADE_FILL",
    "FEE",
    "FUNDING",
    "INTEREST",
    "PRICE_MARK",
    "FX_RATE",
    "ADJUSTMENT",
]


@dataclass(frozen=True)
class LedgerRefs:
    tx_ref: Optional[str] = None
    venue: Optional[str] = None
    symbol: Optional[str] = None


@dataclass(frozen=True)
class LedgerEvent:
    schema_version: int
    event_id: str
    ts_utc: datetime
    source: str
    account: str
    kind: Kind
    asset: str
    qty: str
    price: Optional[str] = None
    quote: Optional[str] = None
    fee: Optional[str] = None
    fee_asset: Optional[str] = None
    refs: LedgerRefs = LedgerRefs()


def event_to_payload(e: LedgerEvent) -> dict[str, Any]:
    return {
        "schema_version": e.schema_version,
        "event_id": e.event_id,
        "ts_utc": e.ts_utc.isoformat(),
        "source": e.source,
        "account": e.account,
        "kind": e.kind,
        "asset": e.asset,
        "qty": e.qty,
        "price": e.price,
        "quote": e.quote,
        "fee": e.fee,
        "fee_asset": e.fee_asset,
        "refs": {
            "tx_ref": e.refs.tx_ref,
            "venue": e.refs.venue,
            "symbol": e.refs.symbol,
        },
    }
