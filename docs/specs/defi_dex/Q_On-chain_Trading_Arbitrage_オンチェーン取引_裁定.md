# Q — On-chain Trading / Arbitrage（オンチェーン取引・裁定）
Level 1 SSOT Outline（SSOT v2.0 / 入力変換）

## 0. SSOTメタ
- ドメイン文字: **Q**
- 対象: オンチェーン（主にEVM、将来拡張）における **DEX取引・ルーティング・裁定** を「安全に常時稼働」させるためのドメイン :contentReference[oaicite:0]{index=0}
- 最優先KPI: **事故ゼロ（鍵漏洩/資金拘束/再現不能/予期せぬ損失）** :contentReference[oaicite:1]{index=1}
- 基本姿勢: **Unknown/Degradedでは取引しない（Fail-Closed）**。利益が出ていても安全KPI悪化時は停止 :contentReference[oaicite:2]{index=2}

## 0.1 方針（機能凍結と変更統制）
- Qの機能要件は **SSOT v2.0で凍結**（以後は追加ではなく実装/統合の品質向上に集中） :contentReference[oaicite:3]{index=3}
- 以後の変更は **Security Review Gate（Q-57）** と **Change Management（Q-61）** を必須 :contentReference[oaicite:4]{index=4}
- 例外: 欠陥修正・安全性向上・観測定義の明確化のみ :contentReference[oaicite:5]{index=5}

---

## 1. Non-negotiable（目的と到達点）
- Qは、オンチェーンにおける DEX取引・ルーティング・裁定を **安全に常時稼働** させる :contentReference[oaicite:6]{index=6}
- 必須要素 :contentReference[oaicite:7]{index=7}
  - マルチチェーン接続、RPC冗長化、reorg/finality管理
  - DEXルーティング、MEV対策、ガス最適化、フロントラン検知
  - ブリッジ/跨ぎの遅延・失敗・資金拘束リスク管理
  - 署名鍵運用（用途分離・強操作統制・承認・監査）
- 安全原則
  - Fail-Closedを強制（Unknown/Degradedでは取引しない） :contentReference[oaicite:8]{index=8}
  - 安全KPIが悪化したら利益が出ていても停止 :contentReference[oaicite:9]{index=9}

---

## 2. スコープ境界
### 2.1 Qがやること
- DEX取引の安全な構築・事前検証・送信・結果確定 :contentReference[oaicite:10]{index=10}
- 機会検出・保守評価・実行（アトミック実行前提） :contentReference[oaicite:11]{index=11}
- reorg/RPC不一致/MEV/外部障害を前提にした縮退・停止 :contentReference[oaicite:12]{index=12}
- Hot reload / Dry-run / Shadow / 段階ロールアウト / rollback :contentReference[oaicite:13]{index=13}
- マルチアカウント運用、救出（Rescue Ops）、二重実行防止、改ざん検知ログ :contentReference[oaicite:14]{index=14}
- 経済的安全性（不変条件）と攻撃シミュレーション、検証用証拠生成 :contentReference[oaicite:15]{index=15}
- 人的ミス耐性（二人承認、break-glass、学習モード、事故テンプレ） :contentReference[oaicite:16]{index=16}
- Telemetry定義SSOT（観測の意味を固定し、運用崩壊を防ぐ） :contentReference[oaicite:17]{index=17}

### 2.2 Qが接続すること（委譲可能）
- B（Secrets）/ E（Safety）/ J（Risk Gate）/ K（Treasury）/ O（Audit）/ C・D（Obs/Runbook）/ T（Testing/Chaos） :contentReference[oaicite:18]{index=18}

---

