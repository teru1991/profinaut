# N: Experiment / Backtest / Forward — Level 1 SSOT Outline（統一フォーマット）
出典：実装目標機能詳細設計 SSOT（清書・統合最終網羅版 v2.9）:contentReference[oaicite:0]{index=0}

---

## 0. Overview
### 0.1 ドメイン
- ドメイン文字：**N**
- 対象：**実験OS（Experiment OS）**として、検証〜運用〜監査までを一体化し、**戦略コードを変えずに** mode（backtest / paper / forward / shadow / canary / live）を切替可能にする。:contentReference[oaicite:1]{index=1}

### 0.2 用語（この文書内での最小定義）
- Mode switch：戦略ABIを保ったまま、実行モード（backtest/paper/forward/shadow/canary/live）を切替えること。:contentReference[oaicite:2]{index=2}
- Registry/Contracts/Trace：ExperimentSpec / RunRecord / ArtifactsManifest 等の台帳・契約・導線固定。:contentReference[oaicite:3]{index=3}
- Comparable Contract：比較可能性の契約（STRICT/SOFT/REJECT + Misuse Guard）。:contentReference[oaicite:4]{index=4}

---

## 1. Non-negotiable
### 1.1 目的と不変ゴール（Non-Negotiable）
- Nは「戦略コードを変えずに」 backtest / paper / forward / shadow / canary / live を切替可能にし、検証〜運用〜監査までを一体化した実験OSとして成立させる。:contentReference[oaicite:5]{index=5}

### 1.2 必達保証（v2.9）
以下を**必達**とする（詳細は各セクションへマッピング）。:contentReference[oaicite:6]{index=6}
1) 同一ABI Mode Switch（戦略側条件分岐禁止）  
2) 実運用級シム（slippage/fee/板薄/部分約定/遅延/欠損/順序乱れ/429 等）  
3) Forward＝Live同等障害再現＋差分原因の自動分析  
4) 台帳SSOT（dataset/code/params/scenario/env/model/failure_profile/run…の一貫追跡）  
5) 再現性（snapshot+time_window+seed+env+model+semantics+numericで同一出力）  
6) 比較の正しさ（Comparable Contract + Misuse Guard + Compat Gate）  
7) ガバナンス（Run Policy + CI Gate + 禁止依存の物理隔離）  
8) Live最終安全弁（Shadow/Canary + invariants + progressive exposure）  
9) 妥当性証明（Verification + Reality + Stress + Metamorphic + 分布一致）  
10) 時間整合（Time Window / Late-arrival / Time Unification / Drift）  
11) 目的関数SSOT（Objective + Risk Budget：評価と運用で一致）  
12) データ爆発対策（Tiered Storage / Sampling / Compaction / CAS / 差分圧縮 / log dedup）  
13) フォレンジック（Repro Bundle / Diff / Bisect / Time-travel）  
14) 昇格/降格基準SSOT（Promotion/Demotion + 多目的昇格 + regime coverage）  
15) Cross-Venue整合（alignment/reconciliation/correlated failures）  
16) TCA（Implementation Shortfall分解＋内訳強制）  
17) 不確実性/頑健性（CI/感度/robustness + 期間安定性）  
18) 因果説明（Attribution + Counterfactual）  
19) 探索台帳/リーク防止（Search Registry + Leakage Guard + 試行統制）  
20) 成果物セキュリティ（Redaction/Access tiering/WORM/監査ログ）  
21) SRE運用品質（SLO/alert/runbook/自己防衛/failover/cost/ops timeline）  
22) Contracts/CI固定（schemas + gates + trace-index）  
23) 実装補助層（n-core/nctl/dev-fast/bench/storage抽象）  
24) UX/分散（wizard/explorer/what-if、CAS、分散決定論スケジューラ、remote sandbox）  
25) マルチ資産/オルタ（カレンダー/IR/ロール/オンチェーン/feature snapshot）  
26) 現実例外（停止/メンテ/板破損復旧、複数口座、入出金、会計照合）  
27) ポートフォリオ運用（複数戦略の資本配分・衝突解決）  
28) オフライン完全再現（offline pack、外部依存なし再生）  
29) ネットワーク/資源注入（輸送層損失/遅延バースト/CPU-IO劣化）  
30) 規制/制限再現（KYC/AML/出金制限/許可禁止）  
31) 手数料ティア進化（出来高で手数料階段変化）  
32) 異常相場注入（flash crash/流動性真空/news shock）  
33) サンドボックス強化（syscall gate/secretless runtime）  
34) データ真正性・改ざん耐性（attestation/lineage検証/多数決統合）  
35) 攻撃・悪用耐性（adversarial market/API abuse/BAN risk）  
36) 状態ハッシュ/乖離検知（deterministic state hashing/divergence alarm）  
37) 可観測性パック（QoE dashboard/data quality heatmap）  
38) 供給側健全性ルーティング（provider scoring/routing + compliance tests）  
39) 仕様差分監視＆半自動追従（API spec diff + semantics auto-suggest）  
40) 戦略品質ゲート（静的解析 + complexity budget）  
41) 汚染ゼロ実行（hermetic env + deterministic cache）  
42) 実験資産化（composition + template library）  
43) 危険操作ハード封止（interlock + blast radius）  
44) 寿命管理（model/dataset/feature TTL + refresh gate）  
45) 外部依存スナップショット（fingerprint + drift detector）  
46) 学習汚染検知強化（label leakage + feature provenance lock）  
47) 選別バイアス抑止（selection bias guard + holdout freeze）  
48) 最後の盾（v2.9追補）：電源断/ディスク破損/秘密監査/評価参照器/操作dry-run 等  

