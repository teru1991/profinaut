# Dashboard internal audit (UI-DASH-001)

## Existing pages / navigation
- Global shell is `StatusRibbon` + `NavShell` from `app/layout.tsx`.
- Sidebar links currently route to: `/dashboard`, `/bots`, `/portfolio`, `/markets`, `/commands`, `/analytics`, `/datasets`, `/admin/modules`.
- Existing dashboard (`app/dashboard/page.tsx`) is a static KPI + degraded table + quick links view with 10s polling.

## Current data-fetching flows
- `/dashboard` page polls:
  - `GET /api/status/summary`
  - `GET /api/bots?page=1&page_size=50`
- `StatusRibbon` independently polls `GET /api/status/summary` every 10s.
- Proxy API routes:
  - `/api/status/summary` -> upstream `/api/status/summary` with `X-Admin-Token`
  - `/api/bots` -> upstream `/bots`
  - `/api/commands` -> upstream `/commands` (GET/POST)

## Styling conventions observed
- Shared design tokens + CSS variables in `app/globals.css`.
- Reusable classes: `card`, `card-grid`, `btn`, `badge`, `notice`, `page-header`, table utilities.
- Components commonly use a mix of utility classes and small inline style objects for layout tuning.

---
## UI-DASH-002 progress notes
- what_done:
  - Added widget registry/runtime utilities, core widgets, panel chrome, widget catalog, and grid layout drag/resize.
  - Refactored `DashboardWorkspace` to use registry-driven panel rendering with duplicate/remove/lock/pin/protect controls.
- what_next:
  - Verify pointer drag/resize ergonomics against live browser interactions and tune if needed.
- errors:
  - None during implementation.
- commands_next:
  - `cd apps/web`
  - `npm install --no-package-lock`
  - `npm run build`

---
## UI-DASH-004 progress notes
- what_done:
  - Added personal local audit model/store + `/audit` page with filter/export/detail JSON drawer.
  - Added incident console (`/incidents`) with degraded drilldown, local checklist persistence, notes, and audit NOTE entries.
  - Added local snapshots store + save buttons + `/snapshots` listing/export/note annotations with audit logging.
  - Added safe danger action guard for command execution requiring reason + TTL + type-to-confirm; wired into `/commands` PAUSE/RESUME.
  - Updated sidebar navigation for quick access to Incidents/Snapshots/Audit.
- what_next:
  - Consider sharing actor identity from authenticated profile instead of static `local-user`.
  - Consider richer incident suggested checks per component type.
- errors:
  - None.
- commands_next:
  - `cd apps/web`
  - `npm install --no-package-lock`
  - `npm run build`
