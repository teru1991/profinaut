# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 2 delivers backend core APIs with FastAPI + Alembic + PostgreSQL, admin token auth, and health endpoint.

## What is included
- Contracts SSOT with OpenAPI + JSON Schemas (`contracts/`).
- Backend core service at `services/dashboard-api`:
  - DB models + Alembic migration for `bots`, `instances`, `bot_status`, `audit_logs`, `modules`, `module_runs`.
  - APIs:
    - `GET /healthz`
    - `POST /ingest/heartbeat` (upsert status)
    - `GET /bots` (paginated, admin token)
    - `GET/POST/GET:id/DELETE /modules` (admin token)
- PowerShell scripts for dev/test/migrate.
- Cross-platform npm scripts for dev/test/migrate.
- CI checks for contracts + backend tests.

## Quick start (Windows 11 + Docker Desktop)
1. Copy environment file:
   ```powershell
   Copy-Item .env.example .env
   ```
2. Start stack:
   ```powershell
   ./scripts/dev.ps1
   ```
3. Run tests:
   ```powershell
   ./scripts/test.ps1
   ```
4. Apply migrations only:
   ```powershell
   ./scripts/migrate.ps1
   ```

## Cross-platform npm commands
```bash
npm run dev
npm run test
npm run migrate
```

## Repository layout
```text
/contracts/{openapi,schemas}
/services/{dashboard-api,notification,chatops,analytics}
/sdk/{python,node,go}
/apps/web
/infra
/scripts
/docs
```

## Security baseline
- Dashboard uses replaceable local admin auth header: `X-Admin-Token` from `.env`.
- Exchange API keys are not stored in dashboard services.
