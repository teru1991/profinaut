$ErrorActionPreference = "Stop"

$dirs = @(
  "sdk/python/profinaut_agent",
  "sdk/python/tests"
)

foreach ($d in $dirs) {
  New-Item -ItemType Directory -Force -Path $d | Out-Null
}

$files = @(
  "sdk/python/requirements.txt",
  "sdk/python/requirements-dev.txt",
  "sdk/python/run_agent.py",
  "sdk/python/profinaut_agent/__init__.py",
  "sdk/python/profinaut_agent/models.py",
  "sdk/python/profinaut_agent/processor.py",
  "sdk/python/profinaut_agent/deadman.py",
  "sdk/python/profinaut_agent/client.py",
  "sdk/python/profinaut_agent/source.py",
  "sdk/python/profinaut_agent/agent.py",
  "sdk/python/tests/test_processor.py",
  "sdk/python/tests/test_deadman.py",
  "sdk/python/tests/test_runtime.py"
)

foreach ($f in $files) {
  if (-not (Test-Path $f)) {
    New-Item -ItemType File -Path $f | Out-Null
  }
}

Write-Host "Step 4 scaffold paths created. Populate files with repository source content." -ForegroundColor Green
