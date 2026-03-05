from __future__ import annotations

from dataclasses import dataclass
from decimal import Decimal
from typing import Any, Dict, List, Tuple

from app.k_sbor import PortfolioState


@dataclass(frozen=True)
class Confidence:
    score: float
    reasons: List[str]


@dataclass(frozen=True)
class ValuationReport:
    total_value_by_account_quote: Dict[Tuple[str, str], Decimal]
    confidence: Confidence
    evidence: Dict[str, Any]


def valuate(state: PortfolioState, default_quote: str = "JPY") -> ValuationReport:
    totals: Dict[Tuple[str, str], Decimal] = {}
    reasons: List[str] = []
    evidence: Dict[str, Any] = {"missing_marks": [], "missing_fx": []}

    for (account, currency), amt in state.cash.items():
        key = (account, currency)
        totals[key] = totals.get(key, Decimal("0")) + amt

    for (account, asset), pos in state.positions.items():
        mark = state.marks.get(asset)
        if mark is None:
            reasons.append(f"missing_mark:{asset}")
            evidence["missing_marks"].append(asset)
            continue
        qty = sum((lot.qty for lot in pos.lots), Decimal("0"))
        totals[(account, default_quote)] = totals.get((account, default_quote), Decimal("0")) + (qty * mark)

    score = 1.0
    if reasons:
        score = max(0.0, 1.0 - 0.1 * len(set(reasons)))
    return ValuationReport(total_value_by_account_quote=totals, confidence=Confidence(score=score, reasons=sorted(set(reasons))), evidence=evidence)
