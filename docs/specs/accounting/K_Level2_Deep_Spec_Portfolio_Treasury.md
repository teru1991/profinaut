# Level 2 Deep Spec（K：Portfolio / Treasury｜Deep Spec 1ファイル）

> Non-negotiable + Canonical Model/Contract + Behavior/Tests が入力に揃っているため、Level 2 も整理出力します（新規仕様は作らず、未記載は TODO）。

## 0. Deep Spec メタ
- 対象: K-01〜K-60
- 前提原則: Idempotency / Append-only / Explainability / 三重時刻 / Finality / schema_version+calc_version / 改ざん検知 / 変更管理 / 秘密最小化
- 基本データ:
  - Append-only LedgerEvent + Materialized States

---

## 1. 契約（Contract）整理
### 1.1 LedgerEvent Header Contract
- フィールド一覧: 5.1.1参照（event_id, source, source_event_id, …, redaction_level）
- 不変条件:
  - source_event_id を用いた冪等（重複排除）
  - prev_event_hash / event_hash による改ざん検知チェーン（K-37）
  - 三重時刻の品質反映（K-49, QualityState）
- TODO: 署名/ハッシュ方式、ハッシュ対象範囲、チェーン分割（partition）単位（K-20）を固定。

### 1.2 Event Type Taxonomy Contract
- 種別一覧: 5.1.2参照
- TODO: 各イベントの必須payload・通貨/銘柄/数量の単位体系・丸め規則・禁止値（NaN/負数等）を固定。

### 1.3 Materialized State Contract
- 状態一覧: 5.2参照
- TODO: State更新ルール（イベント→状態への写像）、pending_deltaやreorg_risk等の計算規則、集計窓を固定。

---

## 2. 機能（Capabilities）Deep Spec テンプレ
各K-xxについて、入力文書で明示されている範囲のみ確定し、詳細は TODO とする。

### 2.1 台帳中核（K-01〜K-11）
#### K-01 Ingestion Adapters
- 目的: 取引所/チェーン/各ソースからのイベント取り込み（LedgerEvent化の入口）。
- 入力: I Execution, H Market Data,（必要に応じて）オンチェーン等
- 出力: LedgerEvent（Append-only）
- TODO: アダプタ種別、正規化規約、再取得/重複/順序保証、リトライ方針。

#### K-02 Ledger Writer
- 目的: LedgerEventをAppend-onlyで永続化、冪等/改ざん検知チェーンを維持。
- 関連: Idempotency, Append-only, K-37
- TODO: 保管ストア、partition設計（K-20）、書き込み原子性、スナップショット扱い。

#### K-03 State Builder
- 目的: イベント列からMaterialized Statesを構築（スナップショットはキャッシュ）。
- 必達: イベントソーシング再構築、再現性固定（schema_version+calc_version）
- TODO: 増分更新/再構築の境界、遅延イベント/訂正/reorg処理（K-46/K-28）。

#### K-04 Valuation Engine
- 目的: Mark-to-Market評価、基軸通貨換算（JPY/USD等）。
- 依存: H Market Data
- 出力: NavState/ExposureState 等のbase_ccy_value
- TODO: 価格欠損時の扱い（Degraded）、価格鮮度（price_staleness）計算。

#### K-05 PnL Attribution
- 目的: PnL（実現/含み）と費用（fee/funding/interest/gas/other）を分解し帰属。
- 出力: PnLState（+ tax_lots）
- TODO: 原価法/ロット計算詳細（K-12）、按分規則（K-32）。

#### K-06 Exposure / Concentration
- 目的: 露出・集中の算出。
- 出力: ExposureState
- TODO: gross/net/delta定義、流動性キャパ（K-27）との結合。

#### K-07 NAV / Drawdown
- 目的: 純資産（NAV）・ドローダウン算出。
- 出力: NavState（+ kill_switch_nav）
- TODO: DDの算出窓、ベンチマーク、緊急会計（K-55）での保守化規則。

#### K-08 Allocation Ledger
- 目的: Allocation Ledger（按分・帰属の基盤の一部）。
- TODO: 何をAllocation単位として保持するか（bot/戦略/口座/ブック等）。

#### K-09 Treasury Policy
- 目的: 出金・内部振替など移動に関するポリシー。
- 必達: 移動統制（ポリシー＋承認WF＋timelock＋ガードレール）
- TODO: ポリシールール一覧、絶対禁止ルールとの関係（K-54）。

#### K-10 Reconciliation
- 目的: 実態（取引所/チェーン）と台帳の差分検知→修復導線。
- イベント: ReconciliationDiff, ReconciliationRepairAttempted
- TODO: 自動修復の範囲（K-22）、隔離条件、手動介入フロー。

