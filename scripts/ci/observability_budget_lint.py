from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]


def main() -> int:
    policy = Path("docs/policy/observability_budget.toml")
    if tomllib is None:
        return 0
    if not policy.exists():
        print("missing budget policy", file=sys.stderr)
        return 2

    data = tomllib.loads(policy.read_text(encoding="utf-8"))
    metrics = data.get("metrics", {}) if isinstance(data.get("metrics"), dict) else {}
    logs = data.get("logs", {}) if isinstance(data.get("logs"), dict) else {}

    if int(metrics.get("max_unique_series_per_metric", 0)) < 1:
        return 3
    if int(metrics.get("max_total_unique_series", 0)) < 1:
        return 4
    if int(logs.get("max_event_fields", 0)) < 1:
        return 5
    if int(logs.get("max_event_bytes", 0)) < 128:
        return 6

    env = dict(os.environ)
    env["PROFINAUT_OBS_BUDGET_STRICT"] = "1"
    cmd = [
        sys.executable,
        "-m",
        "pytest",
        "-q",
        "tests/test_observability_cardinality_guard.py",
        "tests/test_observability_budget_state_and_healthz.py",
        "tests/test_observability_metrics_drops_on_budget.py",
    ]
    return subprocess.run(cmd, env=env).returncode


if __name__ == "__main__":
    raise SystemExit(main())
