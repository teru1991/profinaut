# UCEL-CHAIN-EVM-011 Verification

## 1) Changed files (`git diff --name-only`)
- docs/specs/ucel/chain_evm_surface_v1.md
- docs/status/trace-index.json
- docs/verification/UCEL-CHAIN-EVM-011.md
- ucel/Cargo.lock
- ucel/crates/ucel-core/src/chain.rs
- ucel/crates/ucel-core/src/lib.rs
- ucel/crates/ucel-chain-ethereum/Cargo.toml
- ucel/crates/ucel-chain-ethereum/src/lib.rs
- ucel/crates/ucel-chain-ethereum/src/provider.rs
- ucel/crates/ucel-chain-ethereum/src/types.rs
- ucel/crates/ucel-chain-ethereum/src/balance.rs
- ucel/crates/ucel-chain-ethereum/src/tx.rs
- ucel/crates/ucel-chain-ethereum/src/nonce.rs
- ucel/crates/ucel-chain-ethereum/src/fees.rs
- ucel/crates/ucel-chain-ethereum/src/receipt.rs
- ucel/crates/ucel-chain-ethereum/src/logs.rs
- ucel/crates/ucel-chain-ethereum/src/finality.rs
- ucel/crates/ucel-chain-ethereum/src/reorg.rs
- ucel/crates/ucel-chain-ethereum/src/resume.rs
- ucel/crates/ucel-chain-ethereum/src/signer.rs
- ucel/crates/ucel-chain-ethereum/src/errors.rs
- ucel/crates/ucel-sdk/src/chain.rs
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-sdk/Cargo.toml
- ucel/crates/ucel-registry/src/lib.rs
- ucel/crates/ucel-testkit/src/chain_evm.rs
- ucel/crates/ucel-testkit/src/lib.rs
- ucel/crates/ucel-testkit/Cargo.toml
- ucel/crates/ucel-testkit/tests/chain_evm_provider_failover.rs
- ucel/crates/ucel-testkit/tests/chain_evm_balance_and_call.rs
- ucel/crates/ucel-testkit/tests/chain_evm_nonce_and_fee.rs
- ucel/crates/ucel-testkit/tests/chain_evm_tx_lifecycle.rs
- ucel/crates/ucel-testkit/tests/chain_evm_logs_and_resume.rs
- ucel/crates/ucel-testkit/tests/chain_evm_reorg_handling.rs
- ucel/docs/chains/evm_adapter_policy.md
- ucel/docs/chains/evm_finality_policy.md
- ucel/docs/chains/evm_nonce_and_fee_policy.md
- ucel/docs/chains/evm_reorg_resume_policy.md
- ucel/docs/chains/evm_support_matrix.md
- ucel/examples/chain_evm_preview.rs
- ucel/examples/chain_evm_logs_preview.rs
- ucel/fixtures/chain_evm/README.md

## 2) What / Why
- Replaced `ucel-chain-ethereum` placeholder behavior with practical EVM surface modules (provider abstraction, balances/calls, fee/nonce/tx, receipt/finality, logs/reorg/resume, signer and reason code mapping).
- Added canonical EVM model into `ucel-core` and exported it for SDK/registry/testkit usage.
- Added SDK `ChainFacade` and registry chain capability preview, keeping changes additive/backward-compatible.
- Added chain policy/spec docs first, then code, to lock finality/nonce/fee/reorg/signer rules in SSOT.
- Added dedicated EVM testkit harness and six regression tests that fail on provider failover, parsing, nonce/fee policy, tx lifecycle, logs resume, and reorg handling regressions.

## 3) Self-check results
- Allowed-path check OK
  - allowlist awk over staged files returned empty.
- Tests added/updated OK
  - chain_evm_provider_failover
  - chain_evm_balance_and_call
  - chain_evm_nonce_and_fee
  - chain_evm_tx_lifecycle
  - chain_evm_logs_and_resume
  - chain_evm_reorg_handling
- Build/Unit test command results
  - `cd ucel && cargo test -p ucel-core` => OK
  - `cd ucel && cargo test -p ucel-chain-ethereum` => OK
  - `cd ucel && cargo test -p ucel-sdk` => OK
  - `cd ucel && cargo test -p ucel-registry` => OK
  - `cd ucel && cargo test -p ucel-testkit --test chain_evm_provider_failover -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test chain_evm_balance_and_call -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test chain_evm_nonce_and_fee -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test chain_evm_tx_lifecycle -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test chain_evm_logs_and_resume -- --nocapture` => 1 passed
  - `cd ucel && cargo test -p ucel-testkit --test chain_evm_reorg_handling -- --nocapture` => 1 passed
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null` => OK.
- Secrets scan
  - `rg -n "0x[a-fA-F0-9]{64}|PRIVATE KEY|mnemonic|https://.*(alchemy|infura)" ucel/fixtures/chain_evm ucel/examples/chain_evm_* docs/verification/UCEL-CHAIN-EVM-011.md` => no real key/RPC leakage.
- docsリンク存在チェック（今回触った docs 内の "docs/" 参照のみ）
  - New chain docs do not add internal `docs/` links.

## 4) ★履歴確認の証拠
- git log/graph/merges
  - confirmed latest chain around `49f283a`, `44cd0a3`, `2d83fec`, `1f72be0` and merge train continuity.
- git show
  - `git show --stat HEAD` and `git show --stat 2d83fec` inspected recent scope and hotspot intent.
- blame
  - `git blame -w ucel/crates/ucel-chain-ethereum/src/lib.rs` showed placeholder-only implementation before this task.
  - `git blame -w` on core/sdk/registry/transport showed where additive exports/facades are safest.
- reflog/branch
  - `git reflog -n 30`, `git branch -vv` recorded branch ancestry and local branch progression.
- merge-base/conflict checks
  - `git merge-base HEAD origin/master` and origin-based diff/log checks are not runnable in this environment because `origin/master` ref is missing.

### Placeholder棚卸し
- pre-change `ucel-chain-ethereum/src/lib.rs` only implemented `Exchange::execute` with `NotSupported`, no provider, tx, receipt, logs, nonce, finality, or reorg-safe behavior.

### provider/finality/nonce/fee/reorg 設計根拠
- provider: `EvmProviderSet::call_with_failover` validates chain id and applies retry budget.
- finality: confirmation-depth mapping (`Pending/Unsafe/Safe/Finalized`) and reorg override path.
- nonce/fee: account+chain scoped reservation and EIP-1559/legacy fee estimate with fee ceiling.
- reorg/resume: cursor hash + removed-log detection, rollback replay range, duplicate log suppression.

### signer/secret redaction 設計根拠
- signer is trait-based (`EvmSigner`) with deterministic test signer for fixtures.
- `redact_signer_material` prevents raw secret/private markers in outputs.
- tests/examples/fixtures avoid real private keys and real RPC endpoints.

### 不足に対する追加実装
- because placeholder had zero runtime capabilities, this task added full read/write/log/reorg scaffolding plus tests/docs instead of extending `NotSupported` stubs.
