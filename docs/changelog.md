# Changelog

## 2026-02-11 — Step 0 (Project Initialization)
- Added required monorepo directory structure for contracts, services, SDKs, apps, infra, scripts, and docs.
- Added Docker Compose baseline with PostgreSQL and placeholders for dashboard API and web app.
- Added `.env.example` with core, auth, database, service port, and webhook placeholders.
- Added PowerShell scripts: `scripts/dev.ps1`, `scripts/test.ps1`, `scripts/migrate.ps1`.
- Added cross-platform npm scripts in `package.json`: `dev`, `test`, `migrate`.
- Added minimal CI skeleton workflow in `.github/workflows/ci.yml`.
- Updated `README.md` with quick start and repository conventions.