#### K-11 Audit & Explain
- 目的: 任意時点の残高/損益/NAVの根拠イベントへ遡及し、因果候補提示。
- I/F: Explain, Compliance
- TODO: 説明グラフ（K-51）との統合、因果候補生成（K-52）の出力契約。

### 2.2 完全性・オンチェーン（K-12〜K-25）
- K-12 Tax Lot / Cost Basis：PnLのロット原価（PnLState.tax_lots）。
  - TODO: ロット生成・消費・丸め境界。
- K-13 Asset Events：Airdrop/TokenSwap/Delist/SplitMerge等を台帳化。
  - TODO: 各イベントの資産増減・評価・税務区分（W連携）の境界。
- K-14 Margin / Collateral / Borrow-Lend：証拠金/借入貸付の台帳・状態。
  - TODO: 取引所固有表現の正規化。
- K-15 Funding/Interest 精密化：FundingPayment/InterestAccrualの精密取り扱い。
  - TODO: 発生タイミング、未確定・訂正時の調整。
- K-16 Fee Classification 完全化：FeeCharged/Rebate等の分類。
  - TODO: fee種別の体系（maker/taker/VIP/手数料通貨等）。
- K-17 Approval Workflow：TransferRequestの承認/却下/期限切れ等をイベント化。
  - TODO: 承認者/権限（S連携）、timelock要件。
- K-18 Withdrawal Safety Enhancements：出金安全性の強化。
  - TODO: 具体ガードレール、誤操作防止（K-40）との境界。
- K-19 Treasury Planner：資金繰り計画。
  - TODO: 計画の入力（制約/流動性/移動時間等）と出力。
- K-20 Multi-Book / Partition：ブック分割/区画。
  - TODO: partitionキー、イベントチェーン（K-37）との整合。
- K-21 Replay & Determinism：同イベント列→同状態保証。
  - TODO: 並び順・再取得差分の扱い（K-46）。
- K-22 Reconciliation 自動修復高度化：差分→自動修復。
  - TODO: 自動修復の安全制約、監査記録。
- K-23 Data Quality / Confidence Score：品質/信頼度スコアとSLO。
  - TODO: スコア算出式、degraded_reasons語彙。
- K-24 On-chain Portfolio：オンチェーン資産の組み込み。
  - TODO: finality/reorgの取り扱い詳細（K-46/K-28）。
- K-25 Bridge / Wrapped Asset 正規化：ブリッジ/ラップ資産の正規化。
  - TODO: 正規化ルール（資産同一性、換算、イベント表現）。

### 2.3 長期運用（K-26〜K-36）
- K-26 Portfolio Constraints Engine：制約違反の検知・状態化。
  - TODO: ルールID体系、重大度、影響推定の算定。
- K-27 Liquidity & Capacity：流動性/キャパ算定。
  - TODO: Market Data依存、Capacity推定法。
- K-28 Settlement & Pending State：pending/confirmed/finalized と pending比率。
  - TODO: pending_delta更新規則、収束条件（K-46）。
- K-29 Fee Tier / VIP / Rebates Tracker：VIPティア・リベート追跡。
  - TODO: ティア情報の入力源、期間、証跡。
- K-30 Multi-Currency Cash Management：多通貨現金管理。
  - TODO: 基軸通貨換算・FX扱い（H連携）。
- K-31 Internal Ledger for Simulated Funds：シミュレーション資金の内部台帳。
  - TODO: 実資金との隔離、book/partition設計。
- K-32 Strategy Attribution（按分・帰属）：bot/戦略への按分帰属。
  - TODO: 按分キー、例外（共有注文/共通資金）。
- K-33 Stress / Scenario Engine：ストレス/シナリオ結果。
  - TODO: シナリオ定義、結果の台帳化（StressScenarioRun）。
- K-34 Insurance / Bankruptcy / Exchange Failure Modeling：破綻等モデル。
  - TODO: モデル入力/出力、どの意思決定へ供給するか（J連携）。
- K-35 Data Retention & Cold Storage：保管/コールドストレージ。
  - TODO: 保管期間、アーカイブ形式（RetentionArchived）。
- K-36 DR / Backup / Restore Drill：DR/復旧訓練。
  - TODO: 訓練頻度、成功基準（RestoreDrillResult）。

### 2.4 監査・安全・変更管理（K-37〜K-45）
- K-37 Tamper-Evident Ledger：ハッシュ連鎖で欠落/改ざん検知。
  - TODO: 監査提出での検証手順。
