# S — Dashboard Product（Complete SSOT v2.0 FINAL）Level 1 SSOT Outline
Source: 「S: Dashboard Product — Complete SSOT v2.0 FINAL（S1〜S205 統合・一括網羅）」:contentReference[oaicite:0]{index=0}

## 1.1 概要
### 1.1.1 目的（Sの責務）
- Sは、運用者が **安全に監視・判断・調査・共有・訓練・改善** を行える「運用UIプロダクト」を提供する。  
- **根拠（Explain）** と **復旧導線** を提示する。  
- **自由レイアウト** と **ウィジェット拡張** により成長できる。:contentReference[oaicite:1]{index=1}

### 1.1.2 境界（Sがしないこと / Non-scope）
- 認証・鍵管理の実体：B
- 収集・保存の実体：H/V
- 実行（発注・取消）の実体：I/E
- リスク判定の実体：J/K
- 監査ログの正本：O/Y
- Sは **表示・統合・安全な導線** を担う（実体は置き換えない）。:contentReference[oaicite:2]{index=2}

### 1.1.3 非交渉要件（Non-negotiable）
1. 安全：危険操作は二段階＋理由＋期限＋権限＋環境＋capability
2. 堅牢：部分障害でも生存、縮退、ウィジェット隔離
3. 高速：差分購読（SSE/WS）＋差分描画＋帯域制御
4. 拡張：自由レイアウト＋ウィジェット基盤＋Registry
5. 外出先：read-only既定＋低帯域＋オフライン耐性
6. 説明可能：原因→根拠→対処→Runbook :contentReference[oaicite:3]{index=3}

---

## 1.2 カテゴリ別Capabilities（見出し保持）

### 1.2.1 PART A：コア（S1〜S12）
- S1 UI Shell / Navigation（ルーティング・テーマ・共通UI・権限ガード・i18n土台）
- S2 Layout Engine（Profile/Workspace/Node、Pinned Zone、編集/保存/復元/競合）
- S3 Widget Runtime（Registry/Instance、SemVer、config schema、隔離、health）
- S4 Data Subscriptions（Snapshot＋SSE/WS、Topicカタログ、Envelope、cursor再同期）
- S5 Explain UI（Explain Card、因果タイムライン、Runbook導線）
- S6 Notifications（P0〜P3、Inbox、dedupe、束ね、クールダウン、導線）
- S7 Ops Console（Danger Zone：理由/期限/二段階/影響範囲/ロールバック、相関ID）
- S8 Mobile UX（device_class別、read-only既定、低帯域、断線復帰）
- S9 Access Control View（RBAC/Capability可視化、監査参照導線）
- S10 Status Board（health/ingest/execution/risk 一枚判断、原因TopN→Explain）
- S11 Search / Drilldown（万能検索、統一ドリルダウン規約）
- S12 UI Reliability（UI計測、縮退、互換性破綻隔離、Feature flags連動）:contentReference[oaicite:4]{index=4}

### 1.2.2 PART B：運用（S13〜S17）
- S13 Incident UX（作成→更新→終結→振り返り、候補生成、対象紐付け）
- S14 Annotation / Ops Notes（注釈付与、検索、共有/個人）
- S15 Export / Share（CSV/JSON、共有リンク（期限/権限））
- S16 Support Bundle UI（収集範囲はY、Sは生成/表示/赤塗り）
- S17 Accessibility（非色依存、高コントラスト、大フォント、緊急プリセット）:contentReference[oaicite:5]{index=5}

### 1.2.3 PART C：拡張・生存性（S18〜S23）
- S18 Multi-tenant境界（personal/shared、Owner/Editor/Viewer、隔離）
- S19 Preferences & Policy（通知/表示/操作/帯域ポリシー、履歴）
- S20 Onboarding（初回ウィザード、次にやるべきこと、デモ）
- S21 Compliance UX（理由/期限/影響範囲、差分ビュー、監査参照）
- S22 Failure-First UI（部分障害表示、古いデータ明示、セーフモード、回復導線）
- S23 Optional Enhancements（A/Bビュー、Playbook Runner、Snapshot Replay）:contentReference[oaicite:6]{index=6}

