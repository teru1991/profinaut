# UCEL-SSOT-V2-ONLY-001 Verification

## 1) Changed files
- (paste) `git diff --name-only`

```text
ucel/crates/ucel-testkit/src/coverage.rs
ucel/crates/ucel-testkit/src/lib.rs
ucel/crates/ucel-testkit/src/ssot_gate.rs
ucel/crates/ucel-testkit/src/ssot_integrity_gate.rs
ucel/crates/ucel-testkit/src/support_bundle_manifest.rs
ucel/crates/ucel-testkit/src/ws_coverage_gate.rs
ucel/crates/ucel-testkit/tests/coverage_h_100_gate.rs
ucel/crates/ucel-testkit/tests/golden_ws.rs
ucel/crates/ucel-testkit/tests/ssot_gate_test.rs
ucel/crates/ucel-testkit/tests/ssot_integrity_gate_test.rs
ucel/crates/ucel-testkit/tests/strict_golden_gate.rs
ucel/crates/ucel-testkit/tests/strict_golden_manifest_gate.rs
ucel/docs/policies/coverage_policy.md

```

## 2) What / Why (3-7 lines)
- CI合否の根拠が coverage v1 (yaml) に依存しており、policyの“v2正本”と矛盾していたため、ゲートをcoverage_v2一本化した。
- v1は削除せず legacy として残し、strict/SSOTは `ucel/coverage/coverage_v2/strict_venues.json` を唯一正本に固定。
- `ucel-testkit` の SSOT/WS/golden 系テストを strict_venues.json + coverage_v2 参照へ置換した。

## 3) Self-check results
- Allowed-path check OK:
  - `git diff --name-only | awk ...`
```text

```
- Binary file check:
```text

```
- Secrets scan:
```text

```
- v1 dependency scan:
```text

```

