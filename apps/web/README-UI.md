# Profinaut Web UI — Design System & Polish Guide

## Theme Approach

The UI uses **CSS custom properties** (CSS variables) defined in `app/globals.css` for all theming.

### Light / Dark Toggle
- **Default**: Dark theme (matches `data-theme="dark"` or no attribute)
- **Light theme**: Applied via `data-theme="light"` on `<html>`
- **Persistence**: `localStorage` key `profinaut-theme`
- **System preference**: Falls back to `prefers-color-scheme` media query when no stored preference
- **Implementation**: `components/ThemeProvider.tsx` provides React context; toggle button lives in the sidebar footer

### CSS Variable Categories
| Category | Examples |
|----------|---------|
| Spacing | `--space-1` (4px) through `--space-16` (64px) |
| Typography | `--text-xs` (11px) through `--text-2xl` (28px) |
| Colors — Backgrounds | `--bg-body`, `--bg-surface`, `--bg-surface-hover`, `--bg-surface-raised` |
| Colors — Text | `--text-primary`, `--text-secondary`, `--text-muted` |
| Colors — Semantic | `--color-success-*`, `--color-warning-*`, `--color-error-*`, `--color-info-*` |
| Colors — Brand | `--color-accent`, `--color-accent-hover`, `--color-accent-subtle` |
| Borders | `--border-default`, `--border-subtle`, `--border-strong` |
| Radius | `--radius-sm` (4px) through `--radius-full` (9999px) |
| Shadows | `--shadow-sm`, `--shadow-md`, `--shadow-lg` |
| Transitions | `--transition-fast` (120ms), `--transition-base` (200ms), `--transition-slow` (300ms) |

---

## Component Inventory

### Layout Components
| Component | File | Description |
|-----------|------|-------------|
| App Shell | `components/NavShell.tsx` | Collapsible sidebar + main area. Mobile responsive with hamburger menu. |
| Status Ribbon | `components/StatusRibbon.tsx` | Top bar showing system health. Polls `/api/status/summary` every 30s. |
| Theme Provider | `components/ThemeProvider.tsx` | React context for light/dark theme with localStorage persistence. |
| Page Header | CSS class `.page-header` | Flex layout with title/subtitle on left, actions on right. |
| Section | CSS class `.section` | Vertical flex container for related content groups. |

### Data Components
| Component | File / Class | Description |
|-----------|-------------|-------------|
| Card | `.card`, `.card-compact` | Surface container with border and padding. |
| Card Grid | `.card-grid`, `.card-grid-2`, `.card-grid-3` | Responsive grid layouts for cards. |
| KPI Card | `.kpi-card` | Metric display with label, value, subtitle. Variants: `kpi-success`, `kpi-warning`, `kpi-error`. |
| Badge | `.badge` | Inline pill/tag. Variants: `badge-success`, `badge-warning`, `badge-error`, `badge-info`, `badge-accent`. |
| Table | `.table-wrapper` + `.table` | Responsive table with sticky headers and row hover. |
| Inline Table | `.table-inline` | Key-value table without wrapper border. |
| Bots Table | `components/BotsTable.tsx` | Full-featured table with search, sort, copy-to-clipboard, loading skeleton, empty/error states. |

### State Components
| Component | Class | Description |
|-----------|-------|-------------|
| Loading Skeleton | `.skeleton` + `.skeleton-text`, `.skeleton-heading`, `.skeleton-card`, `.skeleton-row` | Shimmer animation placeholders. |
| Empty State | `.empty-state` | Centered icon + title + description for empty data views. |
| Error State | `.error-state` | Red-tinted box with title and message for error display. |
| Inline Notice | `.notice` + `.notice-info`, `.notice-warning`, `.notice-error`, `.notice-success` | Alert/notice boxes with icon and content. |
| Placeholder Page | `.placeholder-page` | Full-page centered placeholder for unimplemented features. |