### 1.2.4 PART D：横断SSOT（S24〜S28）
- S24 Compatibility & Versioning（Layout/Widget/Topic schema SemVer＋migration）
- S25 Contract Tests（schema検証＋最小E2E導線）
- S26 Performance Budget（TTI/イベント上限/縮退条件/低帯域標準）
- S27 UI Observability（ログ/例外/再接続/遅延、trace相関、UX SLI）
- S28 Data Governance in UI（SoT/推定確定/PII/Export列制御）:contentReference[oaicite:7]{index=7}

### 1.2.5 PART E：実戦安全（S29〜S33）
- S29 Safety Simulation / Dry-Run（影響予測・不確実性・自動ブロック・記録）
- S30 Rate-limit / Quota UX（制限可視化・UI抑制・P0優先・Explain連携）
- S31 Time & Clock UX（clock skew・timezone固定・期間境界厳密）
- S32 Human Factors（環境強表示・危険文言統一・誤タップ耐性）
- S33 Kill-Switch UX（結果確認・チェックリスト提示・緊急最小画面）:contentReference[oaicite:8]{index=8}

### 1.2.6 PART F：拡張摩擦ゼロ（S34〜S38）
- S34 Capability Discovery & Feature Flags（自動出し分け・理由Explain・段階配布）
- S35 Permission-aware UX（権限不足でも迷わない導線、三重ゲート）
- S36 Data Lineage Lite（出所/鮮度/確度、関連traceへ）
- S37 Self-Service Diagnostics（接続/認可/429/時計/購読過多、推奨アクション）
- S38 Golden Paths（事故対応の型：Status→Explain→Runbook→Incident→操作→監査）:contentReference[oaicite:9]{index=9}

### 1.2.7 PART G：高速と正しさ（S39〜S43）
- S39 UI Cache & Consistency（stale明示、破棄条件、オフライン）
- S40 Mode Awareness（normal/degraded/incident/emergency）
- S41 Operator Accountability（想定結果入力、実行後レビュー、証跡相関）
- S42 UI Threat Model & Abuse Cases（露出最小化、自動保護、fail-safe）
- S43 Documentation-in-UI（画面内ヘルプ、Runbook/用語集リンク）:contentReference[oaicite:10]{index=10}

### 1.2.8 PART H：変更管理（S44〜S53）
- S44 Source-of-Truth Routing（概念別SoT優先、矛盾時差分/Explain/操作ブロック）
- S45 Error Taxonomy for UI（NET/AUTH/RATE/DATA/COMPAT/SOT/INTERNAL＋表現統一）
- S46 Golden Test Fixtures（代表ケース固定）
- S47 Migration Playbook（layout/widget/policy、ロールバック規約）
- S48 Release & Rollback UX（feature flags/canary、リリースノート、隔離/退避）
- S49 Secure Frontend Coding Standard（安全規約）
- S50 State Management & Concurrency（状態境界、順序、重複排除、ロック/ETag）
- S51 Deterministic Rendering（同入力同描画、時刻注入、ランダム排除）
- S52 UI Performance Playbook（最適化手順、回帰検知）
- S53 Acceptance & Evidence（受入証跡固定）:contentReference[oaicite:11]{index=11}

### 1.2.9 PART I：運用最終補強（S54〜S58）
- S54 Offline-First / 災害運用（Minimal Monitor、更新停止）
- S55 Audit Screenshot & Export（焼き込み＋再現メタ）
- S56 One-Command Recovery（再同期固定手順、購読自動調整）
- S57 Training Mode（デモ＋事故シナリオ、dry-run訓練、本番遮断）
- S58 UX Debt Register（遅い/落ちる/再接続多いの可視化と改善ループ）:contentReference[oaicite:12]{index=12}

