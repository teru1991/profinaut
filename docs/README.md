# docs/ (Single Source of Truth)

この `docs/` は Profinaut の **設計・契約・運用の正（SSOT）** を管理します。  
「どれが正か」を迷わせないために、ここで **SSOTの階層**を固定します。

---

## SSOT（唯一の正）

### 1. 契約（JSON Schema）
- **正**：`docs/contracts/`
- ここに置かれる `*.schema.json` が **唯一の正** です。
- 実装（Rust/Python/TS等）は、原則としてこの契約に追随します。

### 2. 固定仕様（Core Spec）
- **正**：`docs/specs/`
- ここは **不変の契約・責務境界・状態機械・安全原則**を定義します。
- 数値閾値（ms/％/日数/接続数等）は **Policy** に委譲し、ここには書きません。

### 3. 運用値（Policy）
- **正**：`docs/policy/`
- ここは **環境や運用で変わる値**（閾値・保持・上限・通知感度）を管理します。
- Policy変更は Core Spec の契約を変えません（値を変えるだけ）。

### 4. 計画（Plan）
- **正**：`docs/plans/`
- ここは **順序・進捗・マイルストーン**（変わる前提）を管理します。

### 5. 手順（Runbook）
- **正**：`docs/runbooks/`
- ここは運用手順（切替、障害対応、鍵ローテ、復元訓練など）を管理します。

---

## 重要ルール（Non-negotiable）

1. **新しい契約は必ず `docs/contracts/`**
   - 契約（必須フィールド、enum、互換性）を別の場所に増やさない。

2. **`docs/contracts/` と矛盾する仕様/実装は"不具合"**
   - 仕様が古いなら SSOT を更新し、互換性ルールに従って version を上げる。

3. **`docs/legacy/` は参照のみ（正ではない）**
   - 古い文書にしか無い内容を採用する場合は、必ず SSOT（contracts/specs）へ移植してから使う。

---

## UCEL の位置づけ（確定）

UCEL は「収集アプリ」ではなく、全領域で再利用される **統合SDK/ライブラリ**です。

- **UCEL SDK（統合層）**：A（Platform Foundation）+ G（Data Contracts）
- **UCEL Market Data Collector（Golden Standard）**：H（Market Data Platform）
- **UCEL Execution Connector（唯一の発注出口）**：I（Execution Platform）
- **UCEL IR/Disclosure Connector**：R（Equity/IR Analytics）
- **UCEL On-chain Connector**：Q（On-chain Trading / Arbitrage）

横断の固定仕様：
- Safety Interlock（E/J横断）
- Audit & Deterministic Replay（O横断）
- Support Bundle（Y横断）

---

## ドキュメント配置（推奨）

- `docs/contracts/`：契約SSOT（JSON Schema）
- `docs/specs/`：固定仕様（Core Spec）
  - `docs/specs/ucel/`：UCEL関連
  - `docs/specs/crosscut/`：横断仕様（Safety/Audit/Bundle）
  - `docs/specs/system/`：システム全体のSSOT・用語等
- `docs/policy/`：運用値
- `docs/plans/`：計画
- `docs/runbooks/`：手順
- `docs/legacy/`：過去資料（参照のみ）

---

## Legacy（過去文書）
過去の仕様・設計・メモは `docs/legacy/` に隔離します。  
SSOTと矛盾する場合は **必ずSSOT側が正** です。

## UCEL Golden Docs (SSOT)
- Entry: docs/specs/ucel_golden/README.md

## Docs OS Index (new)
- SSOT Map: docs/specs/system/ssot_map.md
- Docs Gate: docs/specs/system/docs_gate_spec.md
- Data Classification: docs/specs/security/data_classification_handling_spec.md
- Env/Mode Matrix: docs/specs/system/environment_mode_matrix.md
- Runbooks Index: docs/runbooks/README.md
- UCEL Golden Compatibility: docs/specs/ucel_golden/compatibility_notes.md
- UCEL Library Hardening File-level Plan: docs/plans/ucel/ucel_library_hardening_filelevel_plan.md

## Docs OS Operations
- docs/specs/system/docs_os_operations_policy.md

## System Templates & Ledgers
- docs/specs/system/domain_spec_template.md
- docs/specs/system/cross_domain_dependency_matrix.md
- docs/specs/system/error_catalog_ssot.md
- docs/specs/system/schema_registry_contract_index.md
- docs/specs/system/runbook_index.md
- UCEL Decimal Policy: docs/specs/ucel/decimal_policy/A1_core_policy_and_tick_step.md
- UCEL Decimal Policy Spec: docs/specs/ucel/decimal_policy/decimal_policy_spec.md
