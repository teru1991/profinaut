$ErrorActionPreference = "Stop"

Write-Host "[dev] Starting Profinaut local stack via Docker Compose..." -ForegroundColor Cyan
docker compose up --build
