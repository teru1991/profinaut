# Data Platform Local Stack Runbook

This runbook describes how to run the local data platform stack used by Profinaut:
- SeaweedFS (S3-compatible object store)
- ClickHouse (OLAP serving)
- PostgreSQL (OLTP ledger/ops)
- Valkey (cache)

It follows SSOT expectations in `docs/data-platform/*` for store roles, secret handling, and operational checks.

## 1) Prerequisites
- Docker + Docker Compose plugin installed
- Local shell with `psql`, `clickhouse-client`, and `valkey-cli` for direct checks (optional but recommended)

## 2) Configure environment
```bash
cp infra/env/.env.example infra/env/.env
# Edit secrets/ports as needed before first startup.
```

All credentials and ports are centralized in `infra/env/.env` (never commit real secrets).


## 2.1) Compose interpolation defaults (safe when vars are unset)
`docker-compose.yml` now defines fallback defaults for SeaweedFS/object-store runtime sizing and ports, so `docker compose config` works even in a clean shell:

- `SEAWEEDFS_MASTER_PORT` → `9333`
- `SEAWEEDFS_S3_PORT` → `8333`
- `OBJECTSTORE_MEM_LIMIT` → `512m`
- `OBJECTSTORE_CPUS` → `1.0`

Override options (highest precedence first):
1. Exported shell variables (e.g. `export OBJECTSTORE_CPUS=2.0`)
2. Values in `infra/env/.env` when passed via `--env-file infra/env/.env`
3. Compose fallback defaults in `docker-compose.yml`

Verification commands:
```bash
# clean-shell behavior (uses compose defaults)
unset SEAWEEDFS_S3_PORT SEAWEEDFS_MASTER_PORT OBJECTSTORE_MEM_LIMIT OBJECTSTORE_CPUS
docker compose config > /dev/null

# explicit overrides
export SEAWEEDFS_S3_PORT=18333
export SEAWEEDFS_MASTER_PORT=19333
export OBJECTSTORE_MEM_LIMIT=768m
export OBJECTSTORE_CPUS=2.0
docker compose config | sed -n '/seaweedfs:/,/objectstore-init:/p'
```


## 3) Start / Stop / Reset

### Start
```bash
docker compose --env-file infra/env/.env up -d
```

### Stop
```bash
docker compose --env-file infra/env/.env stop
```

### Full teardown (containers + volumes)
```bash
docker compose --env-file infra/env/.env down -v
```

### Reset from clean state
```bash
docker compose --env-file infra/env/.env down -v
docker compose --env-file infra/env/.env up -d
```

## 4) Health verification

### Service health status
```bash
docker compose --env-file infra/env/.env ps
```

### PostgreSQL
```bash
PGPASSWORD="${POSTGRES_PASSWORD}" psql \
  -h 127.0.0.1 -p "${POSTGRES_PORT}" -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" \
  -c 'SELECT 1;'
```

### ClickHouse
```bash
clickhouse-client \
  --host 127.0.0.1 --port "${CLICKHOUSE_NATIVE_PORT}" \
  --user "${CLICKHOUSE_USER}" --password "${CLICKHOUSE_PASSWORD}" \
  --query 'SELECT 1;'
```

### Valkey
```bash
valkey-cli -h 127.0.0.1 -p "${VALKEY_PORT}" -a "${VALKEY_PASSWORD}" --no-auth-warning ping
```
Expected result: `PONG`

### Object store (SeaweedFS S3)
```bash
AWS_ACCESS_KEY_ID="${OBJECTSTORE_ACCESS_KEY}" \
AWS_SECRET_ACCESS_KEY="${OBJECTSTORE_SECRET_KEY}" \
aws --endpoint-url "http://127.0.0.1:${SEAWEEDFS_S3_PORT}" s3 ls
```

## 5) Inspect object storage buckets/objects

List buckets:
```bash
AWS_ACCESS_KEY_ID="${OBJECTSTORE_ACCESS_KEY}" \
AWS_SECRET_ACCESS_KEY="${OBJECTSTORE_SECRET_KEY}" \
aws --endpoint-url "http://127.0.0.1:${SEAWEEDFS_S3_PORT}" s3api list-buckets
```

List objects in Gold bucket:
```bash
AWS_ACCESS_KEY_ID="${OBJECTSTORE_ACCESS_KEY}" \
AWS_SECRET_ACCESS_KEY="${OBJECTSTORE_SECRET_KEY}" \
aws --endpoint-url "http://127.0.0.1:${SEAWEEDFS_S3_PORT}" s3 ls s3://profinaut-gold/
```

## 6) Backup snapshot & restore

### PostgreSQL logical backup/restore
Backup:
```bash
mkdir -p ./backups
PGPASSWORD="${POSTGRES_PASSWORD}" pg_dump \
  -h 127.0.0.1 -p "${POSTGRES_PORT}" -U "${POSTGRES_USER}" "${POSTGRES_DB}" \
  > "./backups/postgres-$(date +%Y%m%d-%H%M%S).sql"
```

Restore (from latest file example):
```bash
PGPASSWORD="${POSTGRES_PASSWORD}" psql \
  -h 127.0.0.1 -p "${POSTGRES_PORT}" -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" \
  < ./backups/postgres-YYYYmmdd-HHMMSS.sql
```

### ClickHouse data-dir snapshot/restore
Backup volume tarball:
```bash
mkdir -p ./backups
docker run --rm \
  -v profinaut-dataplat_clickhouse_data:/source:ro \
  -v "$(pwd)/backups:/backup" \
  alpine sh -c 'tar -czf /backup/clickhouse-data-$(date +%Y%m%d-%H%M%S).tgz -C /source .'
```

Restore from tarball:
```bash
docker compose --env-file infra/env/.env stop clickhouse
docker run --rm \
  -v profinaut-dataplat_clickhouse_data:/target \
  -v "$(pwd)/backups:/backup:ro" \
  alpine sh -c 'rm -rf /target/* && tar -xzf /backup/clickhouse-data-YYYYmmdd-HHMMSS.tgz -C /target'
docker compose --env-file infra/env/.env start clickhouse
```

## 7) Troubleshooting

### Port already in use
- Update conflicting `*_PORT` values in `infra/env/.env`.
- Restart stack with updated env file.

### Permission denied on volumes
- Ensure Docker daemon user can write named volumes.
- On Linux, verify no stale root-owned bind mounts were introduced accidentally.

### Volume/data corruption symptoms
- Stop affected service.
- Recover from latest backup snapshot.
- If unrecoverable for local dev, run `docker compose --env-file infra/env/.env down -v` then `up -d`.

### Object store init container fails
- Check SeaweedFS health first:
  ```bash
  docker compose --env-file infra/env/.env logs seaweedfs
  ```
- Re-run init step after SeaweedFS is healthy:
  ```bash
  docker compose --env-file infra/env/.env up --force-recreate objectstore-init
  ```
