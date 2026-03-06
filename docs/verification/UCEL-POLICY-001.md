# Verification: UCEL-POLICY-001

## 1) Changed files (`git diff --name-only`)
```bash
docs/specs/ucel/venue_access_policy_v1.md
docs/status/trace-index.json
docs/verification/UCEL-POLICY-001.md
ucel/coverage/coverage_v2/jurisdictions/jp_resident_access.json
ucel/crates/ucel-core/src/lib.rs
ucel/crates/ucel-core/src/policy.rs
ucel/crates/ucel-registry/src/hub/mod.rs
ucel/crates/ucel-registry/src/hub/rest.rs
ucel/crates/ucel-registry/src/hub/ws.rs
ucel/crates/ucel-registry/src/invoker/mod.rs
ucel/crates/ucel-registry/src/lib.rs
ucel/crates/ucel-registry/src/policy.rs
ucel/crates/ucel-testkit/src/coverage_v2.rs
ucel/crates/ucel-testkit/tests/jp_resident_policy_gate.rs
ucel/docs/policies/coverage_policy.md
ucel/docs/policies/venue_access_policy.md
```

## 2) What / Why
- JP resident 向け venue access policy を docs/spec と machine-readable JSON の両方で追加し、`public_only` default を固定した。
- `ucel-core` に `ResidencyClass / VenueAccessScope / AccessSurface / VenueAccessPolicy` の domain model を追加した。
- `ucel-registry` に policy loader/enforcer を追加し、Hub/Invoker の REST/WS 導線で送信前 fail-fast gate を有効化した。
- `default_capabilities` を後方互換で拡張し、`venue_access` 情報を capability として公開できるようにした。
- `ucel-testkit` に JP policy loader helper と gate test を追加し、coverage policy と policy JSON の整合を CI で落とせるようにした。

## 3) Self-check results
- Allowed-path check OK
  - 実行: `git diff --name-only | awk '...allowlist...'`
  - 結果: 出力なし（allowlist外変更なし）
- Tests added/updated OK
  - 追加/更新テスト:
    - `ucel/crates/ucel-core/src/policy.rs` unit tests
    - `ucel/crates/ucel-registry/src/policy.rs` unit tests
    - `ucel/crates/ucel-testkit/tests/jp_resident_policy_gate.rs`
- Build/Unit test command results
  - `cd ucel && cargo test -p ucel-core` => PASS
  - `cd ucel && cargo test -p ucel-registry` => FAIL（既知の既存失敗: `strict_coverage_registry_builds` が `strict venue bithumb missing symbol fixture`）
  - `cd ucel && cargo test -p ucel-testkit --test jp_resident_policy_gate -- --nocapture` => PASS
  - `cd ucel && cargo test -p ucel-testkit` => FAIL（既知の既存失敗: `support_bundle_manifest_fixture_is_sane` が `/workspace/profinaut/fixtures/support_bundle/manifest.json` 不在）
  - `cd ucel && cargo test -p ucel-core -p ucel-registry -p ucel-testkit` => FAIL（上記 `ucel-registry` 既存失敗で停止）
- trace-index json.tool OK
  - `python -m json.tool docs/status/trace-index.json > /dev/null`
- Secrets scan OK
  - `rg -n "AKIA|SECRET|PRIVATE KEY|BEGIN RSA" <changed files>`
  - 結果: ヒットなし
- docsリンク存在チェック OK（今回触った docs 内 `docs/` 参照のみ）
  - `rg -n "docs/" docs/specs/ucel/venue_access_policy_v1.md ucel/docs/policies/venue_access_policy.md`

## 4) ★履歴確認の証拠（必須）
- 実行コマンド:
  - `git log --oneline --decorate -n 50`
  - `git log --graph --oneline --decorate --all -n 80`
  - `git show <直近コミットSHA>`
  - `git show <coverage_policy 最終変更SHA>`
  - `git blame -w ucel/docs/policies/coverage_policy.md`
  - `git blame -w ucel/crates/ucel-registry/src/lib.rs`
  - `git blame -w ucel/crates/ucel-registry/src/hub/rest.rs`
  - `git blame -w ucel/crates/ucel-registry/src/hub/ws.rs`
  - `git blame -w ucel/crates/ucel-registry/src/invoker/mod.rs`
  - `git reflog -n 30`
  - `git merge-base HEAD origin/master`（この環境では `origin/master` 未設定のため実行不可）
  - `git branch -vv`
  - `git log --merges --oneline -n 30`
  - `git show <latest merge sha> --stat`
- 主要 SHA:
  - 直近: `5b7a4920`（docs(ucel) roadmap commit）
  - coverage_policy 最終変更: `9794ef642400bf80120aba1aaf62c4cc3309d270`
  - registry/lib.rs 最終変更: `6ace9ab96db707c3929e51eaaeea06068da1f081`
  - 直近 merge: `46769647223ee0d1e987a9d421d04d8085d0d2a0`
- 履歴からの結論:
  - `coverage_policy.md` では domestic private 4 venue + `sbivc` exception が既に明示されており、このタスクで policy JSON に同値を固定した。
  - `registry/lib.rs`, `hub/rest.rs`, `hub/ws.rs`, `invoker/mod.rs` は hotspot であるため、差分を import + gate 呼び出し中心に局所化した。
  - venue access policy 相当の既存モジュールは無く、新規 `ucel-core::policy` / `ucel-registry::policy` を追加して責務分離した。
- `coverage_policy.md` と `jp_resident_access.json` の接続:
  - `coverage_policy.md` に policy linkage セクションを追加し、JSON SSOT と default/public_only ルールを明文化。
  - `jp_resident_policy_gate` テストで markdown の domestic/sbivc 記述と JSON scope を照合。
- 不足分を追加実装した根拠:
  - 既存 Hub/Invoker では visibility/requires_auth を見ても residency policy を参照していなかったため、REST/WS resolve 直後に `enforce_surface_for_catalog_entry` を追加して送信前遮断を実装。

## 5) Environment limitation notes
- この clone には remote が設定されておらず `origin/master` が存在しないため、`merge-base` および `origin/master...HEAD` 比較は実行不能だった。