### 1.2.10 PART J：純機能拡張（S64〜S115）
- S64 Power User Controls
- S65 Advanced Visualization
- S66 Bot Fleet UX
- S67 Comparison & Regression
- S68 Notification Actions
- S69 Shareable State
- S70 Ops Journal
- S71 What-if & Dry-run+
- S72 Mini Analytics
- S73 Alert Fatigue Advisor
- S74 Custom Dashboard Apps
- S75 Interactive Timeline / Time-Travel
- S76 Correlation Explorer
- S77 Event Diff Viewer
- S78 Rule/Policy Simulator UI
- S79 Smart Drilldown
- S80 Bookmark / Collections
- S81 KPI Scorecards
- S82 Session Recorder
- S83 Collaboration Lite
- S84 Data Labeling UI
- S85 Strategy Library View
- S86 Portfolio / Exposure Map
- S87 Live Collaboration View
- S88 Multi-Account Switcher
- S89 Map/Geo View
- S90 Freshness Heatmap
- S91 Change Radar
- S92 Auto Narrative
- S93 Personalization（提案止まり）
- S94 Widget Marketplace
- S95 Runbook to Widget
- S96 Advanced Alert Triage
- S97 Microstructure Viewer（板/スプレッド等）
- S98 Strategy Parameter Surfacing
- S99 Risk “What’s Next” Assistant（提案）
- S100 Data Provenance Inspector
- S101 Investigation Boards
- S102 Root-Cause Graph View
- S103 Multi-Stream Console
- S104 Snooze by Context
- S105 Smart Layout Auto-Arrange
- S106 Widget Dependency Visualizer
- S107 Live Compare Mode
- S108 Synthetic Monitoring Dashboard
- S109 Operator Load Meter
- S110 Watchlist 2.0
- S111 Macro Event Overlay
- S112 Replay-to-Scenario Builder
- S113 Query-to-Alert
- S114 Personal HUD
- S115 Explain Diff :contentReference[oaicite:13]{index=13}

### 1.2.11 PART K：純機能拡張（上級）（S116〜S130）
- S116 Incident War-Room Board
- S117 Impact Explorer
- S118 Micro-Alert Hooks
- S119 Visual Query Builder 2.0
- S120 State Snapshot Pin
- S121 Risk Envelope View
- S122 Alert Similarity Search
- S123 Confidence Meter
- S124 Multi-Layer Filtering
- S125 Ops Checklist Library
- S126 Drilldown Macros
- S127 Backtest/Forwardtest Monitor
- S128 Regime Dashboard
- S129 Latency Anatomy
- S130 Cost Meter :contentReference[oaicite:14]{index=14}

### 1.2.12 PART L：純機能拡張（最終上級）（S131〜S150）
- S131 Anomaly Storyboard
- S132 Focus Mode
- S133 Signal-to-Noise Dial
- S134 Dependency Health Overlay
- S135 Control Surface
- S136 Edge Alerts
- S137 Strategy Health Cards
- S138 Bot Lifecycle Timeline
- S139 Permission Change Monitor
- S140 Confidence-aware Sorting
- S141 Explain→Checklist Auto-link
- S142 Market Session Awareness
- S143 Liquidity Risk Radar
- S144 Event Replay Sharing
- S145 Alert Escalation Planner
- S146 Postmortem Builder
- S147 Widget Sandbox
- S148 Dashboard Theme Presets
- S149 Cross-Domain Navigator
- S150 Operator Scoreboard :contentReference[oaicite:15]{index=15}

### 1.2.13 PART M：純機能拡張（超上級）（S151〜S170）
- S151 Mission Control Home
- S152 Explain Composer
- S153 Ops Flow Builder
- S154 Trading Session Dashboard
- S155 Regulatory/Tax Pack Export
- S156 Model Insight Panel
- S157 Stress Scenario Viewer
- S158 Position Decomposition
- S159 Execution Quality Dashboard
- S160 Fee/Cost Attribution
- S161 Data Quality Lab
- S162 Release Impact Sandbox
- S163 Cross-Exchange Normalization Inspector
- S164 Onchain Operations Console
- S165 Arbitrage Radar
- S166 Inventory / Risk Budget Planner
- S167 Event-driven Notebook
- S168 Knowledge Base
- S169 User Macro Recorder
- S170 Unified Alerts Inbox v2 :contentReference[oaicite:16]{index=16}

