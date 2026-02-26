# Level 1 SSOT Outline（K：Portfolio / Treasury｜統一台帳・資金管理）v1.4

## 0. ドキュメントメタ
- ドメイン文字: **K**
- 名称: Portfolio / Treasury（統一台帳・資金管理）実装目標機能詳細設計【完全版 v1.4】
- 位置づけ: 取引所/オンチェーン/口座/ボット/戦略を横断する **SBOR（Single Book of Record）** と **Treasury統制** のSSOT

---

## 1. Non-negotiable（目的と到達点）
### 1.1 目的
- 残高・ポジション・注文・約定・費用・移動・評価を **単一の真実（SBOR）** として統合し、監査可能・復元可能・安全・高速・正確に運用できる資産基盤を提供する。
- 監査強度・改ざん耐性・誤操作/不正耐性・全期間再計算・提出パックまで含め、長期運用で詰まらない“究極”の資金/資産基盤を確立する。

### 1.2 必達（不変の要求）
- イベントソーシング：全状態はイベントから再構築可能（スナップショットはキャッシュ）。
- 厳密分解/帰属：PnL（実現/含み）・費用（手数料/資金調達/金利/ガス…）を分離し、bot/戦略へ帰属可能。
- 品質表明：価格欠損・遅延・照合差分・未確定（pending）をConfidenceへ反映し隠さない。
- 移動統制：出金・内部振替をポリシー＋承認WF＋timelock＋ガードレールで統制。
- 照合と修復：取引所/チェーン実態と台帳の差分検知→自動/手動修復導線。
- 説明可能性：任意時点の残高/損益/NAVが根拠イベントへ辿れ、因果候補を提示できる。
- 再現性固定：同イベント列→同状態（schema_version＋calc_version）保証、全期間再計算可能。
- 改ざん検知/監査提出：台帳イベントは改ざん検知可能で、監査提出パックを生成できる。
- 安全弁：影台帳（Shadow Book）等により“実装バグ”自体も検知できる。

---

## 2. スコープ
### 2.1 Kが責任を持つ
- 統一台帳（残高/ポジ/注文/約定/費用/入出金/内部移動/補正/資産イベント）
- 評価（Mark-to-Market）と基軸通貨換算（JPY/USD等）
- PnL Attribution（損益原因分解・帰属・ロット原価）
- Exposure / Concentration（露出・集中）
- NAV / Drawdown（純資産・DD）
- Treasury（資金繰り計画・移動要求・承認・安全統制・防火壁）
- Reconciliation（照合・差分・修復導線・二重系検算）
- Data Quality / Confidence（信頼度スコア/SLO/整合性モデル）
- Retention/Archive、DR/復旧訓練、監査用提出パック
- Stress/Scenario（リスク材料の試算）
- 改ざん検知、異常検知、誤操作防止、変更管理
- 全期間再処理、影台帳、説明グラフ、RCA、自動提案（半自動）

### 2.2 Kが直接はしない（連携）
- 発注実行（I）
- 鍵・署名・秘密情報管理（B）
- リスク判断の最終ゲート（J）
- 税務/会計の最終出力（W）
- 監査ログの保全機構そのもの（O）※Kは監査向けイベントを生成

---

## 3. 依存関係（参照ドメイン）
- I Execution：注文/約定/残高・ポジスナップ、入出金履歴
- H Market Data：価格、FXレート、指数等（評価・流動性）
- J Risk Gate：制約・露出・ストレス結果・異常検知の通知
- B Secrets：強操作（出金等）の承認・実行の分離（秘密最小化）
- E Safety：SAFE_MODE / CLOSE_ONLY / FLATTEN 連動
- O Audit：証跡保存（台帳・承認・補正・変更履歴・提出）
- W Reporting：税務/仕訳/分類、外部エクスポート
- S Dashboard / Access：表示とアクセス制御（秘匿・マスキング）

---

## 4. 台帳原則（不変条件）
- Idempotency：同一source_eventは二重計上しない
- Append-only：修正禁止、補正はAdjustmentイベント
- Explainability：状態→根拠イベントへ必ず辿れる（説明グラフ対応）
- 三重時刻：event_time / recv_time / persist_time（時刻規律の監視）
- 確定/未確定：pending/confirmed/finalized（reorg等含む）
- 整合性モデル明示：確定値/暫定値の境界を仕様として固定（**K-46**）
- 品質表明：Degraded/Unknown/Confidence Score/SLOを必ず返す
- 強操作分離：Kは出金実行しない（要求・許可・証跡）
- 再現性固定：schema_version + calc_version（全期間再計算可能）
- 改ざん検知：イベントチェーン/ハッシュで欠落・改ざん検知
- 変更ガバナンス：重要設定は変更申請→レビュー→適用→ロールバック
- 秘密最小化：機微情報はマスキング・アクセス制御・監査可能

