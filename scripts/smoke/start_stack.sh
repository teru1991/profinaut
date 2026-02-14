#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

if ! command -v docker >/dev/null 2>&1; then
  echo "[smoke][error] docker command not found. Install Docker Desktop/Engine first." >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "[smoke][error] docker daemon is not reachable. Start Docker and retry." >&2
  exit 1
fi

echo "[smoke] starting stack with docker compose up -d"
docker compose up -d

echo "[smoke] stack start command completed"
