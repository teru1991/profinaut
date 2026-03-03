# UCEL Execution Public Surface Spec v1

- Document ID: UCEL-I-EXEC-PS-V1
- Status: Canonical / Fixed Contract
- Task: UCEL-I-EXEC-001
- Depends-on: UCEL-EXE-CONNECTOR-SPEC (`execution_connector_spec.md`), `order_gate_spec.md`

## Goal

UCEL の Execution（I）を "機械的に証明できる" 形で完結させるため、
**唯一の発注出口**を `ucel-sdk::execution` に固定する。

---

## Single Public Surface

- Public surface: `ucel-sdk::execution`
- これ以外の crate / module が "直接発注" してはならない（venue 実装は `ExecutionConnector` を実装するのみ）。
- 外部コードは `use ucel_sdk::execution::*;` または `ucel_sdk::{ExecutionClient, OrderIntent, ...}` でアクセスする。

---

## Core Types（固定）

| 型 | 説明 |
|---|---|
| `OrderIntent` | 発注意図の正規モデル。idempotency/audit の核 |
| `OrderIntentId` | `OrderIntent` の識別子（決定的生成） |
| `IdempotencyKey` | 入口で必須。空・非ASCII printable は拒否。16〜128文字 |
| `ExecutionClient<C>` | 唯一の発注入口。mode/gate/audit を統合 |
| `ExecutionConnector` | venue 実装が満たすべき trait |
| `OrderGate` | 入口 gate の trait（v1: BasicOrderGate） |
| `AuditSink` / `AuditEvent` | 監査: append-only + replay |
| `AuditReplayFilter` | replay の絞り込み条件 |
| `InMemoryAuditSink` | テスト/開発用 in-memory 実装 |
| `ReconcileReport` / `ReconcileSource` | 照合レポート（best-effort） |

---

## Modes

| Mode | 説明 | connector.place_order 呼び出し |
|---|---|---|
| `ExecutionMode::Paper` | 外部発注を**絶対に行わない**。receipt は Accepted を返す | ❌ しない |
| `ExecutionMode::Shadow` | 外部発注を**行わない**。監査に残す。将来 quote/validate で照合強化 | ❌ しない |
| `ExecutionMode::Live` | 実発注。idempotency/audit が必須 | ✅ する |

---

## Gate

- 入口で `OrderGate::validate` を必ず適用する。
- v1 の `BasicOrderGate` が拒否する条件:
  - `venue` が空
  - `symbol` が空
  - `qty <= 0` または非有限
  - `order_type` が `Limit`/`PostOnly` で `price` が None または `<= 0` または非有限
- tick/step/min_notional 等の強い検証は MarketMeta と統合して強化する（v2 / UCEL-I-EXEC-002）。

---

## IdempotencyKey 規約

- `IdempotencyKey::parse(s)`: 外部注入（再送/復旧時の自由度）
- `IdempotencyKey::random_uuid()`: 新規発注の簡易生成
- `IdempotencyKey::derive_from_intent(intent)`: intent JSON を blake3 でハッシュ（安定・決定的）
- 制約: 16〜128文字、ASCII printable のみ（0x21〜0x7E）

---

## AuditSink（監査の差し込み口）

```
trait AuditSink: Send + Sync {
    fn append(&self, event: AuditEvent) -> SdkExecutionResult<Option<String>>;
    fn replay(&self, filter: AuditReplayFilter) -> SdkExecutionResult<Vec<AuditEvent>>;
}
```

- v1 実装: `InMemoryAuditSink`（テスト/開発用）
- 本番実装（WAL/ファイル/remote）は UCEL-I-EXEC-002 で追加
- `AuditEvent` バリアント:
  - `OrderRequested`, `OrderResult`, `CancelRequested`, `CancelResult`, `ReconcileResult`

---

## Compatibility Rules

1. public type のフィールド削除/名前変更は禁止（後方互換を壊す）。
2. フィールド追加は許可（serde の default を用意して互換を保つ）。
3. `SdkExecutionErrorCode` の既存 variant の意味変更は禁止。新規追加のみ可。
4. `ExecutionConnector` / `OrderGate` / `AuditSink` のシグネチャ変更は MAJOR 扱い（SemVer）。
5. `ExecutionMode` の variant 追加は後方互換。削除は MAJOR。

---

## Proof / CI

`ucel-sdk/tests/execution_surface_contract.rs` が常時通ること:

| テスト | 検証内容 |
|---|---|
| `idempotency_must_be_nonempty_and_printable` | 空/空白の IdempotencyKey を拒否 |
| `gate_rejects_missing_price_for_limit` | Limit で price なしを gate が拒否 |
| `paper_and_shadow_do_not_call_connector_place` | Paper/Shadow が venue_order_id=None |
| `idempotency_derive_is_stable_for_same_intent` | derive_from_intent が決定的 |
| `audit_replay_filters_by_run_id` | AuditSink trait + replay が機能 |
| `execution_surface_types_compile` | 全 re-export 型がコンパイルで到達可能 |

---

## Next（UCEL-I-EXEC-002）

- `ExecutionConnector` の venue 実装（1〜2 venue から本実装）
- 監査の永続化（WAL/ファイル）と E2E 証明（再送・クラッシュ復元・replay）
- `MarketMeta` と `OrderGate` の統合強化（tick/step/min_notional を入口で必ず enforce）
- `Shadow` モードで quote/validate/constraints を実接続して照合強化
