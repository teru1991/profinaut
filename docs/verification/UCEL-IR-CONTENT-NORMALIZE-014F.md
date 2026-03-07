# UCEL-IR-CONTENT-NORMALIZE-014F Verification

## 1) Changed files
- See `git diff --name-only` / `git status --short` output for this task scope.

## 2) What/Why
- Added canonical IR normalization model fields in `ucel-core` for deterministic normalized content.
- Implemented `ucel-ir::normalize` pipeline (detect/charset/format handlers/zip safety/errors/content assembly).
- Added SDK and registry helper surfaces for normalization and reason-code/format previews.
- Added text-only fixtures and runtime PDF/ZIP builders in testkit.
- Added normalization docs/specs and drift/compat/safety tests.

## 3) Self-check results
- Allowed-path check OK (task changes are in allowlist; pre-existing unrelated dirty file `services/marketdata-rs/Cargo.lock` excluded from staging).
- Tests added/updated OK:
  - ir_normalize_format_detection
  - ir_normalize_sections_tables
  - ir_normalize_zip_and_attachments
  - ir_normalize_xbrl_xml_html_pdf
  - ir_normalize_error_policy
  - ir_normalize_compatibility
  - ir_normalize_docs_drift
- Build/Unit test command results:
  - `cd ucel && cargo test -p ucel-core` => PASS
  - `cd ucel && cargo test -p ucel-ir` => PASS
  - `cd ucel && cargo test -p ucel-registry` => PASS
  - `cd ucel && cargo test -p ucel-sdk` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ir_normalize_format_detection -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ir_normalize_sections_tables -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ir_normalize_zip_and_attachments -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ir_normalize_xbrl_xml_html_pdf -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ir_normalize_error_policy -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ir_normalize_compatibility -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit --test ir_normalize_docs_drift -- --nocapture` => PASS
- trace-index json.tool OK (`python -m json.tool docs/status/trace-index.json`)
- Secrets scan OK (`rg -n "(AKIA|SECRET|PRIVATE KEY|token)" docs/specs/ucel ucel/docs/ir ucel/crates/ucel-ir/src/normalize ucel/fixtures/ir_normalize` produced no credential-like additions)
- docsリンク存在チェック OK (new docs have no broken `docs/` links)
- binary-free fixture check OK (no `.pdf/.zip/.xbrl/.tar` binary additions; PDF/ZIP are text sources/specs)

## 4) 履歴確認の証拠
- 実行コマンド:
  - `git log --oneline --decorate -n 60`
  - `git log --graph --oneline --decorate --all -n 100`
  - `git show --stat -n 1`
  - `git blame -w ucel/crates/ucel-core/src/ir.rs`
  - `git blame -w ucel/crates/ucel-ir/src/artifact.rs`
  - `git blame -w ucel/crates/ucel-ir/src/document.rs`
  - `git blame -w ucel/crates/ucel-ir/src/fetch.rs`
  - `git blame -w ucel/crates/ucel-ir/src/jp_official/mod.rs`
  - `git blame -w ucel/crates/ucel-ir/src/us_official/mod.rs`
  - `git blame -w ucel/crates/ucel-ir/src/issuer_sites/mod.rs`
  - `git blame -w ucel/docs/ir/ir_canonical_model.md`
  - `git reflog -n 30`
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
- 環境注記:
  - This clone has no `origin/master`, so `merge-base HEAD origin/master` and origin-diff checks are not available.
- 設計根拠:
  - Existing artifact surface already includes content_type/checksum/size/encoding in `IrArtifactDescriptor`; normalization extended from that metadata.
  - Added deterministic schema/versioned normalized model and explicit reason-code errors to prevent silent fallback.
  - ZIP safety caps and traversal/nested-archive guards implemented as fail-fast defaults.
  - PDF/XBRL/charset handling intentionally explicit-error on malformed/unsupported paths.
  - Binary-free fixture policy enforced via `.pdf.txt` and `.zip_spec.json` runtime-bytes generation.
