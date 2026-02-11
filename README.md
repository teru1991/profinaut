# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 4 delivers the Python Agent SDK MVP with heartbeat, command safety (TTL/idempotency), ACK flow, and dead-man switch behavior.

## What is included
- Contracts SSOT with OpenAPI + JSON Schemas (`contracts/`).
- Backend core service at `services/dashboard-api` (Step 2).
- Frontend app at `apps/web` with navigation skeleton and bots polling (Step 3).
- Python Agent SDK MVP at `sdk/python`:
  - periodic heartbeat (default 30s)
  - command processing with required `command_id` idempotency handling
  - TTL enforcement via `expires_at` with `REJECTED_EXPIRED`
  - command ACK publishing back to control plane
  - dead-man switch fallback (`SAFE_MODE` default, `FLATTEN` optional)

## Quick start (Windows 11 + Docker Desktop)
1. Copy environment file:
   ```powershell
   Copy-Item .env.example .env
   ```
2. Start stack:
   ```powershell
   ./scripts/dev.ps1
   ```
3. Run all checks:
   ```powershell
   ./scripts/test.ps1
   ```
4. Run Python SDK tests only:
   ```powershell
   npm run test:sdk-python
   ```

## Cross-platform npm commands
```bash
npm run dev
npm run test
npm run migrate
npm run web:dev
npm run web:build
npm run test:sdk-python
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
- Dashboard never stores exchange API keys.
