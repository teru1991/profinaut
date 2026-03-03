#!/usr/bin/env bash
set -euo pipefail

KILL_FILE="${1:-/tmp/profinaut_kill_switch.json}"

cat > "${KILL_FILE}" <<'JSON'
{
  "reason": "demo_local_kill",
  "trace_id": "demo-trace-id",
  "audit_id": "demo-audit-id"
}
JSON

echo "wrote ${KILL_FILE}"