## 3. 必須アーキテクチャ原則（Qの憲法）
※ v1.9の原則を維持 :contentReference[oaicite:19]{index=19}
- Fail-Closed :contentReference[oaicite:20]{index=20}
- 二段階確定（Included≠確定） :contentReference[oaicite:21]{index=21}
- 三点照合（receipt+logs+state） :contentReference[oaicite:22]{index=22}
- 資源上限（時間/分岐/候補/メモリ/API予算） :contentReference[oaicite:23]{index=23}
- 用途分離（trade/approve/bridge&withdraw） :contentReference[oaicite:24]{index=24}
- データ鮮度SLO :contentReference[oaicite:25]{index=25}
- 再現可能性（opportunity_id） :contentReference[oaicite:26]{index=26}
- 段階ロールアウト＋即rollback :contentReference[oaicite:27]{index=27}
- 二重実行防止（idempotency） :contentReference[oaicite:28]{index=28}
- 最小介入（強操作のみ人間） :contentReference[oaicite:29]{index=29}
- 決定性（同一入力→同一判断/TxPlan） :contentReference[oaicite:30]{index=30}
- 正本時刻（canonical time） :contentReference[oaicite:31]{index=31}
- 順序性（event sequencing） :contentReference[oaicite:32]{index=32}
- 設定ピン留め（config_version） :contentReference[oaicite:33]{index=33}
- 経済的不変条件（破ったら自動停止） :contentReference[oaicite:34]{index=34}
- 人的ミス耐性（Two-Person Rule / Safe Defaults） :contentReference[oaicite:35]{index=35}

---

## 4. Canonical Model（Core Types SSOT）
※ v1.9を維持 :contentReference[oaicite:36]{index=36}
- OpportunityId, Sequence, ConfigVersion :contentReference[oaicite:37]{index=37}
- ChainRef / DexRef / TokenRef / PoolRef / BridgeRef :contentReference[oaicite:38]{index=38}
- TokenClass :contentReference[oaicite:39]{index=39}
- SnapshotRef :contentReference[oaicite:40]{index=40}
- Quote, Opportunity, TxPlan, TxEnvelope :contentReference[oaicite:41]{index=41}
- TradeRecord, SafetyContext, StrongOp :contentReference[oaicite:42]{index=42}
- TODO: 各型のフィールド定義・制約（例: OpportunityIdの生成規約、TxEnvelopeの署名/送信形態、SnapshotRefの内容）

---

## 5. Canonical Contract（Event Contract SSOT）
※ v1.9を維持、**opportunity_id / sequence / config_version 必須** :contentReference[oaicite:43]{index=43}
- Opportunity〜Tx〜Reorg〜Reconciled :contentReference[oaicite:44]{index=44}
- SafetyModeChanged / RescueInvoked :contentReference[oaicite:45]{index=45}
- StrongOp（提案→承認→実行/中止） :contentReference[oaicite:46]{index=46}
- ConfigChange（提案→適用→rollback） :contentReference[oaicite:47]{index=47}
- PostmortemGenerated :contentReference[oaicite:48]{index=48}
- TODO: 各イベントのスキーマ（必須/任意フィールド、状態遷移、重複時の冪等処理、監査参照キー）

---

## 6. Failure Taxonomy（失敗分類）
- v1.9を維持（理由コード必須） :contentReference[oaicite:49]{index=49}
- TODO: 理由コード体系（分類軸、再試行可否、Fail-Closed判定、Runbook/監査へのリンク）

---

## 7. State Machines（状態機械）
- v1.9を維持（Opportunity / Tx / StrongOp） :contentReference[oaicite:50]{index=50}
- TODO: 各状態・遷移表、ガード条件、タイムアウト、reorg時の差戻し/再照合、idempotencyキー

---

## 8. Telemetry SSOT（Q-76〜Q-81）
- 位置づけ: Q固有の「新機能」ではなく、長期運用で壊さないための“運用機能”。C（Observability）にも属するが、Q側SSOTで最低限固定 :contentReference[oaicite:51]{index=51}

### 8.1 Q-76 Metrics Definition SSOT（指標の意味を固定）
- 定義の揺れが事故になるため、式・分母・計上タイミングを固定 :contentReference[oaicite:52]{index=52}
- 対象（例示） :contentReference[oaicite:53]{index=53}
  - opportunity_found_total（found判定地点）
  - opportunity_executed_total（executedをTxSubmittedにするかFinalizedにするか）
  - trade_success_rate（分母：submitted/included/finalized のどれか固定）
  - sim_failure_rate（対象・除外条件）
  - rpc_inconsistency_rate（不一致判定ロジック）
  - snapshot_staleness_seconds（どのsnapshotの年齢か）
  - quote_exec_divergence_bps（基準価格と算出式）
  - mev_suspected_total（疑い/確定の区別）
  - reorg_depth_histogram（測定方法）
  - finality_latency_seconds（included→finalized）
  - pnl_realized_wei / pnl_expected_wei / pnl_worst_wei（定義統一）
