# Troubleshooting: `/api/bots` returns 502

If the web app responds with `502` on:

```text
GET /api/bots?page=1&page_size=50
```

it means the Next.js proxy cannot reach the bots backend.

## Which backend serves bots in this repo?

| Item | Value |
| --- | --- |
| Service | `services/dashboard-api` |
| Endpoint | `GET /bots` |
| Health | `GET /healthz` |
| Auth | `X-Admin-Token` header |

## Environment variables used by web proxy

`apps/web/app/api/bots/route.ts` resolves upstream in this order:

1. `DASHBOARD_API_BASE_URL`
2. `DASHBOARD_API_URL`
3. fallback: `http://localhost:8000`

Token header value is resolved in this order:

1. `DASHBOARD_ADMIN_TOKEN`
2. `ADMIN_TOKEN`
3. fallback: `test-admin-token`

## Recommended values

### Local host dev (web + dashboard-api both on host)

```bash
DASHBOARD_API_BASE_URL=http://localhost:8000
DASHBOARD_ADMIN_TOKEN=test-admin-token
```

### Docker compose dev (web in container, backend service name)

```bash
DASHBOARD_API_BASE_URL=http://dashboard-api:8000
DASHBOARD_ADMIN_TOKEN=test-admin-token
```

## Quick verification

1. Start dashboard-api.
2. Start web app.
3. Check proxy:

```bash
curl -i "http://localhost:3001/api/bots?page=1&page_size=50"
```

- With backend up: expect upstream status/body from `/bots` (typically `200` JSON).
- With backend down: expect `502` JSON with `upstream` and actionable `message`.