### 1.2.14 PART N：純機能拡張（究極）（S171〜S190）
- S171 Data Contract Explorer
- S172 Event Correlation Matrix
- S173 Recovery Advisor
- S174 Bot Health Scoring
- S175 Hedge / Neutrality Monitor
- S176 Capital Flow Map
- S177 Exchange Status Aggregator
- S178 Order Lifecycle Inspector
- S179 PnL Attribution
- S180 Liquidity Shock Simulator
- S181 Portfolio Rebalancing Planner
- S182 Strategy Regime Fit
- S183 Abnormal Spread Watch
- S184 Reorg Timeline
- S185 Bridge / Settlement Tracker
- S186 Compliance Tagging
- S187 Operator Play Metrics
- S188 UI Copilot Panel（提案のみ）
- S189 Signal Library
- S190 One-Click Investigation Pack :contentReference[oaicite:17]{index=17}

### 1.2.15 PART O：純機能拡張（最終オプション）（S191〜S205）
- S191 Unified Object Inspector（全対象共通インスペクタ）
- S192 Time Sync Assistant（時刻同期ガイド）
- S193 Network Condition Simulator（回線条件シミュレータ）
- S194 Widget Composition Builder（合成Widget）
- S195 Alert Template Studio（通知テンプレ工房）
- S196 Ops Scenario Generator（事故シナリオ自動生成）
- S197 Explain Quality Scorer（Explain品質改善）
- S198 Multi-Window Popout（複数ウィンドウ運用）
- S199 Operator Handoff Pack（引き継ぎパック）
- S200 Privacy View Profiles（状況別マスキング）
- S201 Behavioral Guardrails（行動ガードレール）
- S202 Domain Lens（観点別レンズ切替）
- S203 Smart Saved Views（保存ビューの自動劣化対策）
- S204 Meta Dashboard of Dashboards（台帳/棚卸し）
- S205 Knowledge-to-Action Links（ナレッジ→行動リンク）:contentReference[oaicite:18]{index=18}

---

## 1.3 IDギャップ（原文に存在しない番号）
- TODO: S59〜S63 が本文中に登場しない（このSSOT断片に未収録か、別文書に存在する可能性）。:contentReference[oaicite:19]{index=19}

---

## 1.4 正本運用ルール（SSOT Governance）
- 本文書（v2.0 FINAL）を **Sの正本（SSOT）** として固定する。
- 今後の変更は原則禁止。やむを得ず改訂する場合は **v2.1** として差分管理し、**S番号は維持** する。:contentReference[oaicite:20]{index=20}

---

## 1.5 Capability Index（ID保持・一括）
> 目的：全ID（Sxx）を一覧化し、参照の起点を単一化する。

### 1.5.1 Index（S1〜S205）
- PART A：S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12
- PART B：S13, S14, S15, S16, S17
- PART C：S18, S19, S20, S21, S22, S23
- PART D：S24, S25, S26, S27, S28
- PART E：S29, S30, S31, S32, S33
- PART F：S34, S35, S36, S37, S38
- PART G：S39, S40, S41, S42, S43
- PART H：S44, S45, S46, S47, S48, S49, S50, S51, S52, S53
- PART I：S54, S55, S56, S57, S58
- TODO（欠番/未記載）：S59, S60, S61, S62, S63
- PART J：S64〜S115
- PART K：S116〜S130
- PART L：S131〜S150
- PART M：S151〜S170
- PART N：S171〜S190
- PART O：S191〜S205 :contentReference[oaicite:21]{index=21}
