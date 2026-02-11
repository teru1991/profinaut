$ErrorActionPreference = "Stop"

Write-Host "[test] Running contracts lint..." -ForegroundColor Cyan
./scripts/validate_contracts.ps1

Write-Host "[test] Running dashboard-api tests..." -ForegroundColor Cyan
Push-Location services/dashboard-api
python -m pip install -r requirements.txt
python -m pip install -r requirements-dev.txt
python -m pytest -q
Pop-Location

Write-Host "[test] Running python SDK tests..." -ForegroundColor Cyan
Push-Location sdk/python
python -m pip install -r requirements.txt
python -m pip install -r requirements-dev.txt
$env:PYTHONPATH = (Get-Location).Path
python -m pytest -q tests
Pop-Location

Write-Host "[test] Building web app..." -ForegroundColor Cyan
npm --prefix apps/web install
npm --prefix apps/web run build

Write-Host "[test] All checks passed." -ForegroundColor Green
