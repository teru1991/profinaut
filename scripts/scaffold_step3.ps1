$ErrorActionPreference = "Stop"

$dirs = @(
  "apps/web/app/dashboard",
  "apps/web/app/bots",
  "apps/web/app/portfolio",
  "apps/web/app/markets",
  "apps/web/app/analytics",
  "apps/web/app/datasets",
  "apps/web/app/admin/modules",
  "apps/web/app/api/bots",
  "apps/web/app/api/healthz",
  "apps/web/components",
  "apps/web/public"
)

foreach ($d in $dirs) {
  New-Item -ItemType Directory -Force -Path $d | Out-Null
}

$files = @(
  "apps/web/package.json",
  "apps/web/next.config.mjs",
  "apps/web/tsconfig.json",
  "apps/web/next-env.d.ts",
  "apps/web/app/globals.css",
  "apps/web/app/layout.tsx",
  "apps/web/app/page.tsx",
  "apps/web/app/dashboard/page.tsx",
  "apps/web/app/bots/page.tsx",
  "apps/web/app/portfolio/page.tsx",
  "apps/web/app/markets/page.tsx",
  "apps/web/app/analytics/page.tsx",
  "apps/web/app/datasets/page.tsx",
  "apps/web/app/admin/modules/page.tsx",
  "apps/web/app/api/bots/route.ts",
  "apps/web/app/api/healthz/route.ts",
  "apps/web/components/NavShell.tsx",
  "apps/web/components/BotsTable.tsx",
  "apps/web/public/manifest.webmanifest"
)

foreach ($f in $files) {
  if (-not (Test-Path $f)) {
    New-Item -ItemType File -Path $f | Out-Null
  }
}

Write-Host "Step 3 scaffold paths created. Populate files with repository source content." -ForegroundColor Green
