from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


def main() -> int:
    schema = Path("docs/contracts/observability/log_event.schema.json")
    if not schema.exists():
        print(f"missing schema: {schema}", file=sys.stderr)
        return 2

    try:
        json.loads(schema.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        print(f"schema parse failed: {exc}", file=sys.stderr)
        return 3

    env = dict(os.environ)
    env["PROFINAUT_OBS_LOG_STRICT"] = "1"
    cmd = [
        sys.executable,
        "-m",
        "pytest",
        "-q",
        "tests/test_observability_log_contract_schema.py",
        "tests/test_observability_logging_required_keys.py",
        "tests/test_observability_middleware_injects_headers_and_logs.py",
    ]
    return subprocess.run(cmd, env=env).returncode


if __name__ == "__main__":
    raise SystemExit(main())