---

## 5. Canonical Model / Contract（正規化データモデル）
### 5.1 LedgerEvent（Append-only）
#### 5.1.1 共通ヘッダ（必須フィールド）
- event_id（内部UUID）
- source（venue/account/source_kind）
- source_event_id（order_id, trade_id, transfer_id, tx_hash+log_index 等）
- event_type
- event_time / recv_time / persist_time
- finality_state（pending/confirmed/finalized/reorged 等）
- schema_version
- calc_version
- trace_id / run_id
- prev_event_hash / event_hash（改ざん検知：**K-37**）
- time_quality（時刻規律：**K-49**）
- redaction_level（秘匿：**K-42/60**）

#### 5.1.2 イベント種別（体系）
- Snapshot：BalanceSnapshot, PositionSnapshot
- Trade/Order：OrderUpsert, TradeFill
- Costs：FeeCharged, FundingPayment, InterestAccrual, GasCharged, RebateGranted
- Transfers：TransferIn, TransferOut, InternalTransfer
- Adjust/Recon：Adjustment, ReconciliationDiff, ReconciliationRepairAttempted
- Asset Events：Airdrop, TokenSwap, Delist, SplitMerge
- Treasury/Approval：TransferRequestCreated/Approved/Rejected/Cancelled/Expired
- Constraints/Quality：ConstraintViolationDetected, QualityDegraded, SLOViolated
- Scenario：StressScenarioRun, KillSwitchAccountingComputed
- Dual-Verify：DualSourceMismatchDetected
- Anomaly：AnomalyDetected
- Change Mgmt：ConfigChangeProposed/Approved/Applied/RolledBack
- Compliance：AccessLogEmitted, RetentionArchived, RestoreDrillResult, AuditExportPackGenerated
- Shadow Book：ShadowBookMismatchDetected
- RCA：RootCauseHypothesisGenerated
- Autopilot：TreasuryAutopilotSuggestionGenerated

> TODO: 各event_typeのpayload schema（必須/任意/単位/丸め/通貨・銘柄表現/識別子規約）を別紙Contractとして固定。

### 5.2 Materialized States（現在状態）
- BalanceState：available/locked/total + pending_delta
- PositionState：qty/avg_price/notional/unrealized_pnl/liquidation?
- OrderState：status/open_qty/filled_qty/price/type/tif/last_update
- PnLState：realized/unrealized/fees/funding/interest/gas/other + tax_lots
- ExposureState：gross/net/delta/notional/base_ccy_value + liquidity_capacity
- NavState：nav/base_ccy, drawdown metrics + kill_switch_nav
- QualityState：confidence_score / degraded_reasons / price_staleness / pending_ratio / time_discipline
- FinalityState：pending/confirmed/finalized/reorg_risk
- ConstraintState：violations（ルールID、重大度、影響推定）
- SLOState：SLO違反、予算消費（**K-43**）
- ConsistencyState：確定値/暫定値の境界（**K-46**）
- AuditState：提出パック履歴、改ざん検知結果、アクセス履歴

> TODO: Materialized Stateの更新規則（どのeventがどのstateへ、順序、集計窓、丸め、例外）をcalc_version単位で固定。

---

## 6. Capabilities（機能分解：K-01〜K-60）
### 6.1 台帳中核（K-01〜K-11）
- K-01 Ingestion Adapters
- K-02 Ledger Writer
- K-03 State Builder
- K-04 Valuation Engine
- K-05 PnL Attribution
- K-06 Exposure / Concentration
- K-07 NAV / Drawdown
- K-08 Allocation Ledger
- K-09 Treasury Policy
- K-10 Reconciliation
- K-11 Audit & Explain

### 6.2 完全性・オンチェーン（K-12〜K-25）
- K-12 Tax Lot / Cost Basis
- K-13 Asset Events
- K-14 Margin / Collateral / Borrow-Lend
- K-15 Funding/Interest 精密化
- K-16 Fee Classification 完全化
- K-17 Approval Workflow
- K-18 Withdrawal Safety Enhancements
- K-19 Treasury Planner
- K-20 Multi-Book / Partition
- K-21 Replay & Determinism
- K-22 Reconciliation 自動修復高度化
- K-23 Data Quality / Confidence Score
- K-24 On-chain Portfolio
- K-25 Bridge / Wrapped Asset 正規化

### 6.3 長期運用の最後の層（K-26〜K-36）
- K-26 Portfolio Constraints Engine
- K-27 Liquidity & Capacity
- K-28 Settlement & Pending State
- K-29 Fee Tier / VIP / Rebates Tracker
- K-30 Multi-Currency Cash Management
- K-31 Internal Ledger for Simulated Funds
- K-32 Strategy Attribution（按分・帰属）
- K-33 Stress / Scenario Engine
- K-34 Insurance / Bankruptcy / Exchange Failure Modeling
- K-35 Data Retention & Cold Storage
- K-36 DR / Backup / Restore Drill

