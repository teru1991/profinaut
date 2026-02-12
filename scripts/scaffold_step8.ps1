$ErrorActionPreference = "Stop"

$dirs = @(
  "services/dashboard-api/alembic/versions",
  "services/dashboard-api/tests"
)

foreach ($d in $dirs) {
  New-Item -ItemType Directory -Force -Path $d | Out-Null
}

$files = @(
  "services/dashboard-api/alembic/versions/0005_reconcile_results.py",
  "services/dashboard-api/tests/test_api.py",
  "contracts/openapi/control-plane.v1.yaml",
  "contracts/schemas/reconcile.schema.json",
  "docs/roadmap.md",
  "docs/assumptions.md",
  "docs/changelog.md"
)

foreach ($f in $files) {
  if (-not (Test-Path $f)) {
    New-Item -ItemType File -Path $f | Out-Null
  }
}

Write-Host "Step 8 scaffold paths created. Populate files with repository source content." -ForegroundColor Green