---

## 2. Scope
### 2.1 Nが提供（実装対象）
- Registry/Contracts/Trace：ExperimentSpec / RunRecord / ArtifactsManifest / Contracts / CI Gates / Trace Index :contentReference[oaicite:7]{index=7}
- Data/Time/Scenario/FailureProfile/Drift/Provenance：スナップショット、時間窓、障害注入、品質/ドリフト、由来DAG :contentReference[oaicite:8]{index=8}
- Simulation Core：market replay + execution sim + semantics adapter + models + calibration :contentReference[oaicite:9]{index=9}
- Runner：mode switch、sandbox、scheduler、checkpoint、self-defense、one-click ops :contentReference[oaicite:10]{index=10}
- Evaluation：Comparable、差分原因、Objective/Risk、TCA、Stress、Uncertainty、Attribution、DOE、昇格判定 :contentReference[oaicite:11]{index=11}
- Artifacts：manifest/hash tree/tier/CAS/security/audit package/lite evidence/圧縮 :contentReference[oaicite:12]{index=12}
- SRE：SLO/alerts/runbooks/failover/cost/ops timeline :contentReference[oaicite:13]{index=13}
- Tools：n-core、nctl、dev-fast/dev-replay、bench、doctor/repair/explain :contentReference[oaicite:14]{index=14}
- UX/Distributed：wizard/explorer/what-if、distributed scheduler、remote sandbox :contentReference[oaicite:15]{index=15}
- Multi-asset/Alt-data：calendar/IR/roll/onchain/features :contentReference[oaicite:16]{index=16}
- Real-world exceptions：halt/maintenance/book recovery/accounts/cashflow/reconciliation :contentReference[oaicite:17]{index=17}
- Portfolio ops：ensemble/capital allocation/conflict resolver :contentReference[oaicite:18]{index=18}
- Offline reproducibility：offline pack/time travel :contentReference[oaicite:19]{index=19}
- Physical injection/Compliance/Fee tier/Abnormal market：最終現実層 :contentReference[oaicite:20]{index=20}

### 2.2 参照/連携（境界）
- 連携ドメイン：H（Market Data）、I（Execution：liveのみ）、M（Strategy Runtime）、O（Deterministic Replay）、T（Testing/Chaos）、F（Clock）:contentReference[oaicite:21]{index=21}
- Observability：Prometheus/Loki/Grafana/Alertmanager :contentReference[oaicite:22]{index=22}

---

## 3. Architecture
### 3.1 4レイヤ構造
- Layer A：Registry & Contracts :contentReference[oaicite:23]{index=23}
  - ExperimentSpec / RunRecord / ArtifactsIndex / SearchRegistry
  - Contracts（schema）+ CI Gates + Migration/Compat
  - Trace Index（SSOT導線）
  - Ops timeline（アラート→判断→操作）
- Layer B：Data & Time & Scenario :contentReference[oaicite:24]{index=24}
  - Dataset Catalog / Snapshot / Validator / Quality Meta
  - Time Window Lock / Late-arrival / Time Unification / Drift Detector
  - Replay Consistency Contract / Gap Fill policy
  - Cross-venue alignment/reconciliation / correlated failures
  - Failure profile / multi-source failover / maintenance/halt calendars
  - Drift guard / provenance graph / symbol mapping
  - Timestamp integrity / causality / sequence gap
