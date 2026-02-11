# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 3 delivers the frontend skeleton (Next.js + TypeScript) with stable navigation and Bots polling.

## What is included
- Contracts SSOT with OpenAPI + JSON Schemas (`contracts/`).
- Backend core service at `services/dashboard-api` (Step 2).
- Frontend app at `apps/web` with route skeleton:
  - `/dashboard`
  - `/bots`
  - `/portfolio`
  - `/markets`
  - `/analytics`
  - `/datasets`
  - `/admin/modules`
- Bots page polling via internal Next API route (`/api/bots`) that injects admin token from env.
- PowerShell scripts for dev/test/migrate.
- Cross-platform npm scripts for Docker workflow and local web workflow.

## Quick start (Windows 11 + Docker Desktop)
1. Copy environment file:
   ```powershell
   Copy-Item .env.example .env
   ```
2. Start stack:
   ```powershell
   ./scripts/dev.ps1
   ```
3. Open UI: `http://localhost:3000/bots`
4. Run tests:
   ```powershell
   ./scripts/test.ps1
   ```

## Cross-platform npm commands
```bash
npm run dev
npm run test
npm run migrate
npm run web:dev
npm run web:build
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
- Frontend calls backend via Next API route with server-side token injection.
- Exchange API keys are not stored in dashboard services.
