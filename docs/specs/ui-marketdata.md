# UI Specification: MarketData Ticker Monitor

## Overview

Add a dedicated page at `/market` to continuously monitor MarketData `/ticker/latest` for pre-trade health checks.

## Scope

- Route: `apps/web/app/market/page.tsx`
- Local UI component(s): `apps/web/app/market/*`
- API proxy route: `apps/web/app/api/ticker/route.ts`

## UX Rules

- Poll every 5 seconds.
- Read data from `/api/ticker`.
- Show ticker fields:
  - `exchange`
  - `symbol`
  - `ts_utc` (fallback to `timestamp`)
  - `bid`
  - `ask`
  - `last` (if absent, render `-`)
- For non-200 responses, show a clear error banner:
  - Title format: `Error (status)`
  - Body: backend message text

## API Proxy Rules

- Proxy target: `${MARKETDATA_API_BASE_URL}/ticker/latest` with query passthrough.
- Cache policy: `no-store`.
- Preserve upstream `content-type`.
- If upstream is unreachable, return:
  - HTTP 502
  - JSON `{ "error": "bad_gateway", "message": "<details>" }`

## Environment Variables

- `MARKETDATA_API_BASE_URL` (primary)
- `MARKETDATA_BASE_URL` (fallback)
- Default fallback URL: `http://127.0.0.1:8081`
