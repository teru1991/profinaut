# UCEL Runtime Wiring (Boundary SSOT)

## Rule

- `ucel/` is a reusable **library-only** workspace.
- Executable runtime/binary crates must live outside UCEL.
- Therefore, no `bin` crate is allowed under `ucel/crates/**`.

## Runtime entrypoint

- WS subscriber executable: `services/marketdata-rs/ucel-ws-subscriber`

## Wiring order (runtime dependency flow)

`ucel-ws-subscriber` -> `ucel-registry` -> `ucel-cex-*` -> `ucel-transport/ws` -> `ucel-journal` -> `ucel-subscription-store` -> `ucel-subscription-planner` -> `ucel-ws-rules`

## SSOT map

- Operation coverage: `ucel/coverage/*.yaml`
- Exchange catalogs: `docs/exchanges/*/catalog.json`
- WS rules model: `ucel/crates/ucel-ws-rules`

## Why this split

- UCEL stays stable as reusable crates for multiple apps/services.
- Runtime ownership and deployment concerns stay in service layer (`services/**`).
- Entrypoint discovery is unambiguous: runtime starts from `services/marketdata-rs/ucel-ws-subscriber`.
