# Control Plane: Bots Endpoint Specification

## Overview

The `/bots` endpoint provides a paginated list of bot statuses for the dashboard UI. It returns a consistent envelope structure with bot state information including degradation status and last heartbeat time.

**Service:** `dashboard-api`  
**Endpoint:** `GET /bots`  
**Version:** 1.0.0

## Purpose

This endpoint allows operators to:
- Monitor all registered bots and their current status
- Identify degraded or stale bots (missing heartbeats)
- Track bot version, exchange, symbol, and runtime mode
- Support pagination for large bot fleets

## Authentication

Requires admin authentication via `X-Admin-Token` header.

**Response (401 Unauthorized):** Missing or invalid admin token

## Request

### Query Parameters

| Parameter | Type | Required | Default | Constraints | Description |
|-----------|------|----------|---------|-------------|-------------|
| `page` | integer | No | 1 | ≥ 1 | Page number for pagination |
| `page_size` | integer | No | 50 | 1 ≤ x ≤ 200 | Number of items per page |

### Example Request

```http
GET /bots?page=1&page_size=50
X-Admin-Token: your-admin-token
```

## Response

### Success Response (200 OK)

Returns a paginated envelope with bot status items.

**Response Structure:**
```json
{
  "page": 1,
  "page_size": 50,
  "total": 123,
  "items": [...]
}
```

### Envelope Fields

| Field | Type | Description |
|-------|------|-------------|
| `page` | integer | Current page number (1-indexed) |
| `page_size` | integer | Number of items per page |
| `total` | integer | Total number of bots across all pages |
| `items` | array | Array of `BotStatus` objects |

### BotStatus Object

Each item in the `items` array contains:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bot_id` | string | Yes | Unique bot identifier |
| `name` | string | Yes | Human-readable bot name |
| `strategy_name` | string | Yes | Strategy name (e.g., "simple_mm") |
| `state` | string | Yes | Current state: `RUNNING`, `STOPPED`, `UNKNOWN` |
| `degraded` | boolean | Yes | `true` if bot is degraded (stale heartbeat), `false` otherwise |
| `degraded_reason` | string \| null | Yes | Reason for degradation (e.g., `STALE_HEARTBEAT`) or `null` |
| `instance_id` | string \| null | Yes | Current instance ID or `null` if never started |
| `runtime_mode` | string \| null | Yes | Runtime mode: `PAPER`, `LIVE`, or `null` |
| `exchange` | string \| null | Yes | Exchange name (e.g., `BINANCE`) or `null` |
| `symbol` | string \| null | Yes | Trading symbol (e.g., `BTCUSDT`) or `null` |
| `status` | string \| null | Yes | Instance status from database or `null` |
| `last_seen` | string \| null | Yes | **UTC ISO 8601 timestamp** of last heartbeat or `null` if never seen |
| `version` | string \| null | Yes | Bot version string or `null` |

### Field Constraints

#### `last_seen` Format

- **MUST** be a valid ISO 8601 timestamp with UTC timezone when present
- **MUST** be `null` if no heartbeat has been received
- Format: `YYYY-MM-DDTHH:MM:SS.ffffffZ` or `YYYY-MM-DDTHH:MM:SS+00:00`
- Examples:
  - `"2026-02-14T08:35:00.123456Z"`
  - `"2026-02-14T08:35:00+00:00"`
  - `null`

#### `degraded` and `degraded_reason`

- A bot is considered degraded when its heartbeat is stale (configurable threshold, default: 300 seconds)
- When `degraded=true`, `degraded_reason` should explain why (e.g., `STALE_HEARTBEAT`)
- When `degraded=false`, `degraded_reason` must be `null`

#### `state` Values

The `state` field reflects the instance status:
- `RUNNING`: Instance is active and running
- `STOPPED`: Instance has been stopped
- `UNKNOWN`: No instance information available or instance status unknown

### Example Response (With Bots)

```json
{
  "page": 1,
  "page_size": 50,
  "total": 2,
  "items": [
    {
      "bot_id": "bot-1",
      "name": "Market Maker Bot 1",
      "strategy_name": "simple_mm",
      "state": "RUNNING",
      "degraded": false,
      "degraded_reason": null,
      "instance_id": "inst-1",
      "runtime_mode": "PAPER",
      "exchange": "BINANCE",
      "symbol": "BTCUSDT",
      "status": "RUNNING",
      "last_seen": "2026-02-14T08:35:00.123456Z",
      "version": "1.0.1"
    },
    {
      "bot_id": "bot-2",
      "name": "Market Maker Bot 2",
      "strategy_name": "simple_mm",
      "state": "UNKNOWN",
      "degraded": false,
      "degraded_reason": null,
      "instance_id": null,
      "runtime_mode": null,
      "exchange": null,
      "symbol": null,
      "status": null,
      "last_seen": null,
      "version": null
    }
  ]
}
```

### Example Response (Empty List)

When no bots are registered:

```json
{
  "page": 1,
  "page_size": 50,
  "total": 0,
  "items": []
}
```

## Behavior Guarantees

### Always Returns 200

- The endpoint **MUST** return HTTP 200 (OK) when authenticated, even with zero bots
- Empty result set returns `{"page": 1, "page_size": 50, "total": 0, "items": []}`

### Pagination Stability

- `page` and `page_size` in response match request parameters
- `total` reflects the total count across all pages
- Items are ordered by `bot_id` for consistent pagination

### Timestamp Handling

- All `last_seen` timestamps are normalized to UTC
- Timezone-naive timestamps from the database are converted to UTC
- Output format is always ISO 8601 with timezone information

### Degradation Detection

- Degradation is calculated server-side based on `last_seen` age
- Threshold: 300 seconds (5 minutes) by default
- Degradation status is computed per request (not cached)

## Error Responses

### 401 Unauthorized

Missing or invalid `X-Admin-Token` header.

```json
{
  "detail": "Invalid or missing admin token"
}
```

### 422 Unprocessable Entity

Invalid query parameters (e.g., `page=0`, `page_size=1000`).

```json
{
  "detail": [
    {
      "loc": ["query", "page"],
      "msg": "ensure this value is greater than or equal to 1",
      "type": "value_error.number.not_ge"
    }
  ]
}
```

## UI Integration Notes

The frontend (`apps/web`) should:
1. Display `state` as the primary status indicator
2. Show visual alerts when `degraded=true` with the `degraded_reason`
3. Format `last_seen` for user display (e.g., "2 minutes ago" with tooltip showing full UTC timestamp)
4. Handle `null` values gracefully (show "N/A" or similar)
5. Poll this endpoint periodically (recommended: every 5 seconds) for real-time updates

## Implementation Notes

### Database Schema

The endpoint queries:
- `bots` table: Core bot information (`bot_id`, `name`, `strategy_name`)
- `bot_status` table: Latest heartbeat data (`last_seen`, `version`, runtime info)
- `instances` table: Instance status information

### Performance

- Uses `LEFT OUTER JOIN` to include bots without instances/status
- Pagination with `OFFSET` and `LIMIT` for efficient large-scale queries
- Count query separate from data query for accuracy

### Future Enhancements

Potential additions (not in current version):
- Filtering by `state`, `exchange`, `symbol`
- Sorting by different fields
- Search by bot name
- Aggregated status counts in response envelope
