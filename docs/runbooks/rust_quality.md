# Rust Quality Runbook

## Purpose
Maintain zero Rust compiler warnings, zero Clippy warnings, and zero rustdoc warnings.

## Local checks
- macOS/Linux:
  - `./scripts/rust_quality.sh`
- Windows PowerShell:
  - `.\scripts\rust_quality.ps1`

## What these checks enforce
1. `cargo fmt --all -- --check`
2. `RUSTFLAGS="-D warnings" cargo check --workspace --all-targets`
3. `cargo test --workspace --all-targets`
4. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
5. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`

## Common lint fixes
- `unwrap` / `expect`: replace with `?`, `ok_or_else`, and typed errors.
- `unused` imports/vars: remove them; if intentional, narrow scope and name `_var` only when required.
- `dead_code`: remove unused code or make usage explicit via tests/call sites.
- doc warnings: fix broken intra-doc links and add/adjust public API docs.

## Exception policy for `#[allow(...)]`
Only use `#[allow(...)]` as a last resort.
- Scope must be the smallest possible (function/block).
- Add a brief justification comment with safety/quality rationale.
- Add tests/guards that prove behavior remains safe.
