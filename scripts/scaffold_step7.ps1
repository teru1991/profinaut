$ErrorActionPreference = "Stop"

$dirs = @(
  "services/dashboard-api/alembic/versions",
  "services/dashboard-api/tests",
  "apps/web/app/api/portfolio/exposure",
  "apps/web/app/portfolio"
)

foreach ($d in $dirs) {
  New-Item -ItemType Directory -Force -Path $d | Out-Null
}

$files = @(
  "services/dashboard-api/alembic/versions/0004_metrics_positions.py",
  "services/dashboard-api/tests/test_api.py",
  "apps/web/app/api/portfolio/exposure/route.ts",
  "apps/web/app/portfolio/page.tsx"
)

foreach ($f in $files) {
  if (-not (Test-Path $f)) {
    New-Item -ItemType File -Path $f | Out-Null
  }
}

Write-Host "Step 7 scaffold paths created. Populate files with repository source content." -ForegroundColor Green
