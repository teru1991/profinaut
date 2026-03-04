from __future__ import annotations

import sys
from pathlib import Path

import argparse

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from libs.safety_core.access_review import generate_access_review


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="docs/reports/access_review_latest.md")
    ns = ap.parse_args()
    generate_access_review(out_path=Path(ns.out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