- Layer C：Simulation Core :contentReference[oaicite:25]{index=25}
  - Market replay（trade/book/ticker/account等）
  - Execution sim（注文/約定/エラー/部分約定/キュー/impact）
  - Semantics Catalog + Adapter（取引所差吸収）
  - Fee tier evolution / maker-taker edge cases
  - Models（fee/slippage/latency/impact/queue/error…プラガブル）
  - Calibration / applicability / break detection / model drift
  - Verification suite + reality check
  - Stress harness（worst-case + invariants）+ abnormal market injectors
  - Metamorphic / stochastic reproducibility（分布一致）
- Layer D：Runner & Execution :contentReference[oaicite:26]{index=26}
  - Mode switch（backtest/paper/forward/shadow/canary/live）
  - Deterministic strategy sandbox + syscall gate + secretless runtime
  - Checkpoint/Resume（互換性）
  - Scheduler（単機→分散決定論）
  - Two-pass / adaptive fidelity / hotspot
  - Self-defense（auto-degrade/circuit/quarantine）+ compliance constraints
  - One-click ops（re-run/re-calibrate/quarantine/rollback）
  - Remote sandbox execution（コンテナ/VM）
  - nctl（doctor/repair/explain）

---

## 4. Canonical Models and Contracts
### 4.1 主要参照ID（SSOT規約）
- experiment_id, run_id（ULID推奨）:contentReference[oaicite:27]{index=27}
- dataset_ref, dataset_snapshot_ref, time_range, time_window_ref :contentReference[oaicite:28]{index=28}
- code_ref, params_ref, scenario_ref, env_ref, model_ref, failure_profile_ref :contentReference[oaicite:29]{index=29}
- evaluation_def_ref, objective_ref, risk_budget_ref :contentReference[oaicite:30]{index=30}
- run_policy_ref, failure_taxonomy_ref, search_ref, trial_id :contentReference[oaicite:31]{index=31}
- tca_def_ref, stress_suite_ref, numeric_policy_ref :contentReference[oaicite:32]{index=32}
- time_unification_ref, replay_contract_ref, gap_policy_ref :contentReference[oaicite:33]{index=33}
- schema_evolution_ref, regression_baseline_ref, performance_budget_ref :contentReference[oaicite:34]{index=34}
- slo_ref, error_budget_ref, metrics_contract_ref, alert_rules_ref, runbook_ref :contentReference[oaicite:35]{index=35}
- failover_policy_ref, cost_budget_ref, ops_action_ref :contentReference[oaicite:36]{index=36}
- redaction_policy_ref, access_tier_ref, audit_package_ref :contentReference[oaicite:37]{index=37}
- 拡張参照（v2.2〜v2.9）：calendar/feature snapshot/attestation/provider health/hard lock/TTL/holdout freeze 等はRunRecordに保存可能。:contentReference[oaicite:38]{index=38}

### 4.2 Comparable Contract（比較契約）
- STRICT_COMPARE：universe/time_range/time_window/snapshot/model/eval_def/objective/risk_budget（必要ならcalendar/feature等も）一致 :contentReference[oaicite:39]{index=39}
- SOFT_COMPARE：不一致だが原因タグ＋根拠が揃う（参考比較）:contentReference[oaicite:40]{index=40}
- COMPARE_REJECT：PnL定義不一致、snapshot不明、time_window不明、lookahead違反など致命的不一致 :contentReference[oaicite:41]{index=41}
- Misuse Guard：SOFT/REJECTはランキング/採用対象から除外（原則）:contentReference[oaicite:42]{index=42}

### 4.3 Contracts（schemas）
- contracts/n/experiment_spec.schema.json :contentReference[oaicite:43]{index=43}
- contracts/n/run_record.schema.json :contentReference[oaicite:44]{index=44}
- contracts/n/artifacts_manifest.schema.json :contentReference[oaicite:45]{index=45}
- contracts/n/metrics_contract.schema.json :contentReference[oaicite:46]{index=46}
- contracts/n/alert_rules.schema.json :contentReference[oaicite:47]{index=47}

