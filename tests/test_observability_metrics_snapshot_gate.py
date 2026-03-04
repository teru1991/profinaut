import subprocess
import sys


def test_metrics_snapshot_gate():
    process = subprocess.run([sys.executable, "scripts/ci/observability_metrics_snapshot.py"])
    assert process.returncode == 0
