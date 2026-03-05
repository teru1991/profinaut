#!/usr/bin/env bash
set -euo pipefail

echo "[execution-tests] installing deps"
pip install -r requirements-dev.txt

echo "[execution-tests] pytest services/execution"
PYTHONPATH=services/execution pytest -q services/execution/tests
