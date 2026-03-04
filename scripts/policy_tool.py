from __future__ import annotations

import sys
from pathlib import Path

import argparse
import hashlib
import json

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from libs.safety_core.redaction import safe_str


def sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    h.update(p.read_bytes())
    return h.hexdigest()


def validate_json(p: Path) -> None:
    obj = json.loads(p.read_text(encoding="utf-8"))
    if not isinstance(obj, dict):
        raise RuntimeError("policy must be json object")
    if "version" not in obj:
        raise RuntimeError("policy must include version")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["hash", "validate"])
    ap.add_argument("path")
    ns = ap.parse_args()

    p = Path(ns.path)
    try:
        if ns.cmd == "validate":
            validate_json(p)
            print("OK")
            return 0
        if ns.cmd == "hash":
            validate_json(p)
            print(sha256_file(p))
            return 0
    except Exception as e:
        print("error:", safe_str(str(e)))
        return 2
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
