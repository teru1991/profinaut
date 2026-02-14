# E2E Smoke Runbook (Paper-first, Live-safe)

## Goal
Provide a **one-command** local verification for the paper path:
1. service health checks
2. capabilities check
3. ticker fetch
4. paper order-intent submission

This runbook is safe-by-default and does **not** enable live trading.

## One-command verification (paper)

From repo root:

```bash
SMOKE_AUTO_START=1 scripts/smoke/run_paper_e2e.sh
```

- `SMOKE_AUTO_START=1` wraps the standard local startup path (`docker compose up -d`) without modifying compose.
- If your stack is already running, you can skip auto-start:

```bash
scripts/smoke/run_paper_e2e.sh
```

## What the smoke script checks

1. `GET /healthz` for:
   - Control Plane: `http://127.0.0.1:8000/healthz`
   - Execution: `http://127.0.0.1:8001/healthz`
   - MarketData: `http://127.0.0.1:8081/healthz`
2. `GET /capabilities` from execution and prints status/features.
3. `GET /ticker/latest` from marketdata (query path first, then fallback path).
4. `POST /execution/order-intents` with paper payload (`exchange=binance`, `symbol=BTC/USDT`).

## Expected successful output

Look for lines like:

- `[smoke][ok] control-plane reachable`
- `[smoke][ok] execution status=... features=[...]`
- `[smoke][ok] ticker symbol=... bid=... ask=...`
- `[smoke][ok] paper order accepted order_id=... status=...`
- `[smoke][success] paper E2E smoke completed ...`

## If it fails: actionable triage

- `docker command not found` / daemon unreachable:
  - Install/start Docker Desktop/Engine.
- `... not reachable at .../healthz`:
  - Service did not boot. Check `docker compose ps` and service logs.
- `ticker is degraded/stale`:
  - MarketData is not healthy. Verify upstream connectivity and degraded reason.
- `paper order-intent request failed`:
  - Check execution logs and `ALLOWED_SYMBOLS` / `ALLOWED_EXCHANGES` env settings.

---

## Live migration checklist (paper -> live)

### 1) Flags to enable explicitly

- Bot side:
  - `EXECUTION_MODE=live`
  - `EXECUTION_LIVE_ENABLED=true`
- Execution service:
  - `EXECUTION_LIVE_ENABLED=true`
  - `GMO_API_BASE_URL` (sandbox or live endpoint as appropriate)
  - `GMO_API_KEY`, `GMO_API_SECRET` from runtime environment

> Keep defaults on paper and only enable live intentionally.

### 2) Safety checklist before live

- [ ] `SAFE_MODE` behavior verified (orders are skipped when enabled).
- [ ] Dead-man conditions verified (control-plane outage leads to order suppression).
- [ ] MarketData degraded/stale behavior verified (orders are skipped).
- [ ] Execution capabilities status is not degraded.
- [ ] Small-size dry-run strategy parameters are in place.

### 3) Rollback steps

If any risk signal appears:

1. Set `EXECUTION_MODE=paper` (bot side).
2. Set `EXECUTION_LIVE_ENABLED=false` (bot + execution service).
3. Restart affected services.
4. Re-run paper smoke:

```bash
scripts/smoke/run_paper_e2e.sh
```

5. Confirm only paper order acceptance is happening.
