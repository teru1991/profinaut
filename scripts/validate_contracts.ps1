$ErrorActionPreference = "Stop"

Write-Host "[contracts] Validating OpenAPI with Redocly..." -ForegroundColor Cyan
npx --yes @redocly/cli@latest lint contracts/openapi/control-plane.v1.yaml

Write-Host "[contracts] Validating JSON Schemas..." -ForegroundColor Cyan
python scripts/validate_json_schemas.py

Write-Host "[contracts] Contract validation completed." -ForegroundColor Green
