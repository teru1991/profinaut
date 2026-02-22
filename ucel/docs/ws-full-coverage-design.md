# WS Full-Coverage Ingest Design (SSOT)

- Task ID: `UCEL-WS-DOC-000`
- Status: Approved design baseline (single-run serial chain)
- Scope tag: `ucel-ws-doc-v1`

## Goal

Using `ucel/coverage/*.yaml` as source of truth, subscribe to all `Public WS op_id × symbol` for all `ucel-cex-*` venues with:

1. Durable subscription queue
2. Raw WAL persistence
3. 2-level rate limiting (exchange + connection)
4. Supervisor orchestration

Private WS must be mechanism-supported. If credentials are missing, record entries as `Deadletter`.

## Canonical pipeline

`coverage -> symbols -> planner -> durable store -> ws manager -> journal`

Per exchange:

1. Read coverage and extract `crypto.public.ws.*` and `crypto.private.ws.*` ops.
2. Fetch symbols (if unsupported, mark as not supported/deadletter where applicable).
3. Load exchange WS rules from TOML (or conservative fallback when missing).
4. Generate shard plan and startup seed order.
5. Seed durable SQLite queue.
6. Start WS manager tasks; all sends go through limiter.
7. Receive raw frames and append to WAL first.
8. Advance queue states from `pending -> inflight -> active` or `deadletter`.
9. On reconnect, requeue active subscriptions to pending and resume drip sending.

## State model

Subscription state machine:

- `pending`: queued, not yet sent
- `inflight`: subscribe sent, waiting for ack/traffic
- `active`: confirmed by ACK or inbound data observation
- `deadletter`: terminal failure (auth/permission/not-supported/contract-limit/other)

Required transition rules:

- All transition persistence must be durable in store.
- No direct transition skipping durability.
- Reconnect must never trigger bulk blast; always resume from queue.

## Safety invariants (must hold globally)

1. **Limiter invariant:** every subscribe send flows through limiter (no bypass path).
2. **Journal-first invariant:** receive loop appends raw record before heavy decode/normalize.
3. **Resume invariant:** reconnect resumes from durable queue, not re-derived in memory.
4. **Coverage invariant:** coverage is SSOT; missing ops are CI-detectable.
5. **Conservative-default invariant:** unspecified exchange limits use conservative profile.

## Components and responsibilities

### `ucel-ws-rules`
- Defines WS constraints model (`Rate`, `HeartbeatPolicy`, `EntitlementPolicy`, `SupportLevel`, `SafetyProfile`).
- Loads `rules/*.toml`.
- Missing rule file -> `SupportLevel::Unknown` + `SafetyProfile::Conservative`.

### `ucel-subscription-planner`
- Reads `coverage/*.yaml`.
- Extracts WS op ids.
- Builds `SubscriptionKey { exchange_id, op_id, symbol, params }`.
- Produces shard plan and prioritized startup seed.

### `ucel-subscription-store`
- Durable SQLite queue and state transitions.
- Guarantees resume-after-restart and per-connection safe picking.

### `ucel-journal`
- Append-only NDJSON WAL of `RawRecord`.
- Rotation by size/time.
- fsync policies: safe/balanced.
- Reopen behavior tolerates partial last line.

### `ucel-transport` WS stack
- Connection management.
- Two-level limiter.
- Reconnect with backoff + jitter + storm guard.
- Heartbeat and idle/stall detection.
- Backpressure policy (v1 minimum: stop for safety).

### `ucel-ws-ingest`
- Supervisor wiring across exchanges.
- Startup path: coverage -> symbols -> rules -> plan -> store -> manager.
- Support exchange allowlist and single-exchange runs.

### `ucel-registry` + `ucel-cex-*`
- Unified ingest driver trait and registration.
- Standardized per-exchange layout (`symbols.rs`, `ws_manager.rs`, `channels/`).

## Ordering and rollout plan

Recommended order:

1. Foundation (`tasks 1-7`)
2. Ingest + registry (`tasks 8-9`)
3. CEX skeleton standardization (`task 10`)
4. Coverage gate tests (`task 11`)
5. Minimal E2E and operational safety (`task 12`)

## Test policy

Minimum checks expected as features land:

- Rules parsing and conservative defaults.
- Coverage reader correctness and venue-file consistency.
- Planner shard bounds and seed-size assertions.
- Store transitions and reopen-resume behavior.
- Journal append/read, rotation, partial-line handling.
- Limiter/backoff unit tests.
- Coverage-vs-channels consistency gate in CI.

## Naming and file conventions

- Op-id to filename mapping is defined in:
  - `ucel/docs/ws-full-coverage-ops-naming.md`
- One op per channel file for long-term completeness tracking.

## Non-goals for v1.0

- Full semantic decode/normalization in receive loop.
- Aggressive optimization before safety/durability guarantees.
- Exchange-specific perfection of all channel payloads on day one.

## Definition of done (program-level)

For at least one exchange in E2E:

- Subscribe sequence starts from durable seed.
- Inbound frames are persisted to WAL.
- Reconnect occurs on stall/failure.
- Resume dripping occurs from store states.
- Backpressure/failure triggers safe stop or throttle (no silent drops).

## 実装状況サマリ（2026-02時点）

本設計に対する現状の実装棚卸し（主要項目のみ）:

- 実装済み（骨格）
  - `ucel-ws-rules`, `ucel-subscription-planner`, `ucel-subscription-store`, `ucel-journal`, `ucel-ws-ingest` のcrate自体
  - coverage読込・購読seed生成・SQLite永続キュー・WAL append/rotateの基本ユニット
  - `ucel-transport/src/ws/*` に limiter/reconnect/heartbeat/backpressure の基礎モジュール
- 部分実装
  - supervisor起動フロー（coverage→rules→plan→store seed）
  - 取引所個別実装の一部（例: `ucel-cex-okx` は比較的実装が進んでいる）
- 未実装/要補完
  - 多くの `ucel-cex-*` で `ws_manager.rs` / `symbols.rs` / `channels/*` がスタブ
  - ingest本体での実WS接続・受信→journal-first永続化・ACK/受信ベースのActive遷移
  - reconnect時の `active/inflight -> pending` 再滴下の実運用経路
  - 取引所別 `rules/*.toml` の実データ整備（現状は保守的デフォルト依存）
  - coverage定義と `channels/*` 実装のCIゲート（1channel=1file）強制
  - stall検知/バックプレッシャー動作を ingestループへ実配線

堅牢化のための優先改良（欠損最小観点）:

1. `ws_manager` 実装を1取引所ずつ完成（接続・購読滴下・受信・再接続）
2. receiveループで `journal.append(raw)` を最短経路化（decode前）
3. `subscription-store` 状態遷移を接続イベントに厳密連動
4. 取引所制限TOML整備 + 観測ベースの安全側チューニング
5. coverage-vs-channel実装漏れをfail-fastで検知する契約テスト導入
