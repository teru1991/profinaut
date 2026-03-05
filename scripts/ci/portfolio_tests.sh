#!/usr/bin/env bash
set -euo pipefail
pip install -r requirements-dev.txt
PYTHONPATH=services/portfolio pytest -q services/portfolio/tests
