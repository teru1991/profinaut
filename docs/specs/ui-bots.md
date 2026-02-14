# UI Specification: Bots Status Page

## Overview

The bots status page (`/bots`) provides operators with real-time visibility into bot health, degraded states, and system-wide kill switch status. This specification defines the UI requirements for consistent degraded UX, clear error presentation, and read-only kill switch display.

**Location:** `apps/web/app/bots/page.tsx` → `apps/web/components/BotsTable.tsx`  
**Version:** 1.0.0  
**Last Updated:** 2026-02-14

## Purpose

The bots status page enables operators to:
- Monitor all registered bots and their current state
- Quickly identify degraded or stale bots
- Understand error conditions with clear status codes and messages
- View kill switch status (read-only)

## Key Principles

### 1. "壊れた時にすぐ分かる" (Immediately visible when broken)

**Operational priority:** When something is broken, operators must know instantly.

**Implementation:**
- Degraded bots show prominent visual indicators
- Error messages include HTTP status codes
- No silent failures

### 2. Consistent Degraded UX

**Requirements:**
- Degraded badge always visible when `degraded=true`
- Degraded reason always displayed when present
- Visual hierarchy: State → Degraded indicator → Reason
- Color coding: Red tones for degraded states

### 3. Clear Error Presentation

**Non-200 Response Display:**
- Show HTTP status code explicitly: "Error (502)"
- Display full error message from backend
- Distinguish network errors (status 0) from server errors
- Error banner styling: Dark red background, distinct from table

### 4. Read-Only Kill Switch Panel

**Requirements:**
- Kill switch status always visible below bots table
- Shows current state: ACTIVE or INACTIVE
- Includes read-only mode indicator
- No action buttons or controls in this phase

## UI Components

### Bots Table

**Columns:**
1. Bot (name)
2. State (primary status + degraded badge)
3. Last Seen (UTC timestamp)
4. Version
5. Runtime (mode)
6. Exchange
7. Symbol

**State Column Layout:**
```
[STATE_BADGE] [DEGRADED_BADGE (if degraded)]
degraded_reason (if present, smaller text, red tone)
```

**Degraded Badge:**
- Background: `#991b1b` (dark red)
- Text: `#fecaca` (light red)
- Text: "DEGRADED"

**Degraded Reason:**
- Font size: `0.75rem`
- Color: `#fca5a5` (red-300)
- Displayed below badges

### Error Banner

**Display Conditions:**
- Shown when API returns non-200 status
- Shown when network errors occur

**Layout:**
```
Error (STATUS_CODE)
error_message_from_backend
```

**Styling:**
- Background: `#7f1d1d` (dark red)
- Border: `1px solid #991b1b`
- Border radius: `8px`
- Padding: `12px`
- Margin bottom: `16px`

**Status Code Display:**
- Include in title: "Error (502)"
- Omit if network error (status 0): "Error"

### Kill Switch Panel

**Location:** Below bots table with 24px margin-top

**Layout:**
```
Kill Switch
[STATUS_BADGE] Read-only mode

Kill switch is currently inactive. All bots can operate normally.
Control actions are not available in this view.
```

**Status Badge:**
- INACTIVE state:
  - Background: `#065f46` (dark green)
  - Text: `#6ee7b7` (light green)
  - Text: "INACTIVE"

**Container Styling:**
- Background: `#1f2937`
- Border: `1px solid #374151`
- Border radius: `8px`
- Padding: `12px`

## Data Handling

### Bot Row Fields

**Required Fields:**
- `bot_id`: string
- `name`: string
- `state` or `status`: string (fallback chain: state → status → "UNKNOWN")
- `degraded`: boolean (check with `=== true`)
- `degraded_reason`: string | null

**Optional Fields:**
- `instance_id`, `runtime_mode`, `exchange`, `symbol`, `last_seen`, `version`

### Error State

**Type:**
```typescript
type ErrorState = {
  status: number;    // HTTP status code, 0 for network errors
  message: string;   // Error message from backend or client
} | null;
```

**Error Extraction:**
1. Check response.ok
2. Parse content-type (JSON or text)
3. Extract message from JSON fields: `message` → `error` → fallback
4. Display with status code

### Polling Behavior

- Interval: 5 seconds
- Auto-refresh on mount
- Clear interval on unmount
- Show total count: "Total: X"

## Acceptance Criteria

### ✅ Degraded UX
- [ ] Degraded badge visible when `degraded=true`
- [ ] Degraded reason displayed when present
- [ ] Visual hierarchy: badges first, reason below
- [ ] Consistent red color scheme for degraded states

### ✅ Error Presentation
- [ ] Non-200 responses show status code
- [ ] Error message clearly displayed in banner
- [ ] Network errors (status 0) handled gracefully
- [ ] Error banner visually distinct from table

### ✅ Kill Switch Panel
- [ ] Panel visible below bots table
- [ ] Shows current state (INACTIVE by default)
- [ ] Includes "Read-only mode" indicator
- [ ] Explanatory text present
- [ ] No action controls present

## Future Enhancements (Out of Scope)

The following are explicitly **not** included in this phase:

- ❌ Kill switch control actions (POST/execute commands)
- ❌ Bot detail pages (`/bots/:id`)
- ❌ Command execution UI
- ❌ Relative time formatting ("2 minutes ago")
- ❌ Bot filtering or sorting controls
- ❌ Real kill switch API integration

## Implementation Notes

### File Modifications

**Allowed:**
- `apps/web/components/BotsTable.tsx` - Main UI component
- `apps/web/app/bots/page.tsx` - Route wrapper (if needed)
- `docs/specs/ui-bots.md` - This specification

**Forbidden:**
- Backend services
- Contract definitions
- CI/CD workflows
- Docker configurations
- Lock files

### Testing

**Manual Verification:**
1. Start web app in dev mode
2. Verify bots table renders
3. Test error scenarios (backend down, network error)
4. Verify degraded badge display
5. Verify kill switch panel presence
6. Take screenshot of final UI

**Error Simulation:**
- Stop backend to trigger 502 error
- Check error banner shows "Error (502)" with message
- Verify table remains stable

### Styling

Uses existing CSS classes from `apps/web/app/globals.css`:
- `.card` - Container cards
- `.table` - Table layout
- `.badge` - Status badges

Custom inline styles for:
- Error banner (red theme)
- Kill switch panel (neutral theme)
- Degraded reason text (red-300)

## References

- Backend API Spec: `docs/specs/controlplane-bots.md`
- UI Audit: `docs/audits/ui-current-vs-spec.md`
- Tasks: T191 (field alignment), T192 (kill switch), T193 (degraded banner)
