# Reconcile Mismatch Repair Runbook (Canary-first)

## Goal
When reconciliation reports `MISMATCH`, operators must use a **safety-first, canary-first** process:
1. stop risk expansion,
2. classify mismatch pattern,
3. repair consistency,
4. restore in canary scope first,
5. roll back immediately if mismatch persists.

This runbook standardizes recovery for delay/dup/missing/out-of-order mismatch classes.

## Scope and references
- Reconcile endpoints are available in the platform surface:
  - `POST /reconcile`
  - `GET /reconcile/results`
- Related health/safety context:
  - `GET /healthz`
  - `GET /capabilities`
- Safety assumptions:
  - dead-man fallback is `SAFE_MODE` by default and can be configured to `FLATTEN`

Reference documents:
- `README.md` (reconcile and platform endpoints)
- `docs/assumptions.md` (dead-man + safety assumptions)
- `docs/runbooks/e2e-smoke-runbook.md` (service triage pattern)

---

## 1) Detection: identify mismatch and blast radius

### 1.1 Primary detection signals
1. Poll/inspect `GET /reconcile/results` for newest records where `status=MISMATCH`.
2. Confirm timestamp, `instance_id`, and recurrence count (single spike vs repeated).
3. Check alerting channel for WARNING notifications tied to reconciliation mismatch.

### 1.2 Supporting checks (stability before repair)
1. Verify service liveness with `GET /healthz`.
2. Verify degradation/safety status with `GET /capabilities` (`safe_mode`, `degraded_reason`, policy posture).
3. Inspect runtime logs for reconcile executions and related ingest/order timing around mismatch timestamps.

### 1.3 Blast radius quick assessment
Classify scope before action:
- **Low**: single bot/symbol/exchange, short time window.
- **Medium**: multiple symbols within one instance.
- **High**: cross-instance or repeated mismatch across cycles.

If scope is Medium/High, prefer entering stronger safety state immediately (see section 3).

---

## 2) Classification: mismatch pattern taxonomy

Classify into one primary pattern (and optional secondary):

1. **Delay**
   - Signals: expected events eventually appear, but outside reconciliation window.
   - Typical causes: queue lag, temporary upstream latency, clock skew.
2. **Dup (duplicate)**
   - Signals: same logical event counted more than once; idempotency boundary missed.
   - Typical causes: retry without idempotency key alignment, replay overlap.
3. **Missing**
   - Signals: expected state transition/event never arrives.
   - Typical causes: dropped ingest, failed write, connector gap.
4. **Out-of-order**
   - Signals: events exist but sequence/ordering violates assumptions.
   - Typical causes: parallel stream merge race, inconsistent ordering key/time source.

If pattern is unclear, treat as **Missing + Out-of-order (worst-case)** until proven otherwise.

---

## 3) Immediate safety actions (decision criteria)

> Principle: choose the minimum action that prevents further inconsistency, but escalate quickly when uncertain.

### 3.1 Enter `SAFE_MODE` (default first action)
Enter `SAFE_MODE` when:
- mismatch is newly detected and root cause is not yet confirmed,
- new order creation may increase divergence,
- health is up but data consistency is uncertain.

Expected behavior: block new order flow while preserving observability and repair operations.

### 3.2 Use `CLOSE_ONLY` when reduction is needed but controlled unwind is safe
`CLOSE_ONLY` is acceptable only when all are true:
- mismatch scope is narrow and understood,
- current exposure needs risk reduction,
- no evidence of widespread missing/out-of-order state corruption.

Do **not** use `CLOSE_ONLY` as a substitute for containment in unknown/high-severity mismatches.

### 3.3 Require `FLATTEN` / `HALT` when severe or persistent
Escalate to `FLATTEN` (and then `HALT` if needed) when one or more apply:
- mismatch persists across multiple reconcile cycles after attempted repair,
- blast radius is High or expanding,
- state cannot be trusted for safe reduce-only operation,
- operators cannot confidently map true exposure.

Guideline:
- `FLATTEN`: emergency exposure neutralization.
- `HALT`: complete stop when even flatten workflow or state observability is unreliable.

---

## 4) Repair procedure (safe re-reconciliation)

### Step 0 — Incident record
- Open/update incident ticket with: first detection time, affected scope, selected safety mode, operator on-call.

### Step 1 — Stabilize inputs
- Keep system in `SAFE_MODE` or stricter.
- Confirm core services are healthy (`/healthz`) and degraded reason is explicit (`/capabilities`).
- Pause nonessential operational changes (deploys/config churn) during repair.

### Step 2 — Build expected vs observed snapshot
- Use latest trusted source-of-truth snapshot for affected entities (bot/symbol/exchange/time window).
- Compare with observed persisted state associated with mismatch result.
- Tag each discrepancy as delay/dup/missing/out-of-order.

### Step 3 — Apply corrective actions by class
- **Delay**: widen reconciliation window or re-run after lag clears; verify eventual convergence.
- **Dup**: deduplicate by idempotency key/logical event identity, then re-run reconcile.
- **Missing**: backfill/replay missing events from trusted upstream source, then re-run reconcile.
- **Out-of-order**: reorder by canonical sequence/timestamp rule and re-materialize derived state, then re-run reconcile.

### Step 4 — Re-run reconciliation safely
1. Trigger `POST /reconcile` for the smallest affected scope first.
2. Query `GET /reconcile/results` and verify latest result transitions from `MISMATCH` to consistent status.
3. Repeat per scope segment until all affected segments converge.

### Step 5 — Consistency confirmation
Require all before leaving repair phase:
- No new `MISMATCH` in targeted segments over agreed observation window.
- Alerts quiet for mismatch route.
- Capability/health remain stable (no new degradation reason linked to data integrity).

---

## 5) Canary-first restoration and rollback

### 5.1 Canary restoration flow (mandatory)
1. Keep global posture in `SAFE_MODE` (or stricter), then enable only a **small canary scope**:
   - one instance,
   - one or few low-risk symbols,
   - limited exposure/time window.
2. Run reconciliation repeatedly during canary observation window.
3. Validate no mismatch recurrence and no divergence trend in logs/alerts.
4. If successful, expand gradually (small batches) and repeat checks at each step.

### 5.2 Canary success criteria
All criteria must pass before full restoration:
- consecutive reconcile runs in canary scope show consistent results,
- mismatch alert count remains zero in canary window,
- no drift in exposure/state metrics for canary entities,
- operator sign-off recorded in incident ticket.

### 5.3 Rollback if mismatch persists
If any canary mismatch reappears:
1. Immediately return to prior strict posture (`SAFE_MODE` or `FLATTEN/HALT` as severity dictates).
2. Stop scope expansion.
3. Capture fresh evidence (reconcile result IDs, timestamps, logs).
4. Re-enter classification and repair loop from section 2/4.

Never proceed from canary to broad rollout with unresolved or recurrent mismatch signals.

---

## 6) Operator checklist

### During incident
- [ ] `MISMATCH` confirmed in `GET /reconcile/results`
- [ ] Safety posture selected (`SAFE_MODE` / `CLOSE_ONLY` / `FLATTEN` / `HALT`) with rationale
- [ ] Pattern classified: delay / dup / missing / out-of-order
- [ ] Repair steps executed and documented

### Before recovery promotion
- [ ] Canary scope defined and approved
- [ ] Canary reconcile checks all pass
- [ ] Rollback trigger conditions communicated
- [ ] Full restoration approved by on-call + incident owner
