#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
export SCHEMAS_DIR="${REPO_ROOT}/docs/contracts"

cd "${SCRIPT_DIR}"
npm ci

node validate.js
