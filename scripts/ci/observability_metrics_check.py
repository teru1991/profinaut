from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.append(str(REPO_ROOT))


def main() -> int:
    snapshot = subprocess.run([sys.executable, "scripts/ci/observability_metrics_snapshot.py"])
    if snapshot.returncode != 0:
        return snapshot.returncode

    env = dict(os.environ)
    cmd = [
        sys.executable,
        "-m",
        "pytest",
        "-q",
        "tests/test_observability_metrics_required.py",
        "tests/test_observability_metrics_guard_labels.py",
        "tests/test_observability_metrics_snapshot_gate.py",
    ]
    return subprocess.run(cmd, env=env).returncode


if __name__ == "__main__":
    raise SystemExit(main())
