# Y-BUNDLE-002 Verification

## 1) Changed files
- docs/contracts/support_bundle_manifest.schema.json
- docs/specs/crosscut/support_bundle_spec.md
- docs/status/trace-index.json
- docs/verification/Y-BUNDLE-002.md
- ucel/crates/ucel-transport/Cargo.toml
- ucel/crates/ucel-transport/src/diagnostics/bundle.rs
- ucel/crates/ucel-transport/src/diagnostics/limits.rs
- ucel/crates/ucel-transport/src/diagnostics/manifest.rs
- ucel/crates/ucel-transport/src/diagnostics/mod.rs
- ucel/crates/ucel-transport/src/diagnostics/path.rs
- ucel/crates/ucel-transport/src/diagnostics/support_bundle.rs
- ucel/crates/ucel-transport/tests/support_bundle_build.rs
- ucel/crates/ucel-transport/tests/support_bundle_manifest.rs

## 2) What / Why
- YのSupport Bundleを「archive + manifest-first + hashing + limits + path safety」で実装し、外部共有可能な診断パッケージの土台を確立。
- Provider contributions（DiagnosticsRegistry）を入力にし、決定的な順序でファイルを組み立て、manifest.json に sha256/size を記録。
- 同時実行上限・総容量上限・単体上限・時間上限で self-DoS を予防。危険パスは fail-fast で拒否。

## 3) Self-check results
- Allowed-path check: OK（許可パス外の変更なし）
- Tests added/updated:
  - ucel/crates/ucel-transport/tests/support_bundle_build.rs
  - ucel/crates/ucel-transport/tests/support_bundle_manifest.rs（deny pattern評価時にmanifestのdeny list自身を除外する局所修正）
- Build/Test commands:
  - cargo test --manifest-path ucel/Cargo.toml -p ucel-transport => OK
- trace-index json.tool:
  - python -m json.tool docs/status/trace-index.json > /dev/null => OK
- Secrets scan:
  - rg -n "(api[_-]?key|secret|token|password|Authorization:|BEGIN [A-Z ]*PRIVATE KEY)" ucel/crates/ucel-transport/src/diagnostics docs || true
  - 結果: 仕様上の禁止語彙・テスト用文字列のみ検出、秘密値の新規追加なし。
- docsリンク存在チェック（今回触った docs 内の "docs/" 参照だけ。触った場合のみ）:
  - rg -n "diag_semver" docs/specs/crosscut/support_bundle_spec.md || true

## 4) History evidence (required)
- git log --oneline --decorate -n 50: HEAD a03c57f（Y-CORE-001）直後に作業。直近は C-OBS 系マージで bundle archive 実装は未完。
- git log --graph --oneline --decorate --all -n 80: support bundle の最新実装系は 7b6842a 周辺。今回対象の tar.zst+hashing+limits は新規追加余地あり。
- git log --merges --oneline -n 30: 最近のマージは observability/no-leak/safety 系。bundle archive deterministic 実装との競合は観測されず。
- git reflog -n 30: work(a03c57f) から feature/y-bundle-002 を作成。
- merge-base:
  - `git merge-base HEAD origin/master` は remote 未設定のため解決不可
  - 参照ベース: `work` ブランチ先頭 a03c57f
- blame findings:
  - docs/specs/crosscut/support_bundle_spec.md: manifest-first / secret-free / fail-closed を強く固定する方針。
  - ucel/crates/ucel-transport/src/diagnostics/support_bundle.rs: 既存はJSON payload + fixture manifestで、tar.zst生成やファイルhash列挙は未実装。
- Conclusion:
  - 既存に “manifest-first + hashing + deterministic tar.zst” が完成していないため、本タスクは重複実装ではない。
