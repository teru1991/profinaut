from __future__ import annotations

import json
from pathlib import Path

import pytest

from worker.backtest import run_backtest, write_artifacts


def _read_bytes(path: Path) -> bytes:
    return path.read_bytes()


def test_artifacts_are_deterministic_for_same_dataset_ref(tmp_path: Path) -> None:
    dataset_ref = "dataset-alpha"
    first_out = tmp_path / "first" / dataset_ref
    second_out = tmp_path / "second" / dataset_ref

    first_paths = write_artifacts(dataset_ref=dataset_ref, out_dir=first_out)
    second_paths = write_artifacts(dataset_ref=dataset_ref, out_dir=second_out)

    assert _read_bytes(first_paths["result"]) == _read_bytes(second_paths["result"])
    assert _read_bytes(first_paths["summary"]) == _read_bytes(second_paths["summary"])


def test_forward_only_prevents_future_data_influence() -> None:
    dataset_ref = "dataset-lookahead"
    base_dataset = [
        {"t": 0, "close": 100.0},
        {"t": 1, "close": 101.0},
        {"t": 2, "close": 102.0},
        {"t": 3, "close": 103.0},
        {"t": 4, "close": 104.0},
        {"t": 5, "close": 105.0},
    ]
    mutated_dataset = [*base_dataset]
    mutated_dataset[5] = {"t": 5, "close": 999.0}

    base = run_backtest(dataset_ref=dataset_ref, dataset=base_dataset)
    mutated = run_backtest(dataset_ref=dataset_ref, dataset=mutated_dataset)

    # Decision/equity at t=4 (index 3 in trades) must be identical: t=5 mutation is future data.
    assert base["result"]["trades"][3] == mutated["result"]["trades"][3]


def test_dataset_ref_must_be_non_empty() -> None:
    with pytest.raises(ValueError):
        run_backtest(dataset_ref="   ")


def test_summary_schema_is_stable(tmp_path: Path) -> None:
    out_dir = tmp_path / "dataset-beta"
    paths = write_artifacts(dataset_ref="dataset-beta", out_dir=out_dir)

    summary = json.loads(paths["summary"].read_text(encoding="utf-8"))
    assert list(summary.keys()) == [
        "dataset_ref",
        "final_equity",
        "return_pct",
        "total_steps",
        "total_trades",
    ]
