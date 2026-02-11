$ErrorActionPreference = "Stop"

Write-Host "[migrate] Running Alembic migrations for dashboard-api..." -ForegroundColor Cyan
Push-Location services/dashboard-api
python -m pip install -r requirements.txt
alembic upgrade head
Pop-Location

Write-Host "[migrate] Migration completed." -ForegroundColor Green
