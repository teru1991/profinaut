$ErrorActionPreference = "Stop"

Write-Host "[test] Validating Docker Compose configuration..." -ForegroundColor Cyan
docker compose config -q

Write-Host "[test] Running contract checks (OpenAPI + JSON Schema)..." -ForegroundColor Cyan
./scripts/validate_contracts.ps1

Write-Host "[test] All baseline checks passed." -ForegroundColor Green
