# MarketData local environment (MinIO + Postgres)

This runbook brings up a standalone local infra stack for MarketData without touching the repository root `docker-compose.yml`.

## Files

- Compose file: `infra/compose/marketdata-local.yml`

## Prerequisites

- Docker and Docker Compose v2 (`docker compose` command)
- Free local ports (default): `9000`, `9001`, `5432`

## 1) Create `.env` (example)

Create an env file (for example `infra/compose/.env.marketdata-local`) with values like:

```env
# MinIO
MINIO_ROOT_USER=minioadmin
MINIO_ROOT_PASSWORD=minioadmin
MINIO_API_PORT=9000
MINIO_CONSOLE_PORT=9001
BRONZE_BUCKET=bronze-raw

# Postgres
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=profinaut_marketdata
POSTGRES_PORT=5432

# App-side examples
MINIO_ENDPOINT=http://localhost:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin
MINIO_BUCKET=bronze-raw
OBJECT_STORE_BACKEND=s3

POSTGRES_DSN=postgresql://postgres:postgres@localhost:5432/profinaut_marketdata
DB_DSN=postgresql://postgres:postgres@localhost:5432/profinaut_marketdata
```

## 2) Start stack

From repository root:

```bash
docker compose --env-file infra/compose/.env.marketdata-local -f infra/compose/marketdata-local.yml up -d
```

Check status:

```bash
docker compose --env-file infra/compose/.env.marketdata-local -f infra/compose/marketdata-local.yml ps
```

## 3) Verify endpoints

- MinIO API health: `http://localhost:9000/minio/health/ready`
- MinIO Console: `http://localhost:9001`
- Postgres port: `localhost:5432`

## 4) Bucket creation options

### Option A: Auto-create (default)

The `mc-init` service auto-creates `${BRONZE_BUCKET}` on startup.

### Option B: Manual (mc command)

If you prefer manual bucket creation:

```bash
docker run --rm --network host \
  -e MC_HOST_local=http://minioadmin:minioadmin@localhost:9000 \
  minio/mc mb --ignore-existing local/bronze-raw
```

You can also inspect:

```bash
docker run --rm --network host \
  -e MC_HOST_local=http://minioadmin:minioadmin@localhost:9000 \
  minio/mc ls local
```

### Option C: Manual (UI)

1. Open MinIO Console `http://localhost:9001`
2. Login with `MINIO_ROOT_USER` / `MINIO_ROOT_PASSWORD`
3. Create bucket (for example `bronze-raw`)

## 5) Stop stack

```bash
docker compose --env-file infra/compose/.env.marketdata-local -f infra/compose/marketdata-local.yml down
```

To remove volumes too:

```bash
docker compose --env-file infra/compose/.env.marketdata-local -f infra/compose/marketdata-local.yml down -v
```
