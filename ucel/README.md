# UCEL Rust Foundation (v1.1.4)

UCEL workspace (`ucel/`) is **library-only**. Do not add executable `bin` crates under `ucel/crates`.

## Runtime entrypoint

WS subscription runtime entrypoint is located in services workspace:

- `services/marketdata-rs/ucel-ws-subscriber`

## SSOT pointers

- Coverage SSOT: `ucel/coverage/*.yaml`
- Exchange catalog SSOT: `docs/exchanges/*/catalog.json`
- WS rules crate: `ucel/crates/ucel-ws-rules` (rules file format is planned to migrate to TOML)

## Design docs

- [WS Full-Coverage Ingest Design (SSOT)](docs/ws-full-coverage-design.md)
- [Runtime wiring and boundary rules](docs/runtime-wiring.md)
