from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List

from app.k_pnl import PnLReport, compute_pnl
from app.k_sbor import PortfolioState
from app.k_valuation import ValuationReport, valuate


@dataclass(frozen=True)
class ExplainReport:
    valuation: ValuationReport
    pnl: PnLReport
    lineage: Dict[str, List[str]]
    confidence_score: float
    confidence_reasons: List[str]
    evidence: Dict[str, Any]


def explain(conn, state: PortfolioState) -> ExplainReport:
    val = valuate(state)
    pnl = compute_pnl(conn, state)
    return ExplainReport(
        valuation=val,
        pnl=pnl,
        lineage=state.lineage,
        confidence_score=val.confidence.score,
        confidence_reasons=val.confidence.reasons,
        evidence={"valuation": val.evidence},
    )
