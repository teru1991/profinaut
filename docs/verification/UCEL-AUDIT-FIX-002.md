# UCEL-AUDIT-FIX-002 Verification

## 1) Changed files
- (paste) `git diff --name-only`

```
docs/status/trace-index.json
docs/verification/UCEL-AUDIT-FIX-002.md
services/marketdata-rs/Cargo.lock
services/marketdata-rs/symbol-master/Cargo.toml
services/marketdata-rs/symbol-master/config.yaml
services/marketdata-rs/symbol-master/src/app.rs
services/marketdata-rs/symbol-master/src/config.rs
services/marketdata-rs/symbol-master/src/http.rs
services/marketdata-rs/symbol-master/src/lib.rs
services/marketdata-rs/symbol-master/src/main.rs
services/marketdata-rs/symbol-master/src/resync_loop.rs
services/marketdata-rs/symbol-master/tests/smoke_startup.rs
ucel/crates/ucel-symbol-store/src/lib.rs
ucel/crates/ucel-symbol-store/src/replay.rs
ucel/docs/symbol-master-remaining-tasks.md
```

## 2) What / Why
- symbol-master の main.rs が scaffold で常駐運用が成立していなかったため、設定読込・HTTP(/healthz,/readyz)・resync coordinator・graceful shutdown を追加して“運用成立”へ。
- UCEL-AUDIT-FIX-001 の resync 契約（watch）を受け取り、health に反映する最小導線を固定。
- バイナリ資産は追加していない（text only）。

## 3) Self-check results
- Allowed-path check OK:
  - `git diff --name-only | awk ...` の結果は空（`0`）。
- Tests added/updated OK:
  - `services/marketdata-rs/symbol-master/tests/smoke_startup.rs`
- Build/Test commands:
  - `cargo test --manifest-path services/marketdata-rs/Cargo.toml -p symbol-master` => PASS
- fmt/clippy:
  - `cargo fmt --all -- --check` => WARN（repo root に Cargo.toml が無い）
  - `cargo clippy --manifest-path services/marketdata-rs/Cargo.toml --workspace --all-targets -- -D warnings` => WARN（既存 `ucel-ws-subscriber` の clippy エラー）
- Secrets scan:
  - `git diff | rg -n "(api_key|secret|token|passphrase|PRIVATE KEY)"`（ヒット無し）
- Binary file check:
  - `git diff --name-only --diff-filter=A | rg -n "\.(png|jpg|jpeg|gif|zip|bin|exe|dll|so|dylib)$"`（ヒット無し）

## 4) ★History evidence (required)
- `git log --oneline --decorate -n 50`:
  - HEAD は `79765d3 ucel: add symbol store checkpoint/replay + resync contract`。
  - 直近は UCEL の監査/カバレッジ強化が連続。
- `git log --graph --oneline --decorate --all -n 80`:
  - `work` から `feature/ucel-audit-fix-001` を経由して本ブランチを作成した流れを確認。
- `git log --merges --oneline -n 30`:
  - 直近 merge は #426〜#410 連続。
- `git show HEAD`:
  - 直前コミットの目的は checkpoint/replay + resync 契約追加であり、本タスクの実装対象（symbol-master 常駐化）と整合。
- `git reflog -n 30`:
  - `feature/ucel-audit-fix-001` から本ブランチ作成を確認。
- `git merge-base ...`:
  - origin remote が未設定で算出不可（環境制約）。

- 対象ファイル履歴
  - `git log --oneline -n 30 -- services/marketdata-rs/symbol-master/src/main.rs` は `318b253 Add UCEL symbol master core crates and service scaffold` のみ。
  - `git blame -w services/marketdata-rs/symbol-master/src/main.rs` でも同コミットのみで、scaffold（空実装）意図を確認。
  - `rg -n "healthz|readyz|axum::Router|hyper::Server|support_bundle" services/ -S` で `marketdata-rs`/`ucel-ws-subscriber` に既存 health 実装があることを確認し、axum ベースで整合。
