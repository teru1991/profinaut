from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.append(str(REPO_ROOT))
from pathlib import Path

from libs.observability.metrics_catalog import METRICS, METRICS_CATALOG_VERSION
from libs.observability.metrics_guard import validate_catalog

SNAPSHOT_PATH = Path("docs/contracts/observability/metrics_catalog.snapshot.txt")


def render_snapshot() -> str:
    validate_catalog(METRICS)
    lines = [f"version={METRICS_CATALOG_VERSION}"]
    for metric in METRICS:
        labels = ",".join(metric.get("labels", []))
        help_text = str(metric.get("help", "")).replace("\n", " ").strip()
        lines.append(
            f"{metric['name']}|{metric['type']}|{metric.get('unit','')}|{labels}|{help_text}"
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--accept", action="store_true", help="overwrite snapshot (local only)")
    args = parser.parse_args()

    if args.accept and os.getenv("CI") == "true":
        print("--accept is forbidden in CI", file=sys.stderr)
        return 4

    snapshot = render_snapshot()
    if args.accept:
        SNAPSHOT_PATH.parent.mkdir(parents=True, exist_ok=True)
        SNAPSHOT_PATH.write_text(snapshot, encoding="utf-8")
        print(f"snapshot updated: {SNAPSHOT_PATH}")
        return 0

    if not SNAPSHOT_PATH.exists():
        print("snapshot missing. Run with --accept locally to create it.", file=sys.stderr)
        return 2

    current = SNAPSHOT_PATH.read_text(encoding="utf-8")
    if current != snapshot:
        print("metrics catalog snapshot mismatch!", file=sys.stderr)
        print(
            "Run locally: python scripts/ci/observability_metrics_snapshot.py --accept",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