### 6.4 監査・安全・変更管理（K-37〜K-45）
- K-37 Tamper-Evident Ledger（改ざん耐性）
- K-38 Dual-Source Verification（二重系検算）
- K-39 Fraud / Anomaly Detection（不正・異常検知）
- K-40 Action Guardrails（誤操作防止）
- K-41 Performance & Scalability（性能・スケール）
- K-42 Privacy / Redaction（秘匿・マスキング）
- K-43 SLA / SLO Budget（品質予算）
- K-44 Change Management（変更管理）
- K-45 Legal/Compliance Hooks（準拠フック）

### 6.5 究極オプション統合（K-46〜K-60）
- K-46 Byzantine/Consistency Model（不一致モデルの形式化）
- K-47 Multi-Region / Air-Gapped Audit Copy（監査用コピー）
- K-48 Hardware Attestation Hooks（ハード証明フック）
- K-49 Deterministic Time Discipline（時刻規律の超強化）
- K-50 Contract/Schema Proof Artifacts（契約証明成果物）
- K-51 “Explain Anything” Graph（説明グラフ）
- K-52 Automated Root Cause Analysis（自動原因推定）
- K-53 Human-in-the-Loop Policy Simulator（ポリシー事前検証）
- K-54 Portfolio Firewall（資産防火壁）
- K-55 “Kill-Switch Accounting” Mode（緊急会計モード）
- K-56 Full Reprocessing Pipeline（全期間再処理）
- K-57 “Shadow Book” Parallel Ledger（影台帳）
- K-58 Capacity-Aware Treasury Autopilot（資金自動操縦）
- K-59 External Audit Export Packs（監査提出パック）
- K-60 Secrets-Minimizing Proof（秘密最小化の証明）

---

## 7. 外部I/F（読み取り・要求）
### 7.1 読み取り
- Summary：NAV/PnL/Exposure/DD/Confidence/SLO/制約違反/pending/緊急会計
- Explain：根拠イベント＋価格＋料率履歴＋按分＋説明グラフ＋原因候補
- Reconciliation：差分/原因推定/修復履歴/二重系不一致/影台帳不一致
- Stress：シナリオ別のNAV/DD/余力/制約違反
- Compliance：アクセス履歴、アーカイブ状況、監査提出パック、改ざん検知結果、時刻規律状態

### 7.2 要求
- Transfer Request：内部振替/出金要求（検証→承認→期限→防火壁）
- Adjustment：補正（理由必須・監査必須・強操作）
- Constraints/Fee Tier/Calc Version更新：変更管理（K-44）必須、事前シミュレータ（K-53）推奨
- Full Reprocess Run：全期間再計算の実行（監査化）
- Audit Export Pack Run：提出パック生成

> TODO: API/CLI/メッセージング（topic/route）、入出力スキーマ、認可モデル（S連携）を固定。

---

## 8. 劣化運転（Degraded/Unknown）
- 価格欠損/乖離：Confidence低下、Explainに理由
- スナップ欠損：ReconciliationをDegraded、再取得強化
- 二重系不一致：Mismatchイベント→自動照合→隔離
- 影台帳不一致：ShadowBookMismatch→即アラート（致命度高）
- pending増大：未確定比率を可視化しSLOへ反映
- 異常検知：重大度に応じてSAFE_MODE要請＋緊急会計（K-55）

---

## 9. Behavior / Tests（テスト・検証＝完成条件）
- 冪等性：重複入力でも二重計上しない
- 再現性：同イベント列→同状態（calc_version含む）
- 保存則：PnL分解合計一致（丸め境界含む）
- 照合：差分は必ず記録され修復導線がある
- 強操作：ガードレール＋承認＋防火壁が必ず適用される
- 改ざん検知：ハッシュ連鎖の断裂/改ざんを検知できる
- 性能：想定負荷でSLO内に収まる
- DR：定期リストア検証が成功している
- 全期間再計算：差分/検証/ロールバックが成立する
- 影台帳：差分が検知され、原因追跡可能

---