- TODO: 各メトリクスの正式仕様（式、分母、除外条件、観測点、サンプルイベント）

### 8.2 Q-77 Cardinality Budget（ラベル爆発防止）
- opportunity_id をメトリクスラベルに使わない（ログ/トレースへ） :contentReference[oaicite:54]{index=54}
- chain/dex/token 等は上位Nのみ（残りは other） :contentReference[oaicite:55]{index=55}
- error_code は許可、raw revert reason はラベル禁止 :contentReference[oaicite:56]{index=56}
- TODO: 上位Nの定義、other化ルール、許可ラベル一覧（allowlist）

### 8.3 Q-78 Correlation IDs & Structured Logging（相関ID統一）
- trace_id と opportunity_id の紐づけ規約固定 :contentReference[oaicite:57]{index=57}
- ログは構造化、イベントは sequence と snapshot_ref 必須 :contentReference[oaicite:58]{index=58}
- O（監査）とC（観測）の相互参照キー固定 :contentReference[oaicite:59]{index=59}
- TODO: 相関IDの命名/伝播、ログスキーマ、監査イベントとのJOINキー

### 8.4 Q-79 Sampling Policy（サンプリング規約）
- 全件保持: 強操作、モード変更、GateRejected、TxFinalized、TxReorged :contentReference[oaicite:60]{index=60}
- サンプリング対象: OpportunityDetected（高頻度） :contentReference[oaicite:61]{index=61}
- 失敗時はサンプリング率を自動で引き上げ :contentReference[oaicite:62]{index=62}
- TODO: 既定率、引き上げ条件、期間、上限、監査/コストの扱い

### 8.5 Q-80 Data Quality Checks for Telemetry（計測健全性）
- “あり得ない値” を検知したらアラート、必要ならSAFE_MODE :contentReference[oaicite:63]{index=63}
- 例: finality_latency < 0 / success_rate > 1 / pnl桁違い / divergence異常スパイク :contentReference[oaicite:64]{index=64}
- 「観測が壊れた」も停止理由に含める（Fail-Closed） :contentReference[oaicite:65]{index=65}
- TODO: チェック一覧の完全版、閾値、SAFE_MODE遷移、復帰条件

### 8.6 Q-81 Telemetry Versioning & Rollout（観測互換性）
- メトリクス名/ラベル/意味をバージョニング :contentReference[oaicite:66]{index=66}
- 破壊的変更は段階適用＋ダッシュボード同時更新＋rollback手順必須 :contentReference[oaicite:67]{index=67}
- TODO: バージョン付与方式（semantic等）、互換性ルール、移行期間、ロールバック手順テンプレ

---

## 9. Capabilities（凍結対象の機能カタログ）
- **Q-01〜Q-75（v1.9）をそのまま凍結**。以後は機能追加ではなく実装・テスト・品質改善として扱う :contentReference[oaicite:68]{index=68}
- TODO: Q-01〜Q-75の各機能説明（v1.9の本文参照が必要）

---

## 10. Done Definition（DoD / 完成条件）
※ v1.9維持＋Telemetry分追加 :contentReference[oaicite:69]{index=69}
- 不変条件を破らない（worst_case_profit等） :contentReference[oaicite:70]{index=70}
- Fail-Closedが徹底される :contentReference[oaicite:71]{index=71}
- reorgでも破綻しない（二段階確定＋三点照合＋差戻し） :contentReference[oaicite:72]{index=72}
- 二人承認・監査証跡（改ざん検知）・再現性（決定性＋config pin＋replay） :contentReference[oaicite:73]{index=73}
- 重複配信でも収束（状態機械＋idempotency） :contentReference[oaicite:74]{index=74}
- Telemetry定義が揺れず、cardinality爆発が起きず、相関IDで追跡できる :contentReference[oaicite:75]{index=75}
- “観測が壊れた” を検知し、必要なら自動停止できる :contentReference[oaicite:76]{index=76}

---

## 11. Capability Index（ID一覧）
> 要求: 番号/ID（Q-xx 等）は必ず保持してここに集約する。

### 11.1 Governance / Gates
- Q-57 — Security Review Gate（変更に必須） :contentReference[oaicite:77]{index=77}
- Q-61 — Change Management（変更に必須） :contentReference[oaicite:78]{index=78}

