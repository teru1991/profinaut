# Data Platform Backend Smoke

Run after `docker compose up -d`.

## PostgreSQL
```bash
PGPASSWORD=change-me-postgres psql -h 127.0.0.1 -p 5432 -U profinaut_app -d profinaut -c 'SELECT 1;'
PGPASSWORD=change-me-ledger psql -h 127.0.0.1 -p 5432 -U ledger_user -d ledger -c 'SELECT 1;'
PGPASSWORD=change-me-serving psql -h 127.0.0.1 -p 5432 -U serving_user -d serving -c 'SELECT 1;'
```

## ClickHouse
```bash
docker compose exec -T clickhouse clickhouse-client --query 'SELECT 1;'
docker compose exec -T clickhouse clickhouse-client --query 'SHOW TABLES FROM gold;'
```

## Valkey
```bash
valkey-cli -h 127.0.0.1 -p 6379 -a change-me-valkey --no-auth-warning ping
```

## SeaweedFS S3
```bash
AWS_ACCESS_KEY_ID=change-me-objectstore-access \
AWS_SECRET_ACCESS_KEY=change-me-objectstore-secret \
aws --endpoint-url http://127.0.0.1:8333 s3 ls
```