## 10. 「Kが究極」と呼べる最低ライン（v1.4 기준）
- K-01〜K-60 が仕様として固定され、少なくとも以下が稼働：
  - SBOR＋Explain＋Confidence＋SLO
  - PnL分解＋ロット原価（K-12）＋按分（K-32）
  - Margin/Borrow（K-14）＋Funding/Interest（K-15）
  - Treasury統制（K-17〜18）＋Planner（K-19）＋防火壁（K-54）
  - Reconciliation（K-10/22）＋二重系（K-38）＋改ざん検知（K-37）
  - Pending/Finality（K-28）＋オンチェーン（必要ならK-24/25）
  - Constraints（K-26）＋Stress（K-33）＋緊急会計（K-55）
  - Retention（K-35）＋DR訓練（K-36）＋監査提出パック（K-59）
  - 全期間再処理（K-56）＋影台帳（K-57）
  - 変更管理（K-44）＋秘匿（K-42/60）＋RCA（K-52）

---

## 11. Capability Index（ID保持）
- K-01 Ingestion Adapters
- K-02 Ledger Writer
- K-03 State Builder
- K-04 Valuation Engine
- K-05 PnL Attribution
- K-06 Exposure / Concentration
- K-07 NAV / Drawdown
- K-08 Allocation Ledger
- K-09 Treasury Policy
- K-10 Reconciliation
- K-11 Audit & Explain
- K-12 Tax Lot / Cost Basis
- K-13 Asset Events
- K-14 Margin / Collateral / Borrow-Lend
- K-15 Funding/Interest 精密化
- K-16 Fee Classification 完全化
- K-17 Approval Workflow
- K-18 Withdrawal Safety Enhancements
- K-19 Treasury Planner
- K-20 Multi-Book / Partition
- K-21 Replay & Determinism
- K-22 Reconciliation 自動修復高度化
- K-23 Data Quality / Confidence Score
- K-24 On-chain Portfolio
- K-25 Bridge / Wrapped Asset 正規化
- K-26 Portfolio Constraints Engine
- K-27 Liquidity & Capacity
- K-28 Settlement & Pending State
- K-29 Fee Tier / VIP / Rebates Tracker
- K-30 Multi-Currency Cash Management
- K-31 Internal Ledger for Simulated Funds
- K-32 Strategy Attribution（按分・帰属）
- K-33 Stress / Scenario Engine
- K-34 Insurance / Bankruptcy / Exchange Failure Modeling
- K-35 Data Retention & Cold Storage
- K-36 DR / Backup / Restore Drill
- K-37 Tamper-Evident Ledger（改ざん耐性）
- K-38 Dual-Source Verification（二重系検算）
- K-39 Fraud / Anomaly Detection（不正・異常検知）
- K-40 Action Guardrails（誤操作防止）
- K-41 Performance & Scalability（性能・スケール）
- K-42 Privacy / Redaction（秘匿・マスキング）
- K-43 SLA / SLO Budget（品質予算）
- K-44 Change Management（変更管理）
- K-45 Legal/Compliance Hooks（準拠フック）
- K-46 Byzantine/Consistency Model（不一致モデルの形式化）
- K-47 Multi-Region / Air-Gapped Audit Copy（監査用コピー）
- K-48 Hardware Attestation Hooks（ハード証明フック）
- K-49 Deterministic Time Discipline（時刻規律の超強化）
- K-50 Contract/Schema Proof Artifacts（契約証明成果物）
- K-51 “Explain Anything” Graph（説明グラフ）
- K-52 Automated Root Cause Analysis（自動原因推定）
- K-53 Human-in-the-Loop Policy Simulator（ポリシー事前検証）
- K-54 Portfolio Firewall（資産防火壁）
- K-55 “Kill-Switch Accounting” Mode（緊急会計モード）
- K-56 Full Reprocessing Pipeline（全期間再処理）
- K-57 “Shadow Book” Parallel Ledger（影台帳）
- K-58 Capacity-Aware Treasury Autopilot（資金自動操縦）
- K-59 External Audit Export Packs（監査提出パック）
- K-60 Secrets-Minimizing Proof（秘密最小化の証明）

---

## 12. TODO（不足の明示：推測で増やさない）
- TODO: LedgerEvent 各event_typeのpayload schemaを、schema_versionとして固定。
- TODO: calc_versionの管理方法（命名、互換性、ロールバック境界）を契約化。
- TODO: Determinism（K-21）における「同イベント列」定義（並び順・同時刻・重複・再取得時の扱い）。
- TODO: Consistency Model（K-46）の「確定値/暫定値」の境界・収束条件・例外時処理の詳細。
- TODO: SLO（K-43）指標、予算消費ルール、Degraded判定の一覧。
- TODO: Transfer Request / Approval の状態遷移・timelock仕様・防火壁ルール（K-54）一覧。
- TODO: Explain（K-11/K-51）のグラフ構造（ノード/エッジ種別）と因果候補生成の入出力契約。
- TODO: 監査提出パック（K-59）の出力フォーマット（CSV群、ハッシュ、リンク、証明）詳細。
- TODO: Shadow Book（K-57）の「別実装/別計算方式」定義と差分許容範囲。

---
