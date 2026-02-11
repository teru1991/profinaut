# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 1 establishes contracts as SSOT with OpenAPI + JSON Schemas and CI enforcement.

## What is included
- Required directory structure for contracts, services, SDKs, apps, infra, scripts, and docs.
- Docker Compose baseline with PostgreSQL and service placeholders.
- PowerShell helper scripts for development, tests, migrations, and contract validation.
- Cross-platform npm scripts (`dev`, `test`, `migrate`, `contracts:lint`).
- Baseline documentation:
  - `docs/roadmap.md`
  - `docs/assumptions.md`
  - `docs/changelog.md`
- CI contract checks in `.github/workflows/ci.yml`.

## Quick start
1. Copy environment file:
   ```powershell
   Copy-Item .env.example .env
   ```
2. Start local services:
   ```powershell
   ./scripts/dev.ps1
   ```
   or
   ```bash
   npm run dev
   ```
3. Run baseline checks:
   ```powershell
   ./scripts/test.ps1
   ```
4. Run contract-only checks:
   ```powershell
   ./scripts/validate_contracts.ps1
   ```
   or
   ```bash
   npm run contracts:lint
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

## Contracts (SSOT)
- OpenAPI source: `contracts/openapi/control-plane.v1.yaml`
- JSON Schemas:
  - `contracts/schemas/heartbeat.schema.json`
  - `contracts/schemas/command.schema.json`
  - `contracts/schemas/ack.schema.json`
  - `contracts/schemas/reconcile.schema.json`
  - `contracts/schemas/audit.schema.json`
  - `contracts/schemas/module.schema.json`
  - `contracts/schemas/module_run.schema.json`

## Notes
- **Contracts-first**: `contracts/` is the source of truth for API and schema definitions.
- **Security baseline**: API keys must never be stored in the dashboard.
- **Time policy**: store timestamps in UTC ISO-8601; UI can render local time.
