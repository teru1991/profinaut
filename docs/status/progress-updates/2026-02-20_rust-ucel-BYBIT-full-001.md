# RUST-UCEL-BYBIT-FULL-001 Progress Update

- Task: BYBIT Full-Coverage 基盤SSOT化
- Scope: `rust-ucel-BYBIT-ssot-contracts-coverage-gate-perf-base`

## Catalog ingestion counts (SSOT source: `docs/exchanges/bybit/catalog.json`)

- REST rows: 77
- WS rows: 19
- Contract target rows (REST+WS): 96

## Mapping rule fixed (single source of truth)

1. BYBIT rows are mapped in one function (`map_bybit_operation`) using catalog row fields (`id`, optional `operation`) only.
2. `requires_auth` is mechanically decided from `visibility=private` (with fallback visibility extraction from `.public.` / `.private.` in catalog `id` when `visibility` is omitted).
3. Optional `requires_auth` field is validated against visibility and rejected on conflict (`CATALOG_INVALID`).

## Coverage gate design (BYBIT track)

- Manifest path: `ucel/coverage/BYBIT.yaml`
- Manifest mode: `strict: false` (warn-only in this task)
- Entries: all 96 catalog ids are enumerated as gate targets.
- Gate behavior: no gaps => pass, gaps+strict=true => fail, gaps+strict=false => warn.

## perf base note

- UCEL baseline remains: typed deserialize (`serde` types, no `serde_json::Value` in SSOT rails), `Bytes`-based transport, and bounded channel / backpressure for WS.

## Next task handoff

- 次タスクで BYBIT の全 catalog 行を `implemented/tested=true` へ段階的に埋め、coverage gate を strict 化して未実装を fail に昇格させる。