### 4.4 CI Gates
- N-CONTRACT-GATE：schema適合 :contentReference[oaicite:48]{index=48}
- N-COMPAT-GATE：migration/旧成果物読取 :contentReference[oaicite:49]{index=49}
- N-REGRESSION-GATE：baseline再実行 + performance budget :contentReference[oaicite:50]{index=50}
- N-OBS-GATE：metrics/alerts/runbook整合 :contentReference[oaicite:51]{index=51}
- 推奨：N-SEC-GATE（redaction/access tier/secret検知）:contentReference[oaicite:52]{index=52}

### 4.5 Trace（SSOT導線）
- docs/trace-index.json：SSOT→contracts→tests→runbooks→dashboards を固定 :contentReference[oaicite:53]{index=53}
- TODO: trace-index.json の実体（キー、参照ルール、更新規約、CIでの検証方法）

---

## 5. Data Models
### 5.1 ExperimentSpec（要点）
- mode、dataset/snapshot/time_window、code/params/scenario/env/model、failure_profile :contentReference[oaicite:54]{index=54}
- eval_def/objective/risk_budget、policy、SRE（SLO/alerts/failover/cost）:contentReference[oaicite:55]{index=55}
- security（redaction/access/audit）、UX/分散（任意）:contentReference[oaicite:56]{index=56}
- multi-asset/feature snapshot/calendar/rule snapshot（任意）:contentReference[oaicite:57]{index=57}
- physical injection/compliance/fee tier/abnormal market（任意）:contentReference[oaicite:58]{index=58}
- TTL/holdout freeze/interlock/blast radius（live/canaryで強制可能）:contentReference[oaicite:59]{index=59}

### 5.2 RunRecord（要点）
- lifecycle、policy decision log、materialized inputs :contentReference[oaicite:60]{index=60}
- compare_contract + 理由、promotion_state + 根拠、failure_class :contentReference[oaicite:61]{index=61}
- artifacts index、metrics summary、integrity（hash root）:contentReference[oaicite:62]{index=62}
- explanation（uncertainty/robustness/attribution/counterfactual）:contentReference[oaicite:63]{index=63}
- SRE（slo status、failover/circuit/quarantine、ops timeline、cost）:contentReference[oaicite:64]{index=64}
- security（access log、audit package manifest、lite evidence）:contentReference[oaicite:65]{index=65}
- drift/bias/stability/reconciliation/leakage/interlock/TTL 等の判定結果 :contentReference[oaicite:66]{index=66}

### 5.3 Artifacts Manifest（必須）
- artifacts一覧、schema versions、hash tree root :contentReference[oaicite:67]{index=67}
- tier/hot-warm-cold、compaction、CASキー、delta encoding情報 :contentReference[oaicite:68]{index=68}
- redaction/access tier、export policy注意書き :contentReference[oaicite:69]{index=69}
- audit package / lite evidence / offline pack の索引 :contentReference[oaicite:70]{index=70}

---

## 6. Runtime / Tools / Operations
### 6.1 実装補助層（n-core / nctl / dev-fast / bench）
- n-core：spec/ids/contracts/artifacts/telemetry/replay/policy を共通化 :contentReference[oaicite:71]{index=71}
- ports&adapters：Runner/Simulator/Evaluator/Adapters I/F先凍結 :contentReference[oaicite:72]{index=72}
- dev-fast/dev-replay：小データでも台帳/成果物形式は本番同一 :contentReference[oaicite:73]{index=73}
- bench + profiling hooks：throughput/latency/IO/ディスク増分 :contentReference[oaicite:74]{index=74}
- storage抽象：local/MinIO/S3統一、命名/パーティションSSOT :contentReference[oaicite:75]{index=75}
- nctl：doctor/repair/explain（診断・修復・比較理由説明・自然言語要約）:contentReference[oaicite:76]{index=76}
- TODO: 各ツールのCLI仕様（サブコマンド、入出力、終了コード、互換性方針）

### 6.2 SRE運用
- SLO + error budget :contentReference[oaicite:77]{index=77}
- metrics contract + alert rules + runbook links :contentReference[oaicite:78]{index=78}
- auto-degrade / circuit breaker / quarantine :contentReference[oaicite:79]{index=79}
- multi-source failover :contentReference[oaicite:80]{index=80}
- cost telemetry/budget（run/search）:contentReference[oaicite:81]{index=81}
- one-click ops（re-run/re-calibrate/quarantine/rollback）:contentReference[oaicite:82]{index=82}
- incident timeline（ops_timeline）:contentReference[oaicite:83]{index=83}
- TODO: SLO定義（指標、閾値、計算窓、対象モード）、Runbook体系（命名/リンク規約）

