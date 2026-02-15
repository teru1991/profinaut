"""Minimal deterministic backtest runner pinned to a dataset_ref."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any


def _require_dataset_ref(dataset_ref: str) -> str:
    dataset_ref = dataset_ref.strip()
    if not dataset_ref:
        raise ValueError("dataset_ref must be a non-empty string")
    return dataset_ref


def load_dataset(dataset_ref: str, points: int = 24) -> list[dict[str, Any]]:
    """Return a deterministic pseudo dataset derived from dataset_ref.

    This keeps the minimal runner self-contained while pinning all behavior to
    an explicit dataset_ref.
    """
    pinned_ref = _require_dataset_ref(dataset_ref)
    digest = hashlib.sha256(pinned_ref.encode("utf-8")).digest()

    data: list[dict[str, Any]] = []
    price = 100.0 + (digest[0] % 7)
    for index in range(points):
        delta_raw = digest[index % len(digest)]
        delta = ((delta_raw % 11) - 5) / 10.0
        price = round(max(1.0, price + delta), 6)
        data.append({"t": index, "close": price})
    return data


def run_backtest(dataset_ref: str, dataset: list[dict[str, Any]] | None = None) -> dict[str, Any]:
    """Run a forward-only replay (no future access) and return deterministic results."""
    pinned_ref = _require_dataset_ref(dataset_ref)
    prices = dataset if dataset is not None else load_dataset(pinned_ref)

    if not prices:
        raise ValueError("dataset must contain at least one row")

    cash = 10_000.0
    position = 0.0
    trades: list[dict[str, Any]] = []

    for idx in range(1, len(prices)):
        history = prices[:idx]  # forward-only slice; excludes current/future rows
        current = prices[idx]

        prev_price = float(history[-1]["close"])
        mean_price = sum(float(row["close"]) for row in history) / len(history)
        current_price = float(current["close"])

        action = "hold"
        if prev_price <= mean_price and current_price > prev_price and cash >= current_price:
            position = round(position + 1.0, 6)
            cash = round(cash - current_price, 6)
            action = "buy"
        elif prev_price >= mean_price and current_price < prev_price and position >= 1.0:
            position = round(position - 1.0, 6)
            cash = round(cash + current_price, 6)
            action = "sell"

        equity = round(cash + (position * current_price), 6)
        trades.append(
            {
                "t": int(current["t"]),
                "action": action,
                "price": round(current_price, 6),
                "position": position,
                "cash": cash,
                "equity": equity,
            }
        )

    final_price = float(prices[-1]["close"])
    final_equity = round(cash + (position * final_price), 6)
    result = {
        "dataset_ref": pinned_ref,
        "rows": [{"t": int(row["t"]), "close": round(float(row["close"]), 6)} for row in prices],
        "trades": trades,
    }
    summary = {
        "dataset_ref": pinned_ref,
        "total_steps": len(prices) - 1,
        "total_trades": sum(1 for trade in trades if trade["action"] != "hold"),
        "final_equity": final_equity,
        "return_pct": round(((final_equity / 10_000.0) - 1.0) * 100.0, 6),
    }
    return {"result": result, "summary": summary}


def write_artifacts(dataset_ref: str, out_dir: Path, dataset: list[dict[str, Any]] | None = None) -> dict[str, Path]:
    payload = run_backtest(dataset_ref=dataset_ref, dataset=dataset)

    out_dir.mkdir(parents=True, exist_ok=True)
    result_path = out_dir / "result.json"
    summary_path = out_dir / "summary.json"

    result_path.write_text(
        json.dumps(payload["result"], indent=2, sort_keys=True, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    summary_path.write_text(
        json.dumps(payload["summary"], indent=2, sort_keys=True, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return {"result": result_path, "summary": summary_path}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run minimal deterministic backtest")
    parser.add_argument("--dataset-ref", required=True, help="Pinned dataset identifier")
    parser.add_argument("--out", required=True, help="Artifact output directory")
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    dataset_ref = _require_dataset_ref(args.dataset_ref)
    write_artifacts(dataset_ref=dataset_ref, out_dir=Path(args.out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
