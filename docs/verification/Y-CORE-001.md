# Y-CORE-001 Verification

## 1) Changed files
- docs/contracts/diag_semver.schema.json
- docs/contracts/diagnostics_provider.schema.json
- docs/specs/crosscut/support_bundle_spec.md
- docs/specs/system/Y_Supportability_Diagnostics_Governance_Spec_v1.0.md
- docs/status/trace-index.json
- docs/verification/Y-CORE-001.md
- ucel/Cargo.toml
- ucel/crates/ucel-diagnostics-core/Cargo.toml
- ucel/crates/ucel-diagnostics-core/src/diag_semver.rs
- ucel/crates/ucel-diagnostics-core/src/lib.rs
- ucel/crates/ucel-diagnostics-core/src/provider.rs
- ucel/crates/ucel-diagnostics-core/src/registry.rs
- ucel/crates/ucel-diagnostics-core/tests/registry_determinism.rs

## 2) What / Why
- Y spec の TODO（diag_semver contract 固定、DiagnosticsProvider contract 参照）を、docs/contracts に契約として追加し、spec/crosscut が参照する形で SSOT を完成させた。
- 実装側は UCEL workspace に `ucel-diagnostics-core` を新設し、DiagnosticsProvider / Contribution / Registry を提供。
- Registry で provider_id 一意性、path安全（no absolute/no '..'）、決定的ソートを強制し、Task2以降の bundle 化/manifest 化の安全な土台を確立。

## 3) Self-check results
- Allowed-path check: OK (許可パス外の変更なし)
- Tests added/updated:
  - ucel/crates/ucel-diagnostics-core/tests/registry_determinism.rs
- Build/Test commands:
  - cargo test --manifest-path ucel/Cargo.toml -p ucel-diagnostics-core => OK
- trace-index json.tool:
  - python -m json.tool docs/status/trace-index.json > /dev/null => OK
- Secrets scan:
  - rg -n "(api[_-]?key|secret|token|password|Authorization:|BEGIN [A-Z ]*PRIVATE KEY)" docs ucel/crates/ucel-diagnostics-core || true
  - 結果: 既知の仕様文言・禁止語彙の記載のみで、実秘密の追加なし
- docsリンク存在チェック（今回触った docs 内の "docs/" 参照だけ）:
  - rg -n "docs/contracts/diag_semver.schema.json" docs/specs || true
  - rg -n "docs/contracts/diagnostics_provider.schema.json" docs/specs || true

## 4) History evidence (required)
- git log --oneline --decorate -n 50: HEAD は 29b3c4f（PR #448 merge）。直近は C-OBS 系の段階的マージで、Y コア基盤の追加と競合する実装は未確認。
- git log --graph --oneline --decorate --all -n 80: #444〜#448 が観測性領域を段階拡張。Y 専用 contract/core crate は未導入。
- git log --merges --oneline -n 30: 連続 merge は C-OBS/B-STEP/E-PLAN 系で、今回追加する diag_semver/provider contract とは責務が分離。
- git reflog -n 30: 本作業は `work`(29b3c4f) から `feature/y-core-001` を新規作成。
- merge-base:
  - `git merge-base HEAD origin/master` は remote 未設定のため解決不可
  - `git merge-base HEAD work` = 29b3c4f12137ec14af36f35561f094ad634b648b
- branch pointers:
  - `git branch -vv`: `feature/y-core-001` と `work` は同一点 29b3c4f を指す
- Y/support-bundle 関連コミット:
  - Y spec: c3b5d14
  - crosscut support bundle spec: ecb02ed, de4b355, d072af7, d4ff147
  - support bundle manifest contract: d9d008f
  - transport support_bundle.rs: 7b6842a, bf5f8ee, d632ed9
- blame findings:
  - docs/specs/crosscut/support_bundle_spec.md: contract-first（manifest必須）・invariant固定の運用を維持する方針。
  - ucel/crates/ucel-transport/src/diagnostics/support_bundle.rs: diagnostics payloadは既存 manifest fixture を組み込み、bundle生成は構造化JSON主導。
- Conclusion:
  - 既存に diag_semver/DiagnosticsProvider の code+contract は無く、本タスクは重複実装ではない。
