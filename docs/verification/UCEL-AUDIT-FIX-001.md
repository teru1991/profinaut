# UCEL-AUDIT-FIX-001 Verification

## 1) Changed files
- (paste) `git diff --name-only`

```
docs/status/trace-index.json
docs/verification/UCEL-AUDIT-FIX-001.md
ucel/crates/ucel-symbol-adapter/Cargo.toml
ucel/crates/ucel-symbol-adapter/src/lib.rs
ucel/crates/ucel-symbol-adapter/src/resync.rs
ucel/crates/ucel-symbol-adapter/tests/resync_contract.rs
ucel/crates/ucel-symbol-store/Cargo.toml
ucel/crates/ucel-symbol-store/src/checkpoint.rs
ucel/crates/ucel-symbol-store/src/lib.rs
ucel/crates/ucel-symbol-store/src/replay.rs
ucel/crates/ucel-symbol-store/tests/checkpoint_replay.rs
ucel/docs/symbol-master-remaining-tasks.md
```

## 2) What / Why
- Symbol Master の未達（checkpoint/from-version, lagged→resync契約）を UCEL の後方互換な API 追記で解消。
- A〜Y 実装の基盤として「復元できる・追いつける・落ちるべき時に落ちる」をテストで固定。
- バイナリ資産は追加しない（text only）。

## 3) Self-check results
- Allowed-path check OK:
  - `git diff --name-only | awk ... | wc -l` => `0`
- Tests added/updated OK:
  - `ucel/crates/ucel-symbol-store/tests/checkpoint_replay.rs`
  - `ucel/crates/ucel-symbol-adapter/tests/resync_contract.rs`
- Build/Test commands:
  - `cargo test -p ucel-symbol-store`
  - `cargo test -p ucel-symbol-adapter`
  - `cargo test --manifest-path ucel/Cargo.toml -p ucel-symbol-store -p ucel-symbol-adapter --all-targets`
  - 結果: workspace 既存不整合（`ucel-cex-sbivc/Cargo.toml` duplicate dependency）で実行不能。
- fmt/clippy (if CI requires):
  - `cargo fmt --manifest-path ucel/Cargo.toml --all -- --check` (未実行: workspace manifest エラーのため)
  - `cargo clippy --manifest-path ucel/Cargo.toml --workspace --all-targets -- -D warnings` (未実行: workspace manifest エラーのため)
- Secrets scan (quick):
  - `git diff | rg -n "(api_key|secret|token|passphrase|PRIVATE KEY)"`（ヒット無し）
- Binary file check:
  - `git diff --name-only --diff-filter=A | rg -n "\.(png|jpg|jpeg|gif|zip|bin|exe|dll|so|dylib)$"`（ヒット無し）
- docs links check (touched docs only):
  - `rg -n "docs/" ucel/docs/symbol-master-remaining-tasks.md`

## 4) ★History evidence (required)
- outputs summary:
  - `git log --oneline --decorate -n 50`: `/tmp/log1.txt`
  - `git log --graph --oneline --decorate --all -n 80`: `/tmp/log2.txt`
  - `git log --merges --oneline -n 30`: `/tmp/log3.txt`
  - `git merge-base ...`: origin remote が未設定のため算出不可（`/tmp/merge_base.txt`）。
- Blame notes:
  - ucel-symbol-store/lib.rs: 既存は snapshot diff 主体の構造だったため、public API 互換のまま `event_log` と replay/checkpoint API を局所追記。
  - ucel-symbol-adapter/lib.rs: 既存 `SymbolSubscriber` 互換維持のため新 trait `SymbolSubscriberExtResync` を追加し opt-in 化。
- Gaps found & fixed:
  - checkpoint/from-version の欠落 → `checkpoint.rs`/`replay.rs` + `export_since`/`import_events` + テストで固定。
  - lagged→resync 契約欠落 → `resync.rs` + 契約テストで固定。
- revert痕跡/重複実装判定:
  - 直近履歴/grep/blame上、`checkpoint` / `from-version` / `SymbolSubscriber` resync 拡張の既存実装は確認されず、重複回避方針で新規 module 追加。