### 6.3 UX/分散/数学的裏取り
- Run Wizard（条件不足なら進めない）:contentReference[oaicite:84]{index=84}
- Decision Explorer（Diff/Attribution/Counterfactual）:contentReference[oaicite:85]{index=85}
- What-if UI（条件置換で反事実実行）:contentReference[oaicite:86]{index=86}
- Artifact CAS、決定論分散スケジューラ、remote sandbox :contentReference[oaicite:87]{index=87}
- Invariant harness / Metamorphic testing / Stochastic reproducibility :contentReference[oaicite:88]{index=88}
- TODO: 分散決定論の前提（時計/順序/乱数/再試行の取り扱い）と制約一覧

---

## 7. Capabilities（機能一覧：カテゴリ別）
> 注：原文は「N-F100〜」等のレンジで整理。**レンジ内の個別ID定義は原文に無い**ため、ここではレンジをSSOTとして保持し、個別粒度は TODO とする。:contentReference[oaicite:89]{index=89}

### 7.1 コア（台帳/データ/シム/runner/評価/再現性/安全）
- N-F100〜199：Registry/Spec/Run :contentReference[oaicite:90]{index=90}
- N-F200〜299：Dataset管理 :contentReference[oaicite:91]{index=91}
- N-F300〜399：Scenario/注入 + Failure profile :contentReference[oaicite:92]{index=92}
- N-F400〜499：Market Replay :contentReference[oaicite:93]{index=93}
- N-F500〜599：Execution Simulation :contentReference[oaicite:94]{index=94}
- N-F600〜699：Mode Switch :contentReference[oaicite:95]{index=95}
- N-F700〜799：Runner :contentReference[oaicite:96]{index=96}
- N-F800〜899：Evaluation :contentReference[oaicite:97]{index=97}
- N-F900〜999：Repro/Integrity :contentReference[oaicite:98]{index=98}
- N-F1000〜1199：Safety/Ops基礎 :contentReference[oaicite:99]{index=99}
- N-F1100〜1199：昇格/降格 :contentReference[oaicite:100]{index=100}
- TODO: 上記レンジに含まれる**個別機能ID**の列挙（N-F1xx 等の粒度）

### 7.2 ガバナンス/妥当性/運用（v1.2〜v1.7）
- Run Policy / Repro Build / Verification/Reality / Failure taxonomy 等 :contentReference[oaicite:101]{index=101}
- Two-pass / Shadow/Canary / Config/Secret hygiene / Time Window / Objective / Forensics / Storage / Workflow / Promotion 等 :contentReference[oaicite:102]{index=102}
- Cross-venue / TCA / Stress / Sandbox / Misuse / Search / Security / Semantics / Numeric / Replay / Audit / Time / Compat / Uncertainty / Regression / Attribution / Auto-ops / Adaptive / Drift 等 :contentReference[oaicite:103]{index=103}
- SRE（SLO/obs/alerts/failover/cost/self-defense/one-click/audit package）:contentReference[oaicite:104]{index=104}
- TODO: v1.2〜v1.7の**対応IDレンジ**（原文にはIDレンジ明記なし）

### 7.3 v2.1〜v2.8（拡張群）
- v2.1：型安全/順序/DOE/隔離/レポート強化（data quality deep guard 等）:contentReference[oaicite:105]{index=105}
- v2.2：マルチ資産/特徴量/ルール変更/手動介入/長期頑健性 :contentReference[oaicite:106]{index=106}
- v2.3：停止/メンテ/口座/入出金/会計照合/異常検知/ポートフォリオ/オフライン :contentReference[oaicite:107]{index=107}
- v2.4：物理現象/資源劣化/規制制限/手数料階段/圧縮/昇格強化/異常相場 :contentReference[oaicite:108]{index=108}
- v2.5：真正性/攻撃耐性/状態ハッシュ/可観測性 :contentReference[oaicite:109]{index=109}
- v2.6：完全決定論/逆校正/GC/外部変動仮説/Live封止/意思決定同値/因果 :contentReference[oaicite:110]{index=110}
- v2.7：供給側健全性/仕様差分監視/戦略品質/汚染ゼロ/資産化 :contentReference[oaicite:111]{index=111}
- v2.8：ハード封止/寿命管理/外部依存/汚染検知/バイアス抑止 :contentReference[oaicite:112]{index=112}
- TODO: v2.1〜v2.8の各項目を、**どのN-Fレンジ/IDに紐づけるか**（原文では概念列挙のみ）

