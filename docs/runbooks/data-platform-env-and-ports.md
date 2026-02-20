# Data Platform Env and Ports

All compose variables and defaults are defined in `infra/env/.env.example`.

## Variables and defaults

| Variable | Default | Purpose |
|---|---|---|
| `COMPOSE_PROJECT_NAME` | `profinaut-dataplat` | Compose project/network/volume prefix |
| `POSTGRES_DB` | `profinaut` | Default app DB |
| `POSTGRES_USER` | `profinaut_app` | App DB user |
| `POSTGRES_PASSWORD` | `change-me-postgres` | App DB password |
| `POSTGRES_PORT` | `5432` | Host postgres port |
| `POSTGRES_MEM_LIMIT` | `768m` | Postgres memory limit |
| `POSTGRES_CPUS` | `1.0` | Postgres CPU quota |
| `CLICKHOUSE_DB` | `profinaut` | Default ClickHouse DB |
| `CLICKHOUSE_USER` | `profinaut_app` | ClickHouse user |
| `CLICKHOUSE_PASSWORD` | `change-me-clickhouse` | ClickHouse password |
| `CLICKHOUSE_HTTP_PORT` | `8123` | ClickHouse HTTP port |
| `CLICKHOUSE_NATIVE_PORT` | `9000` | ClickHouse native port |
| `CLICKHOUSE_MEM_LIMIT` | `2g` | ClickHouse memory limit |
| `CLICKHOUSE_CPUS` | `2.0` | ClickHouse CPU quota |
| `VALKEY_PASSWORD` | `change-me-valkey` | Valkey auth password |
| `VALKEY_PORT` | `6379` | Valkey host port |
| `VALKEY_MEM_LIMIT` | `512m` | Valkey memory limit |
| `VALKEY_CPUS` | `0.75` | Valkey CPU quota |
| `OBJECTSTORE_ACCESS_KEY` | `change-me-objectstore-access` | SeaweedFS S3 access key |
| `OBJECTSTORE_SECRET_KEY` | `change-me-objectstore-secret` | SeaweedFS S3 secret key |
| `OBJECTSTORE_REGION` | `us-east-1` | S3 region |
| `OBJECTSTORE_BUCKETS` | `profinaut-bronze,profinaut-silver,profinaut-gold` | Buckets created by init |
| `SEAWEEDFS_MASTER_PORT` | `9333` | SeaweedFS master admin port |
| `SEAWEEDFS_FILER_PORT` | `8888` | SeaweedFS filer HTTP port |
| `SEAWEEDFS_S3_PORT` | `8333` | S3-compatible endpoint port |
| `OBJECTSTORE_MEM_LIMIT` | `512m` | SeaweedFS service memory limit |
| `OBJECTSTORE_CPUS` | `1.0` | SeaweedFS service CPU quota |

## CI-safe defaults
`docker-compose.yml` uses `${VAR:-default}` for all typed/numeric-sensitive fields (`cpus`, `mem_limit`, and host ports) to avoid blank-string interpolation failures in CI.

## Override examples
```bash
# one-off shell overrides
export POSTGRES_CPUS=2.0
export SEAWEEDFS_S3_PORT=18333
docker compose config

# file-based overrides
cp infra/env/.env.example infra/env/.env
# edit values
docker compose --env-file infra/env/.env up -d
```
