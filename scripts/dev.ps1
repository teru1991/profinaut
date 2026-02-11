$ErrorActionPreference = "Stop"

Write-Host "[dev] Starting Profinaut stack (Postgres + API + Web)..." -ForegroundColor Cyan
docker compose up --build
