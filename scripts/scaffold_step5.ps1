$ErrorActionPreference = "Stop"

$dirs = @(
  "services/dashboard-api/alembic/versions",
  "services/dashboard-api/tests"
)

foreach ($d in $dirs) {
  New-Item -ItemType Directory -Force -Path $d | Out-Null
}

$files = @(
  "services/dashboard-api/alembic/versions/0002_commands_and_acks.py",
  "services/dashboard-api/tests/test_api.py"
)

foreach ($f in $files) {
  if (-not (Test-Path $f)) {
    New-Item -ItemType File -Path $f | Out-Null
  }
}

Write-Host "Step 5 scaffold paths created. Populate files with repository source content." -ForegroundColor Green
