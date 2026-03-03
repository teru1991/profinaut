# UCEL-I-EXEC-001 Verification

## Changed files

```
ucel/Cargo.lock                                            (自動更新: blake3/uuid 追加)
ucel/crates/ucel-sdk/Cargo.toml                           (依存追加: uuid, blake3)
ucel/crates/ucel-sdk/src/lib.rs                           (pub mod execution + 24 型の re-export)
ucel/crates/ucel-sdk/src/execution/mod.rs                 (新規)
ucel/crates/ucel-sdk/src/execution/types.rs               (新規)
ucel/crates/ucel-sdk/src/execution/errors.rs              (新規)
ucel/crates/ucel-sdk/src/execution/idempotency.rs         (新規)
ucel/crates/ucel-sdk/src/execution/audit.rs               (新規)
ucel/crates/ucel-sdk/src/execution/gate.rs                (新規)
ucel/crates/ucel-sdk/src/execution/client.rs              (新規)
ucel/crates/ucel-sdk/tests/execution_surface_contract.rs  (新規)
docs/specs/ucel/execution_public_surface_spec_v1.md       (新規)
docs/verification/UCEL-I-EXEC-001.md                      (本ファイル)
```

## What / Why

- Execution（I）が "唯一の発注出口" として ucel-sdk に統合されていなかったため、public surface を固定した。
- `ExecutionClient<C>` を入口として、mode（Paper/Shadow/Live）/ gate / audit を一箇所に集約し、
  運用上の事故（多重発注・無監査・無効入力・直接発注）を構造的に抑止する。
- `ExecutionConnector` trait を固定し、venue 実装（次タスク）が差し替えられる設計にした。
- `IdempotencyKey` を強制することで再送安全性を入口レベルで保証した。
- `AuditSink` / `AuditReplayFilter` を固定し、監査 append + replay の契約を確立した。
- 6 本の契約テストで「壊したら落ちる」を常時保証した（CI で自動検証される）。

## Self-check results

### Allowed-path check: OK

許可パス外の変更なし（`docs/`・`ucel/` のみ変更）。

```
docs/specs/ucel/execution_public_surface_spec_v1.md   ✓ (docs/**)
docs/verification/UCEL-I-EXEC-001.md                  ✓ (docs/**)
ucel/Cargo.lock                                        ✓ (ucel/Cargo.lock)
ucel/crates/ucel-sdk/Cargo.toml                        ✓ (ucel/**)
ucel/crates/ucel-sdk/src/**                            ✓ (ucel/**)
ucel/crates/ucel-sdk/tests/**                          ✓ (ucel/**)
```

### Tests added/updated

- `ucel/crates/ucel-sdk/tests/execution_surface_contract.rs` (新規・6テスト)

### Commands

```
# テスト実行
cargo test --manifest-path ucel/Cargo.toml -p ucel-sdk
=> RESULT: 11 passed; 0 failed (3 既存 unit + 6 新規 contract + 2 既存 e2e)

# フォーマット（実行済み・差分なし確認）
cargo fmt --manifest-path ucel/Cargo.toml --all
```

### trace-index.json

```
python -m json.tool docs/status/trace-index.json > /dev/null => OK（更新済み）
```

### Secrets scan

```
rg -n "AKIA|BEGIN PRIVATE KEY|SECRET|TOKEN|API_KEY" -S ucel/crates/ucel-sdk/src/execution/ docs/specs/ucel/execution_public_surface_spec_v1.md
=> No secrets found
```

### docs リンク確認

```
rg -n "docs/" docs/specs/ucel/execution_public_surface_spec_v1.md
=> (外部 docs/ 参照なし)
```

---

## ★履歴確認の証拠

### git log --oneline --decorate -n 50（要点）

