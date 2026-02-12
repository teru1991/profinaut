$ErrorActionPreference = "Stop"

$dirs = @(
  "services/dashboard-api/alembic/versions",
  "services/dashboard-api/tests"
)

foreach ($d in $dirs) {
  New-Item -ItemType Directory -Force -Path $d | Out-Null
}

$files = @(
  "services/dashboard-api/alembic/versions/0006_cost_ledger.py",
  "services/dashboard-api/tests/test_api.py",
  "services/dashboard-api/app/main.py",
  "services/dashboard-api/app/models.py",
  "services/dashboard-api/app/schemas.py",
  "contracts/openapi/control-plane.v1.yaml",
  "docs/roadmap.md",
  "docs/assumptions.md",
  "docs/changelog.md"
)

foreach ($f in $files) {
  if (-not (Test-Path $f)) {
    New-Item -ItemType File -Path $f | Out-Null
  }
}

Write-Host "Step 9 scaffold paths created. Populate files with repository source content." -ForegroundColor Green
