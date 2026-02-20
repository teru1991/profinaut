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
