$ErrorActionPreference = "Stop"

Write-Host "[test] Validating Docker Compose configuration..." -ForegroundColor Cyan
docker compose config -q
Write-Host "[test] Compose configuration is valid." -ForegroundColor Green