### 7.4 v2.9 追補（追加機能ID：N-F10100〜）
#### 7.4.1 N-F10100：Live操作の“事前影響予測”
- **N-F10101 Live Action Dry-Run Executor**  
  - live/canaryで行う操作（上限変更/停止/ロールバック等）を実行前に影響予測としてシミュレーションし、危険ならブロック。:contentReference[oaicite:113]{index=113}
- **N-F10102 Policy Explanation & Challenge Workflow**  
  - policy拒否理由を自然言語で説明し、例外要求→承認→記録までをワークフロー化。:contentReference[oaicite:114]{index=114}

#### 7.4.2 N-F10200：電源断・瞬断・破損の再現
- **N-F10201 Power Loss / Sudden Reboot Scenario**  
  - プロセス即死/PC再起動を注入し、再開・整合性・二重発注防止を検証。:contentReference[oaicite:115]{index=115}
- **N-F10202 Disk Corruption / Partial Write Emulator**  
  - 部分書き込み/破損ファイル/壊れたパーティションを再現し、manifest/hash treeで検知→復旧を検証。:contentReference[oaicite:116]{index=116}

#### 7.4.3 N-F10300：秘密の扱いの最終強化
- **N-F10301 One-time Secret Handle**  
  - 秘密を値で保持せず短命ハンドル参照のみ許可（使用回数/期限を強制）。:contentReference[oaicite:117]{index=117}
- **N-F10302 Secret Use Audit Trail**  
  - どのrunがどの秘密IDをいつ使ったかを監査ログに保存（値は保存しない）。:contentReference[oaicite:118]{index=118}

#### 7.4.4 N-F10400：評価の決定論と参照評価器
- **N-F10401 Evaluation Determinism Harness**  
  - 評価処理を別プロセスで決定論実行し、集計誤差・スレッド順序依存を排除。:contentReference[oaicite:119]{index=119}
- **N-F10402 Reference Evaluator（Verified）**  
  - 小規模データで正しさが確認された参照評価器（遅いが正しい）を持ち、差分検証の基準にする。:contentReference[oaicite:120]{index=120}

---

## 8. Execution Flow
### 8.1 規範フロー（v2.9）
- v2.8の実行フローに加え：:contentReference[oaicite:121]{index=121}
  - live/canary操作は dry-run を必須化（N-F10101）
  - 障害注入に 電源断/ディスク破損 を含め、復旧能力を検証（N-F10201/10202）
  - secret利用は one-time handle と 使用監査を強制（N-F10301/10302）
  - 評価は determinism harness と 参照評価器で検証可能（N-F10401/10402）
- TODO: v2.8までのフロー本体（ステップ、入力/出力、失敗時分岐、RunRecordへの記録点）

---

## 9. Definition of Done (DoD)
### 9.1 N v2.9 完成条件（最低限）
- live/canary操作は dry-run を通過しないと実行できない :contentReference[oaicite:122]{index=122}
- power loss / disk corruption の少なくとも一つのシナリオで復旧検証が通る :contentReference[oaicite:123]{index=123}
- secret use audit trail が生成され、値が一切残らない :contentReference[oaicite:124]{index=124}
- 評価決定論（harness）か参照評価器のどちらかが運用可能 :contentReference[oaicite:125]{index=125}

---

## 10. Diff / Drift Tagging
### 10.1 差分原因タグ（拡張）
- POWER_DRIFT（電源断/再起動影響）:contentReference[oaicite:126]{index=126}
- DISK_CORRUPTION_DRIFT（部分書き込み/破損影響）:contentReference[oaicite:127]{index=127}
- SECRET_USAGE_DRIFT（秘密取扱い起因）:contentReference[oaicite:128]{index=128}
- EVAL_DRIFT（評価決定論/参照評価器差分）:contentReference[oaicite:129]{index=129}
- TODO: 既存（v2.8まで）の差分原因タグ全集

---