### Action Components
| Component | Class / File | Description |
|-----------|-------------|-------------|
| Button | `.btn` | Standard button. Variants: `btn-primary`, `btn-danger`, `btn-success`, `btn-ghost`, `btn-sm`, `btn-icon`. |
| Confirm Dialog | `components/DangerousActionDialog.tsx` | Two-step confirmation modal for dangerous operations. Uses `.dialog-overlay` + `.dialog`. |
| Copy Button | `.copy-btn` | Inline copy-to-clipboard trigger. |
| Search Input | `.search-input` | Styled search field with focus ring. |

### Utility Module
| Module | File | Description |
|--------|------|-------------|
| Format helpers | `lib/format.ts` | `formatNumber`, `formatCompact`, `formatCurrency`, `formatBps`, `formatPct`, `formatDuration`, `formatTimestamp`, `formatRelative`, `copyToClipboard` |

---

## Pages (Routes)

| Route | Status | Features |
|-------|--------|----------|
| `/` | Redirect | Redirects to `/dashboard` |
| `/dashboard` | Polished | KPI cards (status, bots, active, degraded), degraded component table, quick nav grid. Loading skeleton. |
| `/bots` | Polished | Page header + BotsTable with search, sort, loading skeleton, empty/error states, copy bot ID. Kill switch panel. |
| `/portfolio` | Polished | KPI cards (net/gross exposure, equity, positions), exposure-by-symbol table, loading skeleton, empty/error states. |
| `/markets` | Polished | Market selector form, ticker KPI cards (bid/ask/last/mid), degraded notice, loading skeleton, empty/error states. |
| `/commands` | Polished | Bot selector, PAUSE/RESUME buttons with policy gating, last ack card, command history table with badges, loading skeleton, empty/error states. |
| `/analytics` | Placeholder | Styled placeholder with info notice about planned features. |
| `/datasets` | Placeholder | Styled placeholder with info notice about planned features. |
| `/admin/modules` | Placeholder | Styled placeholder with info notice about planned features. |
| `/market` | Redirect | Legacy redirect to `/markets` |

---

## Manual QA Steps

### Build Verification
```bash
cd apps/web
npm run build
```
Build must complete without errors.

### Visual Verification (each route)
For each page listed above:
1. Navigate to the route
2. Verify: page header renders (title + subtitle)
3. Verify: loading skeleton appears briefly before data loads
4. Verify: data renders correctly when loaded
5. Verify: error state renders if backend is unreachable
6. Verify: empty state renders if data is empty

### Theme Toggle
1. Click the theme toggle in the sidebar footer
2. Verify: all surfaces, text, borders switch between light and dark
3. Refresh the page — theme selection persists
4. Open browser DevTools, clear localStorage, refresh — verify it defaults to system preference

### Navigation
1. Click each nav item — verify active state highlights correctly
2. Click the sidebar collapse button — verify sidebar collapses to icons only
3. Resize browser to <1024px — verify mobile hamburger menu appears
4. Open mobile menu — verify nav works and backdrop closes menu

### Accessibility
1. Tab through navigation links — verify visible focus ring
2. Tab through dialog buttons — verify focus ring and keyboard activation
3. Verify icon-only buttons have `aria-label` attributes
4. Verify dialog has `role="dialog"` and `aria-modal="true"`

### Security
1. Open browser DevTools Network tab
2. Navigate pages and verify no `X-Admin-Token` header appears in client-side requests
3. Verify all API calls go through `/api/*` Next.js routes (server-side proxy)
4. Inspect page source — verify no admin token in HTML or JS bundles

---

## Assumptions

- No new npm dependencies were added. All styling is pure CSS with CSS custom properties.
- SVG icons are inline (no icon library dependency).
- The legacy `/market` route is preserved as a redirect.
- The `TickerCard.tsx` in `/market` is kept but unused (legacy).
- All data fetching continues through existing Next.js API routes with server-side token injection.
- The dashboard page aggregates data from existing `/api/status/summary` and `/api/bots` endpoints.
