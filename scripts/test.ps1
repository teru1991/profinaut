$ErrorActionPreference = "Stop"

Write-Host "[test] Running contracts lint..." -ForegroundColor Cyan
./scripts/validate_contracts.ps1

Write-Host "[test] Running dashboard-api tests..." -ForegroundColor Cyan
Push-Location services/dashboard-api
python -m pip install -r requirements.txt
python -m pip install -r requirements-dev.txt
python -m pytest -q
Pop-Location

Write-Host "[test] All checks passed." -ForegroundColor Green