## 11. Capability Index（ID/契約/ゲート/参照の索引）
### 11.1 機能ID（レンジ）
- N-F100〜199（Registry/Spec/Run）:contentReference[oaicite:130]{index=130}
- N-F200〜299（Dataset管理）:contentReference[oaicite:131]{index=131}
- N-F300〜399（Scenario/注入 + Failure profile）:contentReference[oaicite:132]{index=132}
- N-F400〜499（Market Replay）:contentReference[oaicite:133]{index=133}
- N-F500〜599（Execution Simulation）:contentReference[oaicite:134]{index=134}
- N-F600〜699（Mode Switch）:contentReference[oaicite:135]{index=135}
- N-F700〜799（Runner）:contentReference[oaicite:136]{index=136}
- N-F800〜899（Evaluation）:contentReference[oaicite:137]{index=137}
- N-F900〜999（Repro/Integrity）:contentReference[oaicite:138]{index=138}
- N-F1000〜1199（Safety/Ops基礎）:contentReference[oaicite:139]{index=139}
- N-F1100〜1199（昇格/降格）:contentReference[oaicite:140]{index=140}

### 11.2 機能ID（v2.9 個別）
- N-F10100（Live操作の事前影響予測）:contentReference[oaicite:141]{index=141}
  - N-F10101 / N-F10102 :contentReference[oaicite:142]{index=142}
- N-F10200（電源断・破損の再現）:contentReference[oaicite:143]{index=143}
  - N-F10201 / N-F10202 :contentReference[oaicite:144]{index=144}
- N-F10300（秘密取扱い強化）:contentReference[oaicite:145]{index=145}
  - N-F10301 / N-F10302 :contentReference[oaicite:146]{index=146}
- N-F10400（評価の決定論/参照評価器）:contentReference[oaicite:147]{index=147}
  - N-F10401 / N-F10402 :contentReference[oaicite:148]{index=148}

### 11.3 契約（Schema）
- contracts/n/experiment_spec.schema.json :contentReference[oaicite:149]{index=149}
- contracts/n/run_record.schema.json :contentReference[oaicite:150]{index=150}
- contracts/n/artifacts_manifest.schema.json :contentReference[oaicite:151]{index=151}
- contracts/n/metrics_contract.schema.json :contentReference[oaicite:152]{index=152}
- contracts/n/alert_rules.schema.json :contentReference[oaicite:153]{index=153}

### 11.4 CI Gates（固定）
- N-CONTRACT-GATE / N-COMPAT-GATE / N-REGRESSION-GATE / N-OBS-GATE /（推奨）N-SEC-GATE :contentReference[oaicite:154]{index=154}

### 11.5 Trace / SSOT導線
- docs/trace-index.json :contentReference[oaicite:155]{index=155}

### 11.6 参照ID（主要）
- experiment_id, run_id, dataset_ref, dataset_snapshot_ref, time_range, time_window_ref, code_ref, params_ref, scenario_ref, env_ref, model_ref, failure_profile_ref, evaluation_def_ref, objective_ref, risk_budget_ref, run_policy_ref, failure_taxonomy_ref, search_ref, trial_id, tca_def_ref, stress_suite_ref, numeric_policy_ref, time_unification_ref, replay_contract_ref, gap_policy_ref, schema_evolution_ref, regression_baseline_ref, performance_budget_ref, slo_ref, error_budget_ref, metrics_contract_ref, alert_rules_ref, runbook_ref, failover_policy_ref, cost_budget_ref, ops_action_ref, redaction_policy_ref, access_tier_ref, audit_package_ref :contentReference[oaicite:156]{index=156}

---

## 12. Open TODO（推測で埋めない）
- TODO: N-F100〜1199 の**個別ID**定義（原文はレンジのみ）
- TODO: v1.2〜v2.8 の各機能を、どのN-Fレンジ/IDに紐づけるか（原文は概念列挙中心）
- TODO: docs/trace-index.json の仕様（参照構造、必須キー、更新規約、CIでの検証）
- TODO: “Verification suite / Stress harness / Metamorphic testing” の**テストカタログ**（テストID、入力、期待、メトリクス、失敗時の分類・タグ付け）
- TODO: 分散決定論スケジューラの前提条件（時計、順序、乱数、再試行、I/O決定論）
- TODO: Runbook体系（runbook_ref命名規約、リンク規約、SLO/alert_rulesとの整合ルール）
- TODO: Offline pack / Time-travel debugger の成果物仕様（Artifacts Manifest内の索引設計、WORM/Redactionとの関係）
