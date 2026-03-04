from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


SCHEMA_DIR = Path("docs/contracts/observability")


def main() -> int:
    for schema_path in sorted(SCHEMA_DIR.glob("*.schema.json")):
        json.loads(schema_path.read_text(encoding="utf-8"))

    test_files = sorted(str(path) for path in Path("tests").glob("test_observability_contract_*.py"))
    cmd = [sys.executable, "-m", "pytest", "-q", *test_files]
    return subprocess.call(cmd)


if __name__ == "__main__":
    raise SystemExit(main())
