# Data Governance / Catalog / Lineage Core Spec v1.0（固定仕様）
Dataset catalog SSOT / Lineage & provenance / Quality metadata / Evidence-linked governance

- Document ID: DATA-CATALOG-LINEAGE-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): V（Data Governance）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Market Data Collector: `docs/specs/market_data/collector_framework_spec.md`
  - On-chain Ingestion: `docs/specs/onchain/onchain_ingestion_finality_reorg_integrity_spec.md`
  - Accounting/Ledger: `docs/specs/accounting/order_trade_ledger_pnl_spec.md`
  - Equities IR: `docs/specs/equities_ir/equities_ir_financials_ingestion_scoring_spec.md`
  - FX/Macro: `docs/specs/fx_macro/fx_macro_ingestion_normalization_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
- Policy separation（固定しない）:
  - retention、分類ラベル、アクセス権詳細、承認フロー → `docs/policy/**`
  - 運用手順（登録/廃止/問い合わせ/監査対応）→ `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
データが増えるほど「どれが正か」「どこから来たか」「品質はどうか」が曖昧になり、事故と手戻りが増える。
本仕様は、全データセットを **カタログ（SSOT）**で管理し、**系譜（lineage）**と**品質（quality）**を固定的に表現し、
再現・監査・アクセス制御へ繋げる不変条件を固定する。

必達要件（固定）：
1) **Catalog SSOT**：データセットは一意に識別され、正本が迷わない
2) **Lineage is explicit**：生成物は入力・処理・出力の系譜を持つ（provenance）
3) **Quality metadata**：欠損/遅延/UNKNOWN が隠れない（integrityと連動）
4) **Evidence-linked**：データは replay_pointers / integrity_report / audit_event へ辿れる
5) **Change-aware**：スキーマ/版の変化を明示し、破壊的変更を防ぐ
6) **Access-controlled**：データアクセスは最小権限、監査可能
7) **No secrets**：カタログ/系譜/メタに秘密を入れない

---

## 1. 範囲（in / out）
### 1.1 In
- データセットの一意識別（dataset_id）と分類（raw/canonical/derived）
- メタデータ（所有者、用途、更新頻度、保持、品質）
- lineage（入力→処理→出力）の表現
- schema/version の管理（破壊変更防止）
- access control（閲覧/エクスポートの境界）
- evidence refs（integrity/audit/replay pointers）

### 1.2 Out
- 具体のカタログ製品選定（Glue等）
- UIの詳細
- 具体のACL実装詳細（Policy/Runbook）

---

## 2. Dataset Identity（固定）
### 2.1 dataset_id（固定）
すべてのデータセットは `dataset_id` で一意に識別できる。
dataset_id は少なくとも以下を含意できる：
- domain（market_data/onchain/ledger/equities_ir/fx_macro等）
- layer（raw/canonical/derived）
- scope（venue/chain/issuer/seriesなど）
- format/table（論理名）

固定ルール：
- dataset_id は時間が経っても意味が変わらない
- dataset_id の再利用は禁止（廃止は deprecate として表現）

### 2.2 dataset metadata（固定最小セット）
- `dataset_id`
- `description`
- `owner`（team/service）
- `source_system`（collector/onchain/ledger等）
- `layer`（raw/canonical/derived）
- `schema_version`（該当する場合）
- `freshness_expectation`（目安：具体値はPolicy）
- `retention_policy_ref`（Policy参照）
- `quality`（OK/DEGRADED/UNKNOWN）
- `access_classification`（public/internal/restricted 等：詳細はPolicy）
- `evidence_refs`（integrity/audit/replay pointers 参照）

---

## 3. Lineage（固定）
### 3.1 Lineage record（固定要件）
lineage は “生成物” に必ず付与できる：
- `output_dataset_id`
- `input_dataset_ids[]`
- `transform_id`（処理/ジョブ識別）
- `transform_version`（バイナリhash or SemVer）
- `window`（対象期間/パーティション）
- `policy_snapshot_ref`（処理ルール）
- `replay_pointers_ref`（入力参照）
- `quality`（OK/DEGRADED/UNKNOWN）
- `notes`（差分/例外）

固定ルール：
- lineage が不明な派生データは “正” として扱わない（最低でも UNKNOWN）
- lineage は audit_event で追跡できる

### 3.2 Provenance（固定）
- 入力の出所（source refs）を辿れること
- raw→canonical→derived の関係が明示されること

---

## 4. Quality metadata（固定）
### 4.1 Quality levels（固定）
- `OK`
- `DEGRADED`
- `UNKNOWN`

固定ルール：
- UNKNOWN を “健康” と表示しない（reporting/observability準拠）
- quality は integrity_report の結果と整合する

### 4.2 Quality sources（固定）
quality は少なくとも以下から決まる（実装自由だが概念固定）：
- missing intervals / gaps
- staleness
- source disagreement
- quarantine impacts
- observability missing intervals
- schema mismatch / parse failures

---

## 5. Schema & Versioning（固定）
### 5.1 schema_version（固定）
- 構造化データは schema_version を持つ
- 破壊変更は新schema_versionへ（versioning_policy準拠）

### 5.2 Backward compatibility（固定）
- 古いデータを読めること（最低限の再現性）
- 互換性破綻がある場合は catalog に明示し、移行計画（plans）に繋ぐ

---

## 6. Access control（固定）
### 6.1 Access classes（固定枠組み）
- public（公開可）
- internal（内部）
- restricted（機微：税務/個人情報/取引詳細等）

詳細なポリシーは Policy へ。
固定ルール：
- restricted のデータは export/共有が dangerous op になり得る
- access denied は監査（audit_event）に残す（重要拒否）

### 6.2 No secret leakage（固定）
- カタログ/lineageに secret_ref 以外の秘密値を入れない
- forbidden-key scan の対象

---

## 7. Evidence linking（固定）
データセットと lineage は少なくとも以下へ辿れる：
- integrity_report（品質根拠）
- gate_results（検証結果）
- audit_event（取得/処理/変更）
- replay_pointers（入力参照）

固定ルール：
- “どのデータを使ってこの結果が出たか” を辿れることが必須

---

## 8. Change management（固定）
- dataset の新規追加/廃止/再分類は監査対象（audit_event）
- 廃止は stub/redirect（参照切れ防止）を提供できる（概念として）
- 重大変更は governance の gate を通る（release safety）

---

## 9. Audit（固定）
最低限、以下を audit_event に残す（秘密値なし）：
- `catalog.dataset.registered`（dataset_id + metadata summary）
- `catalog.dataset.updated`（diff summary）
- `catalog.dataset.deprecated`（replacement if any）
- `lineage.recorded`（output_dataset_id + refs）
- `catalog.access.denied`（重要拒否）
- `integrity.record` / `gate.record`

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) dataset_id が一意で再利用されない
2) 派生データが input→output の lineage を持つ
3) quality=UNKNOWN/DEGRADED が隠れず、integrityと整合する
4) schema_version の破壊変更が新versionで表現される
5) restricted データのアクセスが制御され、拒否が監査に残る
6) evidence refs（integrity/audit/replay pointers）へ辿れる

---

## 11. Policy/Runbookへ逃がす点
- retention、分類ラベル、アクセス権詳細、承認フロー
- 登録/廃止/問い合わせ/監査対応手順
→ Policy/Runbookへ（意味は変えない）

---
End of document
