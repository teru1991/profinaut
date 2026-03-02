# UCEL-100P-STEP1-SSOT-MIGRATION-001 Verification

## 1) Changed files (`git diff --name-only`)
- docs/status/trace-index.json
- docs/verification/UCEL-100P-STEP1-SSOT-MIGRATION-001.md
- ucel/coverage/binance-coinm.yaml
- ucel/coverage/binance-options.yaml
- ucel/coverage/binance-usdm.yaml
- ucel/coverage/binance.yaml
- ucel/coverage/bitbank.yaml
- ucel/coverage/bitflyer.yaml
- ucel/coverage/bitget.yaml
- ucel/coverage/bithumb.yaml
- ucel/coverage/bitmex.yaml
- ucel/coverage/bittrade.yaml
- ucel/coverage/bybit.yaml
- ucel/coverage/coinbase.yaml
- ucel/coverage/coincheck.yaml
- ucel/coverage/deribit.yaml
- ucel/coverage/gmocoin.yaml
- ucel/coverage/htx.yaml
- ucel/coverage/kraken.yaml
- ucel/coverage/okx.yaml
- ucel/coverage/sbivc.yaml
- ucel/coverage/upbit.yaml

## 2) What / Why
- Migrated all `ucel/coverage/*.yaml` files to coverage schema v1 header + entries shape.
- Added `scope`, top-level `implemented/tested`, normalized `strict=false` (Step1 visibility-first).
- Ensured every catalog ID is represented in `entries`; missing legacy IDs were added as `implemented=false/tested=false` with notes.
- Added inferred `kind` and `access` per ID (`.ws.` / `.private.` rule) to align with v1 gate contract.
- Updated trace-index for this task only and linked this verification evidence.

## 3) Self-check results
- Allowed-path check OK: only `docs/**` and `ucel/coverage/**` were modified.
- Tests: `cargo test --manifest-path ucel/Cargo.toml --workspace --all-targets` **failed** due to pre-existing compile errors in `ucel-cex-sbivc` (`EndpointAllowlist` / `SubdomainPolicy` unresolved).
- SSOT validate OK: `python scripts/ssot/validate_ucel_ssot.py` passed.
- trace-index json.tool OK: `python -m json.tool docs/status/trace-index.json > /dev/null` passed.
- Secrets scan OK: `rg -n "(AKIA|BEGIN RSA|BEGIN PRIVATE KEY|password\s*=)" docs/status/trace-index.json ucel/coverage/*.yaml` produced no findings.

## 4) History checks evidence (required 0.1)
- `git log --oneline --decorate -n 50`: HEAD is `d43d456` (merge PR #415, SSOT v1 extension).
- `git log --graph --oneline --decorate --all -n 80`: branch is linear from `d43d456`; local stash `938ee3f` exists from preflight cleanup attempt.
- `git show HEAD`: confirms prior task added SSOT specs/gate (`8e005d4` merged by `d43d456`)—this task is consistent with that direction.
- `git reflog -n 30`: shows checkout from `work` to `feature/ucel-100p-step1-ssot-migration-001` at same SHA.
- `git merge-base HEAD origin/<default-branch>`: not runnable in this environment because no `origin` remote is configured.
- `git branch -vv`: local branches `work` and `feature/ucel-100p-step1-ssot-migration-001` both at `d43d456`.
- `git log --merges --oneline -n 30`: recent merges include #415/#414/#413 with SSOT continuity.
- `git blame -w` spot checks:
  - `ucel/coverage/bithumb.yaml`: latest structural changes from `b5f912f`.
  - `docs/exchanges/bithumb/catalog.json`: catalog source authored in `99720fd8`.
  - `ucel/crates/ucel-testkit/src/ssot_gate.rs`: v1-capable gate added in `8e005d4`.
- Conclusion: Step1 migration aligns with current SSOT v1 contract/gate trajectory and does not conflict with recent merge intent.

## 5) Step2 handoff
- Suggested strict=true candidate venues (already fully green in Step1 data):
  - `binance-coinm`, `binance-options`, `binance-usdm`, `binance`, `bitbank`, `bitflyer`, `bittrade`, `bybit`, `coincheck`, `gmocoin`, `okx`.
- Venues with remaining unmet entries (implemented/tested false):
  - `bitget`(1), `bithumb`(13), `bitmex`(1), `coinbase`(4), `deribit`(33), `htx`(2), `kraken`(3), `upbit`(1).
- Step2 focus: complete unmet entries (notably `bithumb` and `deribit`) then enable strict per venue incrementally.