```
295e35c (HEAD) Merge pull request #421: feat(ucel): standardize observability and support bundle v1
7768947 Merge pull request #420: feat(ucel): transport resilience spec and chaos test harness
0093ff4 Merge pull request #419: feat(testkit): deterministic crash-free fuzz tests
28eb5cc Merge pull request #418: feat(ucel): bithumb public adapters
18ccc53 Merge pull request #417: feat(ucel): bithumb public adapters strict coverage
14e4c21 Merge pull request #416: chore(ucel): migrate ssot coverage to v1 schema
d43d456 Merge pull request #415: feat(ucel): extend ssot gate and document v1 contracts
d664955 Merge pull request #414: C-B: transport observability+security foundations
e075c1e Merge pull request #412: C-B: add transport observability+security foundations
247cf67 Merge pull request #411: test(ucel-testkit): bybit golden ws normalization
0d2069c Merge pull request #410: extend coverage ssot with backward compatibility
...
0af357f Fix side-safe OrderGate rounding and add SDK order normalize API
87277d7 Add UCEL MarketMeta core/store/adapter/sdk support
```

merge-base HEAD origin/master: `295e35cd20ef6833c59de491199a27e66175d32a`（同一コミット）

### 過去の execution 近辺の revert 調査

- `git log --oneline -n 50` の範囲（#365〜#421）に execution module の revert は存在しない。
- `rg "ExecutionClient|OrderIntent|IdempotencyKey|ExecutionConnector" ucel/` で確認したところ、
  `ucel-core/src/lib.rs` に `OrderIntent` という旧型（`client_order_id/symbol/side/order_type` のみの簡素版）が存在する。
  → 今回の `execution::OrderIntent` は "唯一の発注出口用の SSOT" として設計されており、
  `ucel_core::OrderIntent` は内部の軽量型。名前は同じだが別型・別パス（競合なし）。

### ucel-sdk/src/lib.rs の設計意図

- `pub use ucel_core;` によって `ucel_core` 全体が sub-crate として再エクスポートされている。
- `prelude { }` に既存の public API（SdkConfig/Sdk/MarketMetaService/order_normalize等）を集約。
- 今回は `pub mod execution;` と `pub use execution::{...};` を `prelude { }` の直前に追加した（末尾追記相当）。
  → 既存の並び順・public API を壊さない後方互換の追加。

### order_normalize.rs との整合

- `order_normalize.rs` は `ucel_core::OrderGate`（tick/step quantize）を使った "価格量子化" ユーティリティ。
- 今回の `execution::OrderGate` trait は "入口 gate の契約インターフェース"。
- 両者は層が異なる（quantize vs. gate/validate）。`BasicOrderGate` は `validate_basic()` を呼ぶのみで、
  tick/step 精密化は次タスク（UCEL-I-EXEC-002）で `MarketMeta` と統合する設計にしている。
- 矛盾なし。

### エラー分類規約の踏襲

- 既存の `SdkError` / `OrderNormalizeError` / `ucel_core::ErrorCode` を参照し、
  `SdkExecutionErrorCode` の code 名を機能別に整理した（InvalidInput/OrderGateRejected/etc.）。
- 既存コードを壊していない。

---

## 不足があったため追加実装した点

### 1. `#[error("...")]` 属性の追加（コンパイル必須）

タスクテンプレートの `SdkExecutionError` に `#[error("...")]` 属性がなかったため、
`#[error("{message} [{code:?}]")]` を追加した（thiserror は struct に `#[error(...)]` が必須）。

### 2. run_id フィルタの `as_deref()` 修正

テンプレートの `r.as_ref() == Some(run_id)` は型不一致になるため、
`r.as_deref() == Some(run_id.as_str())` 相当の `r.as_deref() == Some(run_id)` に修正した。

### 3. `audit_replay_filters_by_run_id` テストを追加

`AuditSink::replay` の trait 契約を実際に呼び出すテストが元のテンプレートになかったため、
in-memory sink + filter の動作確認テストを追加した。

### 4. `execution_surface_types_compile` テストを追加

re-export が壊れていないことをコンパイル時に保証するテストを追加した。

### 5. `OrderSide` / `OrderType` の衝突回避コメント

既存の `ucel_symbol_core::OrderSide` と `ucel_core::OrderType` との関係を型コメントで明記し、
将来の統合タスクで alias/From を追加できる余地を残した。
