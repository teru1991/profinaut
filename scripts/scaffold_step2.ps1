$ErrorActionPreference = "Stop"

$dirs = @(
  "services/dashboard-api/app",
  "services/dashboard-api/alembic/versions",
  "services/dashboard-api/tests"
)

foreach ($d in $dirs) {
  New-Item -ItemType Directory -Force -Path $d | Out-Null
}

$files = @(
  "services/dashboard-api/requirements.txt",
  "services/dashboard-api/requirements-dev.txt",
  "services/dashboard-api/app/__init__.py",
  "services/dashboard-api/app/config.py",
  "services/dashboard-api/app/database.py",
  "services/dashboard-api/app/models.py",
  "services/dashboard-api/app/schemas.py",
  "services/dashboard-api/app/auth.py",
  "services/dashboard-api/app/main.py",
  "services/dashboard-api/alembic.ini",
  "services/dashboard-api/alembic/env.py",
  "services/dashboard-api/alembic/script.py.mako",
  "services/dashboard-api/alembic/versions/0001_initial.py",
  "services/dashboard-api/tests/conftest.py",
  "services/dashboard-api/tests/test_api.py"
)

foreach ($f in $files) {
  if (-not (Test-Path $f)) {
    New-Item -ItemType File -Path $f | Out-Null
  }
}

Write-Host "Step 2 scaffold paths created. Populate files with repository source content." -ForegroundColor Green
