# Data Platform Local Runbook

## Start
```bash
docker compose --env-file infra/env/.env.example up -d
```

## Stop
```bash
docker compose --env-file infra/env/.env.example stop
```

## Reset (destructive)
```bash
docker compose --env-file infra/env/.env.example down -v
```

## Verify health
```bash
docker compose --env-file infra/env/.env.example ps
```
Expected state: `postgres`, `clickhouse`, `valkey`, `seaweedfs-master`, `seaweedfs-volume`, `seaweedfs-filer`, `seaweedfs-s3` are `healthy` and `objectstore-init` exits `0`.

## Verify backends
Run smoke commands in `docs/runbooks/data-platform-backend-smoke.md`.