### 11.2 Frozen Capabilities（v1.9凍結）
- Q-01 — TODO: v1.9参照
- Q-02 — TODO: v1.9参照
- Q-03 — TODO: v1.9参照
- Q-04 — TODO: v1.9参照
- Q-05 — TODO: v1.9参照
- Q-06 — TODO: v1.9参照
- Q-07 — TODO: v1.9参照
- Q-08 — TODO: v1.9参照
- Q-09 — TODO: v1.9参照
- Q-10 — TODO: v1.9参照
- Q-11 — TODO: v1.9参照
- Q-12 — TODO: v1.9参照
- Q-13 — TODO: v1.9参照
- Q-14 — TODO: v1.9参照
- Q-15 — TODO: v1.9参照
- Q-16 — TODO: v1.9参照
- Q-17 — TODO: v1.9参照
- Q-18 — TODO: v1.9参照
- Q-19 — TODO: v1.9参照
- Q-20 — TODO: v1.9参照
- Q-21 — TODO: v1.9参照
- Q-22 — TODO: v1.9参照
- Q-23 — TODO: v1.9参照
- Q-24 — TODO: v1.9参照
- Q-25 — TODO: v1.9参照
- Q-26 — TODO: v1.9参照
- Q-27 — TODO: v1.9参照
- Q-28 — TODO: v1.9参照
- Q-29 — TODO: v1.9参照
- Q-30 — TODO: v1.9参照
- Q-31 — TODO: v1.9参照
- Q-32 — TODO: v1.9参照
- Q-33 — TODO: v1.9参照
- Q-34 — TODO: v1.9参照
- Q-35 — TODO: v1.9参照
- Q-36 — TODO: v1.9参照
- Q-37 — TODO: v1.9参照
- Q-38 — TODO: v1.9参照
- Q-39 — TODO: v1.9参照
- Q-40 — TODO: v1.9参照
- Q-41 — TODO: v1.9参照
- Q-42 — TODO: v1.9参照
- Q-43 — TODO: v1.9参照
- Q-44 — TODO: v1.9参照
- Q-45 — TODO: v1.9参照
- Q-46 — TODO: v1.9参照
- Q-47 — TODO: v1.9参照
- Q-48 — TODO: v1.9参照
- Q-49 — TODO: v1.9参照
- Q-50 — TODO: v1.9参照
- Q-51 — TODO: v1.9参照
- Q-52 — TODO: v1.9参照
- Q-53 — TODO: v1.9参照
- Q-54 — TODO: v1.9参照
- Q-55 — TODO: v1.9参照
- Q-56 — TODO: v1.9参照
- Q-57 — Security Review Gate（変更に必須） :contentReference[oaicite:79]{index=79}
- Q-58 — TODO: v1.9参照
- Q-59 — TODO: v1.9参照
- Q-60 — TODO: v1.9参照
- Q-61 — Change Management（変更に必須） :contentReference[oaicite:80]{index=80}
- Q-62 — TODO: v1.9参照
- Q-63 — TODO: v1.9参照
- Q-64 — TODO: v1.9参照
- Q-65 — TODO: v1.9参照
- Q-66 — TODO: v1.9参照
- Q-67 — TODO: v1.9参照
- Q-68 — TODO: v1.9参照
- Q-69 — TODO: v1.9参照
- Q-70 — TODO: v1.9参照
- Q-71 — TODO: v1.9参照
- Q-72 — TODO: v1.9参照
- Q-73 — TODO: v1.9参照
- Q-74 — TODO: v1.9参照
- Q-75 — TODO: v1.9参照

### 11.3 Telemetry Capabilities（SSOT v2.0で固定）
- Q-76 — Metrics Definition SSOT（指標定義の固定） :contentReference[oaicite:81]{index=81}
- Q-77 — Cardinality Budget（ラベル爆発防止） :contentReference[oaicite:82]{index=82}
- Q-78 — Correlation IDs & Structured Logging（相関ID統一） :contentReference[oaicite:83]{index=83}
- Q-79 — Sampling Policy（サンプリング規約） :contentReference[oaicite:84]{index=84}
- Q-80 — Data Quality Checks for Telemetry（計測健全性チェック） :contentReference[oaicite:85]{index=85}
- Q-81 — Telemetry Versioning & Rollout（観測互換性） :contentReference[oaicite:86]{index=86}