- K-38 Dual-Source Verification：二重系検算。
  - TODO: 第二系の定義、差分許容範囲。
- K-39 Fraud / Anomaly Detection：不正・異常検知（AnomalyDetected）。
  - TODO: 重大度、SAFE_MODE要請条件（E連携）。
- K-40 Action Guardrails：誤操作防止。
  - TODO: ガードレール一覧、適用ポイント（Transfer Request等）。
- K-41 Performance & Scalability：性能・スケール。
  - TODO: 想定負荷、SLO項目、ベンチ条件。
- K-42 Privacy / Redaction：秘匿・マスキング。
  - TODO: redaction_level設計、S Dashboard/Access（S連携）での表示制御。
- K-43 SLA / SLO Budget：品質予算。
  - TODO: 予算消費ロジック、違反時の運用（SLOViolated）。
- K-44 Change Management：変更管理（提案/承認/適用/ロールバック）。
  - TODO: 変更対象（制約/Calc Version等）の分類と承認フロー。
- K-45 Legal/Compliance Hooks：準拠フック。
  - TODO: 具体的な外部要件（法令/規程）とのマッピング（新規要件は追加しない）。

### 2.5 究極オプション統合（K-46〜K-60）
- K-46 Byzantine/Consistency Model：嘘/遅延/訂正/reorg前提で確定/暫定境界を固定。
  - TODO: 境界・収束条件・例外時扱い。
- K-47 Multi-Region / Air-Gapped Audit Copy：監査用コピー複製。
  - TODO: 複製先、検証手順、復旧手順。
- K-48 Hardware Attestation Hooks：TPM等の拡張点。
  - TODO: attestation情報の格納・提示形式。
- K-49 Deterministic Time Discipline：NTPずれ検知、単調増加、隔離。
  - TODO: time_qualityの算出と隔離基準。
- K-50 Contract/Schema Proof Artifacts：calc_versionが契約に一致する証明成果物。
  - TODO: 成果物フォーマット、保存先、監査提示の導線。
- K-51 Explain Anything Graph：イベント・価格・料率・制約等をグラフ接続。
  - TODO: ノード/エッジ仕様、クエリI/F。
- K-52 Automated Root Cause Analysis：原因候補ランキングと次アクション提示。
  - TODO: 入力（差分/Confidence低下等）と出力契約。
- K-53 Human-in-the-Loop Policy Simulator：変更前影響シミュレーション。
  - TODO: 対象（制約/承認/ガードレール）、評価指標。
- K-54 Portfolio Firewall：絶対禁止ルールを強制、突破不可。
  - TODO: ルール一覧、例外無/有の扱い。
- K-55 Kill-Switch Accounting Mode：SAFE_MODE時の保守的評価でNAV/Exposure即時計算。
  - TODO: worst-case評価規則、切替条件（E連携）。
- K-56 Full Reprocessing Pipeline：calc_version更新時の全期間再計算（監査化/ロールバック）。
  - TODO: 進捗/差分/検証/ロールバックの記録形式。
- K-57 Shadow Book Parallel Ledger：別方式で並走し実装バグ検知。
  - TODO: 別方式の定義、差分検知→原因追跡の導線。
- K-58 Capacity-Aware Treasury Autopilot：資金配置提案（半自動）、実行は承認必須。
  - TODO: 提案入力（流動性/制約/リスク/手数料/移動時間）と出力。
- K-59 External Audit Export Packs：CSV、ハッシュ、証跡リンク集、証明をワンクリ生成。
  - TODO: pack構成、再生成の決定性、検証手順。
- K-60 Secrets-Minimizing Proof：秘密を触れずに検証、秘密がログに出ない監査自動化。
  - TODO: 検査項目、redaction＋検査の具体。

---

## 3. テスト（Behavior/Tests）をCapabilityへ割り当て（整理のみ）
- 冪等性（重複計上なし）→ K-01/K-02/K-03
- 再現性（同イベント列→同状態）→ K-03/K-21/K-56
- 保存則（PnL分解合計一致）→ K-05/K-12/K-15/K-16
- 照合（差分記録と修復導線）→ K-10/K-22/K-38/K-57
- 強操作（ガードレール＋承認＋防火壁）→ K-17/K-18/K-40/K-54
- 改ざん検知（ハッシュ連鎖断裂検知）→ K-37/K-59
- 性能（SLO内）→ K-41/K-43
- DR（定期リストア成功）→ K-36

> TODO: 各テストの具体的な入力データセット、期待値、許容誤差、失敗時の診断手順（Runbook連携）を別途固定（本入力には未記載）。
