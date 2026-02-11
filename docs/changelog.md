# Changelog

## 2026-02-11 — Step 2 (Backend core + auth MVP + health)
- Added FastAPI backend service scaffold in `services/dashboard-api`.
- Added SQLAlchemy models and initial Alembic migration for:
  - `bots`
  - `instances`
  - `bot_status`
  - `audit_logs`
  - `modules`
  - `module_runs`
- Implemented endpoints:
  - `GET /healthz`
  - `POST /ingest/heartbeat` (upsert)
  - `GET /bots` (paginated, admin token)
  - module registry CRUD (`GET/POST/GET:id/DELETE /modules`, admin token)
- Added pytest coverage for health, heartbeat upsert, auth checks, and module CRUD.
- Updated docker compose to run API service with migrations and uvicorn.
- Updated scripts and npm commands for migration and API tests.
- Updated CI to run backend tests in addition to contracts.

## 2026-02-11 — Step 1 (Contracts SSOT + CI enforcement)
- Added OpenAPI contract at `contracts/openapi/control-plane.v1.yaml`.
- Added JSON Schemas:
  - `heartbeat.schema.json`
  - `command.schema.json`
  - `ack.schema.json`
  - `reconcile.schema.json`
  - `audit.schema.json`
  - `module.schema.json`
  - `module_run.schema.json`
- Added contract validation scripts:
  - `scripts/validate_contracts.ps1`
  - `scripts/validate_json_schemas.py`
- Updated npm scripts to include `contracts:lint` and made `test` run contract checks.
- Updated CI workflow to enforce contract linting/validation.
- Updated README with contract SSOT and validation commands.

## 2026-02-11 — Step 0 (Project Initialization)
- Added required monorepo directory structure for contracts, services, SDKs, apps, infra, scripts, and docs.
- Added Docker Compose baseline with PostgreSQL and placeholders for dashboard API and web app.
- Added `.env.example` with core, auth, database, service port, and webhook placeholders.
- Added PowerShell scripts: `scripts/dev.ps1`, `scripts/test.ps1`, `scripts/migrate.ps1`.
- Added cross-platform npm scripts in `package.json`: `dev`, `test`, `migrate`.
- Added minimal CI skeleton workflow in `.github/workflows/ci.yml`.
- Updated `README.md` with quick start and repository conventions.