## 4) ★History evidence (required)
- `git log --oneline --decorate -n 50`
```text
46edb92 (HEAD -> feature/ucel-ssot-v2-only-001, work) Merge pull request #432 from teru1991/codex/harden-private-connections-in-ucel-70bars
a1f5a4a Merge branch 'master' into codex/harden-private-connections-in-ucel-70bars
d04e343 ucel: complete ws discoverability + gate coverage_v2 alignment; mark coverage v1 docs as legacy
1f6b262 更新
d51650b Merge pull request #431 from teru1991/codex/harden-private-connections-in-ucel-r8jsjz
91f2f65 Merge branch 'master' into codex/harden-private-connections-in-ucel-r8jsjz
16e31b4 symbol-master: implement resync snapshot->store->checkpoint; gate domestic private request shapes via mock
09595fa Merge pull request #430 from teru1991/codex/harden-private-connections-in-ucel
4cc7c3e ucel: gate domestic private signing/time/nonce + standard retry/idempotency; deprecate coverage v1 usage
aa6ee1e Merge pull request #429 from teru1991/codex/audit-ucel-implementation-for-compatibility-bxeh6x
692615b Merge branch 'master' into codex/audit-ucel-implementation-for-compatibility-bxeh6x
5a39164 ucel: gate coverage_v2 + fill ws discoverability + sbivc policy exception
9a8be24 Merge pull request #428 from teru1991/codex/audit-ucel-implementation-for-compatibility-h4kjc0
6e8a4c4 Merge branch 'master' into codex/audit-ucel-implementation-for-compatibility-h4kjc0
018ff0a 更新
0ce9c9d symbol-master: make service runnable with healthz/readyz and resync coordinator
e2cc172 Merge pull request #427 from teru1991/codex/audit-ucel-implementation-for-compatibility
312bcc2 ucel: add symbol store checkpoint/replay + resync contract
88fa14d Merge pull request #426 from teru1991/codex/enhance-ucel-testing-and-supportability-checks
7b6842a UCEL-TY: add golden/support-bundle manifests and gates for 100% proof
9035aa6 Merge pull request #425 from teru1991/codex/ensure-100%-coverage-for-market-data
d4195e3 UCEL-H: make deribit/sbivc market-data coverage strict and 100%-gated
76c473b 更新
57399f9 Merge pull request #424 from teru1991/claude/ucel-live-execution-YLtqr
6ace9ab chore: apply cargo fmt formatting to pre-existing files
424669c UCEL-I: add async execution client + file audit, implement bittrade execution connector
fb9bcbe Merge pull request #423 from teru1991/claude/strict-ucel-coverage-dwcoj
2219d58 UCEL-H: enforce strict=true coverage and require golden for strict venues
77607b6 Merge pull request #422 from teru1991/claude/check-repo-access-Zq5WI
123a218 UCEL-I: add ucel-sdk execution public surface (UCEL-I-EXEC-001)
1e369f8 UCEL-I: add ucel-sdk execution public surface (client/idempotency/audit/gate)
295e35c Merge pull request #421 from teru1991/codex/implement-observability-and-support-bundle
14fc4d4 feat(ucel): standardize observability and add support bundle v1
7768947 Merge pull request #420 from teru1991/codex/enhance-transport-resilience-to-100%
018c91b feat(ucel): add transport resilience spec and chaos test harness
0093ff4 Merge pull request #419 from teru1991/codex/add-crash-free-fuzz-tests-and-seed-corpus
4739bd7 feat(testkit): add deterministic crash-free fuzz tests and small seed corpus
28eb5cc Merge pull request #418 from teru1991/codex/implement-bithumb-public-rest/ws-s8pjik
6380fb1 Merge branch 'master' into codex/implement-bithumb-public-rest/ws-s8pjik
527f804 test(ucel): add strict-venue ws golden harness and fixtures
18ccc53 Merge pull request #417 from teru1991/codex/implement-bithumb-public-rest/ws
654b7a5 feat(ucel): implement bithumb public adapters and strict coverage
14e4c21 Merge pull request #416 from teru1991/codex/migrate-ssot-coverage-to-v1-schema
ee40194 chore(ucel): migrate all coverage files to ssot schema v1
d43d456 Merge pull request #415 from teru1991/codex/extend-ssot-for-ucel-compliance
8e005d4 feat(ucel): extend ssot gate and document v1 contracts
d664955 Merge pull request #414 from teru1991/codex/add-observability-and-security-foundations-5irz6b
fc46a38 Merge branch 'master' into codex/add-observability-and-security-foundations-5irz6b
ecb02ed C-B: enforce json limits + endpoint allowlist + max frame across registry/cex/adapters
fe0b102 Merge pull request #413 from teru1991/codex/add-observability-and-security-foundations-jayzs0

```
- `git log --graph --oneline --decorate --all -n 80`
```text
*   46edb92 (HEAD -> feature/ucel-ssot-v2-only-001, work) Merge pull request #432 from teru1991/codex/harden-private-connections-in-ucel-70bars
|\  
| *   a1f5a4a Merge branch 'master' into codex/harden-private-connections-in-ucel-70bars
| |\  
| |/  
|/|   
* | 1f6b262 更新
* |   d51650b Merge pull request #431 from teru1991/codex/harden-private-connections-in-ucel-r8jsjz
|\ \  
| * \   91f2f65 Merge branch 'master' into codex/harden-private-connections-in-ucel-r8jsjz
| |\ \  
| |/ /  
|/| |   
* | |   09595fa Merge pull request #430 from teru1991/codex/harden-private-connections-in-ucel
|\ \ \  
| * | | 4cc7c3e ucel: gate domestic private signing/time/nonce + standard retry/idempotency; deprecate coverage v1 usage
|/ / /  
| * / 16e31b4 symbol-master: implement resync snapshot->store->checkpoint; gate domestic private request shapes via mock
|/ /  
| * d04e343 ucel: complete ws discoverability + gate coverage_v2 alignment; mark coverage v1 docs as legacy
|/  
*   aa6ee1e Merge pull request #429 from teru1991/codex/audit-ucel-implementation-for-compatibility-bxeh6x
|\  
| *   692615b Merge branch 'master' into codex/audit-ucel-implementation-for-compatibility-bxeh6x
| |\  
| |/  
|/|   
* |   9a8be24 Merge pull request #428 from teru1991/codex/audit-ucel-implementation-for-compatibility-h4kjc0
|\ \  
| * \   6e8a4c4 Merge branch 'master' into codex/audit-ucel-implementation-for-compatibility-h4kjc0
| |\ \  
| |/ /  
|/| |   
* | | 018ff0a 更新
* | |   e2cc172 Merge pull request #427 from teru1991/codex/audit-ucel-implementation-for-compatibility
|\ \ \  
| * | | 312bcc2 ucel: add symbol store checkpoint/replay + resync contract
|/ / /  
| * / 0ce9c9d symbol-master: make service runnable with healthz/readyz and resync coordinator
|/ /  
| * 5a39164 ucel: gate coverage_v2 + fill ws discoverability + sbivc policy exception
|/  
*   88fa14d Merge pull request #426 from teru1991/codex/enhance-ucel-testing-and-supportability-checks
|\  
| * 7b6842a UCEL-TY: add golden/support-bundle manifests and gates for 100% proof
|/  
*   9035aa6 Merge pull request #425 from teru1991/codex/ensure-100%-coverage-for-market-data
|\  
| * d4195e3 UCEL-H: make deribit/sbivc market-data coverage strict and 100%-gated
* | 76c473b 更新
|/  
*   57399f9 Merge pull request #424 from teru1991/claude/ucel-live-execution-YLtqr
|\  
| * 6ace9ab chore: apply cargo fmt formatting to pre-existing files
| * 424669c UCEL-I: add async execution client + file audit, implement bittrade execution connector
|/  
*   fb9bcbe Merge pull request #423 from teru1991/claude/strict-ucel-coverage-dwcoj
|\  
| * 2219d58 UCEL-H: enforce strict=true coverage and require golden for strict venues
|/  
*   77607b6 Merge pull request #422 from teru1991/claude/check-repo-access-Zq5WI
|\  
| * 123a218 UCEL-I: add ucel-sdk execution public surface (UCEL-I-EXEC-001)
|/| 
| * 1e369f8 UCEL-I: add ucel-sdk execution public surface (client/idempotency/audit/gate)
|/  
*   295e35c Merge pull request #421 from teru1991/codex/implement-observability-and-support-bundle
|\  
| * 14fc4d4 feat(ucel): standardize observability and add support bundle v1
|/  
*   7768947 Merge pull request #420 from teru1991/codex/enhance-transport-resilience-to-100%
|\  
| * 018c91b feat(ucel): add transport resilience spec and chaos test harness
|/  
*   0093ff4 Merge pull request #419 from teru1991/codex/add-crash-free-fuzz-tests-and-seed-corpus
|\  
| * 4739bd7 feat(testkit): add deterministic crash-free fuzz tests and small seed corpus
|/  
*   28eb5cc Merge pull request #418 from teru1991/codex/implement-bithumb-public-rest/ws-s8pjik
|\  
| *   6380fb1 Merge branch 'master' into codex/implement-bithumb-public-rest/ws-s8pjik
| |\  
| |/  
|/|   
* |   18ccc53 Merge pull request #417 from teru1991/codex/implement-bithumb-public-rest/ws
|\ \  
| * | 654b7a5 feat(ucel): implement bithumb public adapters and strict coverage
|/ /  
| * 527f804 test(ucel): add strict-venue ws golden harness and fixtures
|/  
*   14e4c21 Merge pull request #416 from teru1991/codex/migrate-ssot-coverage-to-v1-schema
|\  
| * ee40194 chore(ucel): migrate all coverage files to ssot schema v1
|/  
*   d43d456 Merge pull request #415 from teru1991/codex/extend-ssot-for-ucel-compliance
|\  
| * 8e005d4 feat(ucel): extend ssot gate and document v1 contracts
|/  
*   d664955 Merge pull request #414 from teru1991/codex/add-observability-and-security-foundations-5irz6b
|\  
| *   fc46a38 Merge branch 'master' into codex/add-observability-and-security-foundations-5irz6b
| |\  
| |/  
|/|   
* |   fe0b102 Merge pull request #413 from teru1991/codex/add-observability-and-security-foundations-jayzs0
|\ \  
| * | bf5f8ee C-B: wire transport observability+security into ws/http paths
* | |   e075c1e Merge pull request #412 from teru1991/codex/add-observability-and-security-foundations
|\ \ \  
| |/ /  
|/| |   
| * | 2d8843f C-B: add transport observability+security foundations (catalog/redaction/allowlist/limits)
|/ /  
| * ecb02ed C-B: enforce json limits + endpoint allowlist + max frame across registry/cex/adapters
|/  
*   247cf67 Merge pull request #411 from teru1991/codex/implement-golden-fixture-for-normalization-test
|\  
| * dd739f3 test(ucel-testkit): add bybit golden ws normalization proof
|/  
*   0d2069c Merge pull request #410 from teru1991/codex/extend-coverage-ssot-with-backward-compatibility-rztmcp
|\  
| *   976d429 Merge branch 'master' into codex/extend-coverage-ssot-with-backward-compatibility-rztmcp
| |\  
| |/  
|/|   
* |   495888e Merge pull request #409 from teru1991/codex/extend-coverage-ssot-with-backward-compatibility-cnhapq
|\ \  
| * \   a48e4ab Merge branch 'master' into codex/extend-coverage-ssot-with-backward-compatibility-cnhapq
| |\ \  
| |/ /  
|/| |   
* | |   e8cd694 Merge pull request #408 from teru1991/codex/extend-coverage-ssot-with-backward-compatibility
|\ \ \  
| * | | 75ec470 feat(ucel): extend coverage v1 with explicit support metadata
|/ / /  
| * / fcdde1b feat(testkit): add ssot integrity gate v2 as additive API
|/ /  
| * b5f912f chore(ssot): fill v2 gate contracts and enforce repo-level zero failures
|/  
*   bf6c03d Merge pull request #407 from teru1991/codex/implement-marketmeta-for-public-exchanges-p7oddt
|\  
| *   c368a2d Merge branch 'master' into codex/implement-marketmeta-for-public-exchanges-p7oddt
| |\  
| |/  
|/|   
* |   f22fbc7 Merge pull request #406 from teru1991/codex/implement-marketmeta-for-public-exchanges-f8iqn4
|\ \  
| * \   2310b7a Merge branch 'master' into codex/implement-marketmeta-for-public-exchanges-f8iqn4
| |\ \  
| |/ /  
|/| |   
* | |   ce3bc9c Merge pull request #405 from teru1991/codex/implement-marketmeta-for-public-exchanges
|\ \ \  
| * | | 30e88b3 feat(ucel): add public exchange snapshot->marketmeta adapters
|/ / /  
| * / c84d376 test(ucel): add fixture regressions for exchange marketmeta mappings
|/ /  
| * 312550e feat(ucel): add market meta catalog fallback for JP exchanges
|/  
*   c359fa4 Merge pull request #404 from teru1991/codex/extend-symbols.rs-for-snapshot-support
|\  
| * d748c55 Add symbol snapshot APIs for market meta capable exchanges
|/  
*   4038c42 Merge pull request #403 from teru1991/codex/eliminate-all-rust-warnings-and-errors
|\  
| * 335bcfc Fix semver-checks baseline root path in CI
| * 25dedce Use derived Default for TickStepRounding to satisfy clippy
| * f74dbd3 Resolve remaining clippy/rustdoc blockers for rust quality gate
| * efb302a Add rust quality gates and fix workspace/lint blockers
|/  
*   5d7d0ac Merge pull request #402 from teru1991/codex/add-marketmeta-as-single-source-of-truth
|\  
| * f16368b feat(ucel): add MarketMeta SSOT derivation and connector APIs
|/  

```
- `git log --merges --oneline -n 30`
```text
46edb92 Merge pull request #432 from teru1991/codex/harden-private-connections-in-ucel-70bars
a1f5a4a Merge branch 'master' into codex/harden-private-connections-in-ucel-70bars
d51650b Merge pull request #431 from teru1991/codex/harden-private-connections-in-ucel-r8jsjz
91f2f65 Merge branch 'master' into codex/harden-private-connections-in-ucel-r8jsjz
09595fa Merge pull request #430 from teru1991/codex/harden-private-connections-in-ucel
aa6ee1e Merge pull request #429 from teru1991/codex/audit-ucel-implementation-for-compatibility-bxeh6x
692615b Merge branch 'master' into codex/audit-ucel-implementation-for-compatibility-bxeh6x
9a8be24 Merge pull request #428 from teru1991/codex/audit-ucel-implementation-for-compatibility-h4kjc0
6e8a4c4 Merge branch 'master' into codex/audit-ucel-implementation-for-compatibility-h4kjc0
e2cc172 Merge pull request #427 from teru1991/codex/audit-ucel-implementation-for-compatibility
88fa14d Merge pull request #426 from teru1991/codex/enhance-ucel-testing-and-supportability-checks
9035aa6 Merge pull request #425 from teru1991/codex/ensure-100%-coverage-for-market-data
57399f9 Merge pull request #424 from teru1991/claude/ucel-live-execution-YLtqr
fb9bcbe Merge pull request #423 from teru1991/claude/strict-ucel-coverage-dwcoj
77607b6 Merge pull request #422 from teru1991/claude/check-repo-access-Zq5WI
123a218 UCEL-I: add ucel-sdk execution public surface (UCEL-I-EXEC-001)
295e35c Merge pull request #421 from teru1991/codex/implement-observability-and-support-bundle
7768947 Merge pull request #420 from teru1991/codex/enhance-transport-resilience-to-100%
0093ff4 Merge pull request #419 from teru1991/codex/add-crash-free-fuzz-tests-and-seed-corpus
28eb5cc Merge pull request #418 from teru1991/codex/implement-bithumb-public-rest/ws-s8pjik
6380fb1 Merge branch 'master' into codex/implement-bithumb-public-rest/ws-s8pjik
18ccc53 Merge pull request #417 from teru1991/codex/implement-bithumb-public-rest/ws
14e4c21 Merge pull request #416 from teru1991/codex/migrate-ssot-coverage-to-v1-schema
d43d456 Merge pull request #415 from teru1991/codex/extend-ssot-for-ucel-compliance
d664955 Merge pull request #414 from teru1991/codex/add-observability-and-security-foundations-5irz6b
fc46a38 Merge branch 'master' into codex/add-observability-and-security-foundations-5irz6b
fe0b102 Merge pull request #413 from teru1991/codex/add-observability-and-security-foundations-jayzs0
e075c1e Merge pull request #412 from teru1991/codex/add-observability-and-security-foundations
247cf67 Merge pull request #411 from teru1991/codex/implement-golden-fixture-for-normalization-test
0d2069c Merge pull request #410 from teru1991/codex/extend-coverage-ssot-with-backward-compatibility-rztmcp

```
- `git show HEAD`
```text
commit 46edb928c7895d87650ed502276a2d3287367d05
Merge: 1f6b262 a1f5a4a
Author: teru1991 <48640151+teru1991@users.noreply.github.com>
Date:   Tue Mar 3 19:24:32 2026 +0900

    Merge pull request #432 from teru1991/codex/harden-private-connections-in-ucel-70bars
    
    Add private signing goldens, symbol-master resync/store-bridge, execution-core utilities, and coverage_v2 discoverability gates


```
- `git reflog -n 30`
```text
46edb92 HEAD@{0}: checkout: moving from work to feature/ucel-ssot-v2-only-001
46edb92 HEAD@{1}: checkout: moving from old_work-1772535238 to work
7768947 HEAD@{2}: Branch: renamed refs/heads/work to refs/heads/old_work-1772535238
7768947 HEAD@{4}: checkout: moving from master to work

```
- `git merge-base HEAD origin/<default>`
```text
remote origin unavailable in this environment

```
- “v1依存の根”抽出:
```text
ucel/crates/ucel-testkit/tests/ssot_integrity_gate_test.rs:24:fn write_strict_venues(root: &Path, venues: &[&str]) {
ucel/crates/ucel-testkit/tests/ssot_integrity_gate_test.rs:30:        &root.join("ucel/coverage/coverage_v2/strict_venues.json"),
ucel/crates/ucel-testkit/tests/ssot_integrity_gate_test.rs:47:    write_strict_venues(&root, &["foo"]);
ucel/crates/ucel-testkit/tests/ssot_integrity_gate_test.rs:76:    write_strict_venues(&root, &["foo"]);
ucel/crates/ucel-testkit/tests/ssot_integrity_gate_test.rs:99:    write_strict_venues(&root, &[]);
ucel/crates/ucel-testkit/tests/coverage_h_100_gate.rs:6:    let strict = ucel_testkit::coverage_v2::load_strict_venues(&root).expect("load strict venues");
ucel/crates/ucel-testkit/tests/golden_ws.rs:4:fn strict_venues(repo_root: &std::path::Path) -> Vec<String> {
ucel/crates/ucel-testkit/tests/golden_ws.rs:5:    let mut venues = ucel_testkit::coverage_v2::load_strict_venues(repo_root)
ucel/crates/ucel-testkit/tests/golden_ws.rs:6:        .expect("load strict_venues.json")
ucel/crates/ucel-testkit/tests/golden_ws.rs:13:fn golden_ws_all_strict_venues_are_verified() {
ucel/crates/ucel-testkit/tests/golden_ws.rs:15:    let venues = strict_venues(&repo_root);
ucel/crates/ucel-testkit/tests/no_v1_coverage_dependency_gate.rs:20:    let strict_path = repo_root().join("ucel/coverage/coverage_v2/strict_venues.json");
ucel/crates/ucel-testkit/tests/no_v1_coverage_dependency_gate.rs:21:    let strict_raw = std::fs::read_to_string(strict_path).expect("strict_venues.json must exist");
ucel/crates/ucel-testkit/tests/ssot_gate_test.rs:39:        root.join("ucel/coverage/coverage_v2/strict_venues.json"),
ucel/crates/ucel-testkit/tests/ssot_gate_test.rs:42:    .expect("write strict_venues.json");
ucel/crates/ucel-testkit/tests/strict_golden_gate.rs:11:fn strict_venues_v2() -> Vec<String> {
ucel/crates/ucel-testkit/tests/strict_golden_gate.rs:13:        ucel_testkit::coverage_v2::load_strict_venues(&repo_root()).expect("strict venues");
ucel/crates/ucel-testkit/tests/strict_golden_gate.rs:36:fn strict_venues_must_have_golden_fixtures() {
ucel/crates/ucel-testkit/tests/strict_golden_gate.rs:37:    let venues = strict_venues_v2();
ucel/crates/ucel-testkit/tests/strict_golden_gate.rs:40:        "no strict venues found in coverage_v2/strict_venues.json"
ucel/crates/ucel-testkit/tests/strict_golden_gate.rs:59:fn strict_venues_v2_list_is_non_empty() {
ucel/crates/ucel-testkit/tests/strict_golden_gate.rs:60:    let venues = strict_venues_v2();
ucel/crates/ucel-testkit/tests/strict_golden_manifest_gate.rs:11:fn strict_venues_from_coverage_v2(repo_root: &std::path::Path) -> Vec<String> {
ucel/crates/ucel-testkit/tests/strict_golden_manifest_gate.rs:12:    let mut venues = ucel_testkit::coverage_v2::load_strict_venues(repo_root)
ucel/crates/ucel-testkit/tests/strict_golden_manifest_gate.rs:13:        .expect("load strict_venues.json")
ucel/crates/ucel-testkit/tests/strict_golden_manifest_gate.rs:27:fn strict_venues_must_have_required_golden_files_in_manifest() {
ucel/crates/ucel-testkit/tests/strict_golden_manifest_gate.rs:38:    let venues = strict_venues_from_coverage_v2(&root.join(".."));
ucel/crates/ucel-testkit/tests/strict_golden_manifest_gate.rs:41:        "no strict venues found in strict_venues.json"
ucel/crates/ucel-testkit/src/ssot_integrity_gate.rs:219:    let strict = crate::coverage_v2::load_strict_venues(repo_root)
ucel/crates/ucel-testkit/src/ssot_integrity_gate.rs:220:        .map_err(|e| format!("load strict_venues.json: {e}"))?;
ucel/crates/ucel-testkit/src/coverage.rs:6:pub fn public_crypto_ws_ops_from_coverage(
ucel/crates/ucel-testkit/src/coverage_v2.rs:67:                    && p.file_name().and_then(|n| n.to_str()) != Some("strict_venues.json")
ucel/crates/ucel-testkit/src/coverage_v2.rs:89:                && p.file_name().and_then(|n| n.to_str()) != Some("strict_venues.json")
ucel/crates/ucel-testkit/src/coverage_v2.rs:110:pub fn load_strict_venues(repo_root: &Path) -> Result<StrictVenues, CoverageV2Error> {
ucel/crates/ucel-testkit/src/coverage_v2.rs:112:    let path = root.join("strict_venues.json");
ucel/crates/ucel-testkit/src/ws_coverage_gate.rs:32:    let strict = crate::coverage_v2::load_strict_venues(repo_root).map_err(|e| e.to_string())?;
ucel/crates/ucel-testkit/src/ssot_gate.rs:20:    let strict = crate::coverage_v2::load_strict_venues(repo_root).map_err(|e| e.to_string())?;

```
```text
scripts/ucel/set_strict_true_market_data.py:3:Patch ucel/coverage/*.yaml: change `strict: false` -> `strict: true` (text replacement, minimum diff).
docs/specs/ucel/ssot_integrity_gate_v2.md:4:> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
docs/specs/ucel/ssot_integrity_gate_v2.md:16:- Coverage SSOT (v1 legacy + v2 future): `ucel/coverage/*.yaml` (and `ucel/coverage_v2/*.yaml`)
docs/specs/ucel/coverage_strict_policy_v1.md:4:> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
docs/specs/ucel/coverage_strict_policy_v1.md:40:| Market Data (H) | `ucel/coverage/*.yaml` (v1 schema) | **This policy — all venues strict=true** |
docs/specs/ucel/coverage_strict_policy_v1.md:51:- Reads all `ucel/coverage/*.yaml` files.
docs/specs/ucel/coverage_strict_policy_v1.md:73:1. Add entry to `ucel/coverage/<venue>.yaml` with `strict: false` initially.
docs/specs/ucel/ty_100_definition_spec_v1.md:4:> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
docs/specs/ucel/ty_100_definition_spec_v1.md:14:1. Every `strict: true` venue in `ucel/coverage/*.yaml` has required WebSocket golden files:
docs/specs/ucel/market_data_coverage_scope_policy_v1.md:4:> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
docs/specs/ucel/market_data_coverage_scope_policy_v1.md:10:## What coverage/<venue>.yaml represents
docs/specs/ucel/market_data_coverage_scope_policy_v1.md:11:- `ucel/coverage/<venue>.yaml` は “その venue の Market Data（H）として UCEL がサポートする API/Channel の集合” を表す。
docs/specs/ucel/ssot_coverage_schema_v1.md:4:> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
docs/specs/ucel/ssot_coverage_schema_v1.md:8:This schema standardizes `ucel/coverage/<venue>.yaml` so CI can mechanically prove coverage progress and detect gaps before merge.
docs/specs/ucel/ssot_gate_spec_v1.md:4:> This document references coverage v1 (`ucel/coverage/*.yaml`) and is **NOT USED** for CI gating.
docs/specs/ucel/ssot_gate_spec_v1.md:8:1. Catalog venue must have corresponding `ucel/coverage/<venue>.yaml`.
ucel/docs/policies/coverage_policy.md:5:- Legacy `ucel/coverage/*.yaml` (v1) is informational only unless explicitly stated otherwise.
ucel/docs/policies/coverage_policy.md:42:- `ucel/coverage/*.yaml` (v1) is legacy/informational and MUST NOT be used for CI gating.

```
