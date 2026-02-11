# Roadmap

## Vision
Build a contracts-first, module-driven bot management platform for multi-exchange trading operations with strict safety, auditability, and extensibility guarantees.

## Step Plan
- [x] **Step 0**: Project initialization (layout, compose, scripts, docs, CI skeleton).
- [x] **Step 1**: Contracts SSOT (OpenAPI + JSON Schemas) + CI enforcement.
- [ ] **Step 2**: Backend core (FastAPI + DB + Auth MVP + Health).
- [ ] **Step 3**: Frontend skeleton (Next.js pages + bots polling).
- [ ] **Step 4**: Python Agent SDK MVP (heartbeat, commands, dead man switch).
- [ ] **Step 5**: Command system end-to-end + audit persistence.
- [ ] **Step 6**: Notification router (Discord webhook phase 1).
- [ ] **Step 7**: Metrics/positions/exposure foundation + portfolio UI.
- [ ] **Step 8+**: Reconciliation, NetPnL extensions, execution quality, module expansion.

## Guardrails
- `contracts/` is SSOT and enforced in CI.
- Dashboard does not store exchange API keys.
- All command handling uses idempotency (`command_id`) and TTL (`expires_at`).
- Feature growth occurs via Module Registry and `module_runs`; avoid core bloat.
