# Profinaut V2.5+ — Multi-Exchange / Multi-Language Bot Management Dashboard

Step 0 initializes the monorepo layout and local development workflow for a Web/PWA control plane that targets Windows, Mac, and Linux users.

## What is included in Step 0
- Required directory structure for contracts, services, SDKs, apps, infra, scripts, and docs.
- Docker Compose baseline with PostgreSQL and service placeholders.
- PowerShell helper scripts for development, tests, and migrations.
- Cross-platform npm scripts (`dev`, `test`, `migrate`) for users who prefer npm over PowerShell.
- Baseline documentation:
  - `docs/roadmap.md`
  - `docs/assumptions.md`
  - `docs/changelog.md`
- Minimal CI skeleton at `.github/workflows/ci.yml`.

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

## Notes
- **Contracts-first**: `contracts/` is the source of truth for API and schema definitions.
- **Security baseline**: API keys must never be stored in the dashboard.
- **Time policy**: store timestamps in UTC ISO-8601; UI can render local time.
