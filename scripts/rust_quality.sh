#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   ./scripts/rust_quality.sh
#
# Fails if:
# - rustfmt diff exists
# - compilation warnings exist (RUSTFLAGS=-Dwarnings)
# - clippy warnings exist (-D warnings)
# - rustdoc warnings exist (RUSTDOCFLAGS=-Dwarnings)
# - tests fail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -f "${ROOT_DIR}/ucel/Cargo.toml" ]]; then
  cd "${ROOT_DIR}/ucel"
else
  cd "$ROOT_DIR"
fi

echo "[rust-quality] rustfmt (check)"
cargo fmt --all -- --check

echo "[rust-quality] cargo check (warnings as errors)"
RUSTFLAGS="-D warnings" cargo check --workspace --all-targets

echo "[rust-quality] cargo test"
cargo test --workspace --all-targets

echo "[rust-quality] cargo clippy (warnings as errors)"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "[rust-quality] cargo doc (warnings as errors)"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

echo "[rust-quality] OK"
