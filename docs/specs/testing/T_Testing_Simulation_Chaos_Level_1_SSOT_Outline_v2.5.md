# T. Testing / Simulation / Chaos — Level 1 SSOT Outline（v2.5）
Source: 「T. Testing / Simulation / Chaos — SSOT統合インデックス v2.5（T-01〜T-250 完全網羅・最終版）」 :contentReference[oaicite:0]{index=0}

## 0. SSOTメタ
- ドメイン文字: **T**
- 対象範囲: **T-01 〜 T-250**
- 目的: テスト/シミュレーション/カオス/品質統治に関する要求・能力（Capabilities）をID付きでSSOT化する
- TODO: 上位契約（Non-negotiable / Canonical Model/Contract / Behavior/Tests）の参照元（別ドキュメント）がある場合はリンクを追記

---

## 1. スコープ
本ドキュメントは、以下をSSOTとして整理する。
- テスト種別/ハーネス/決定論/エミュレータ/障害注入
- CIゲート、データ品質、再現性、観測、統治
- 長時間/互換/移行/DR/監査、外部ドリフト対応
- 研究寄りの高度化、および「品質統治OS化」までの拡張

---

## 2. 用語（最小）
- SSOT: Single Source of Truth
- Harness: テスト実行・再現・成果物管理を統一する基盤
- Golden: 必須ケースの期待結果（または期待レンジ）を固定したE2E/差分比較基準
- Drift: 外部仕様/環境/依存の変化によるズレ

TODO: プロジェクト全体の共通用語集がある場合は参照リンクを追加

---

## 3. Capabilities（1.x）
> 入力の見出し（T-xx〜T-yyのカテゴリ）を 1.x として保持する。

### 3.1 T-01〜T-10（基盤・疑似世界・入口）
- **T-01** Taxonomy：テスト種別（Unit/Contract/E2E…）と実行区分をSSOT化
- **T-02** Test Harness Core：RunId/Seed/再現コマンド/標準レポート/成果物管理
- **T-03** Deterministic Fixture：SimClock/seed固定/Virtual I/Oで決定論を強制
- **T-04** CEX Emulator：REST/WS疑似取引所＋異常注入（429/欠損/逆順/再接続/部分約定）
- **T-05** Market Simulator：価格・板・約定モデル（薄化/急変/停止/スリッページ）
- **T-06** On-chain/Reorg Emulator：finality/reorg/RPC不一致を再現する疑似チェーン
- **T-07** Fault Injection Framework：network/storage/deps/cpu/mem等への統一障害注入（FaultProfile）
- **T-08** Property-based Testing：不変条件（状態遷移/冪等/収束/整合）で証明
- **T-09** Fuzz Testing：WS/REST/設定/デコーダ等の境界破壊（panicゼロ）
- **T-10** Golden E2E：paper→shadow→live同型のゴールデンE2E（必須ケース固定）

### 3.2 T-11〜T-20（性能・フレーク・データ・ネットワーク）
- **T-11** Performance/Latency Regression：p50/p95/p99/スループットの回帰防止
- **T-12** Flaky Control：flaky測定→隔離→復帰（再現情報なしはDoD未達）
- **T-13** Dataset/Artifact Management：dataset_ref/保持/赤塗り/再現材料の標準管理
- **T-14** CI Integration Gate：PR/main/releaseの実行スイートと落下理由の固定
- **T-15** Replay Fidelity Suite：録画再生で状態一致（checkpoint/倍速/途中再開）
- **T-16** Time Semantics：時刻逆転/飛び/境界集計/取引所時刻ズレ耐性
- **T-17** Data Quality Gate：欠損/重複/遅延/順序乱れ/板整合の予算ゲート
- **T-18** Secret Leak Tests：ログ/トレース/成果物の秘密漏洩ゼロを強制
- **T-19** Schema/Contract Diff：スキーマ/契約の互換性差分検知（破壊的変更停止）
- **T-20** Network/TLS/Proxy/Tunnel E2E：mTLS/Proxy/cloudflared等“本番形”通信のE2E

### 3.3 T-21〜T-30（復旧・並行性・差分・分散前提）
- **T-21** Runbook-as-Test：復旧手順（再同期/再接続/再起動）を自動検証
- **T-22** Poisoned Data：NaN/∞/巨大/不正UTF-8等の毒入力耐性
- **T-23** Concurrency & Linearizability：注文/取消/約定競合、デッドロック等の検証
- **T-24** Strategy Validation Frame：backtest/forward/shadow整合と再現性固定（任意）
- **T-25** Safety Invariants：SAFE_MODE/CLOSE_ONLY/kill-switch等が破れない証明
- **T-26** Observability Contract：span/metric/alert/trace_idの存在と意味の契約
- **T-27** Forensics Bundle：秘密除去済みの診断束を1コマンド生成
- **T-28** Delta Debugging/Shrinker：失敗入力を最小化（原因特定を高速化）
- **T-29** Reference Oracle：参照実装（遅くてOK）との突合で正しさ固定
- **T-30** Exactly/At-least-once Proof：重複/再試行でも台帳が壊れない証明

### 3.4 T-31〜T-40（再同期・依存・長時間・監査・誘導）
- **T-31** Snapshot/State Invariants：snapshot/diff再同期の不変条件と収束/安全停止
- **T-32** Metamorphic Testing：期待値が作れない領域を同値変換不変で検証
- **T-33** Spec Coverage Matrix：spec↔tests紐付け、未カバーをCIで停止
- **T-34** Dependency Contract Regression：DB/Kafka/Cache等依存の互換回帰検知
- **T-35** Resource Exhaustion：Disk full/OOM/FD枯渇→安全停止→復旧→整合回復
- **T-36** Blue/Green（二重起動）テスト：A/B並走→切替→追いつき→旧停止の証明
- **T-37** Soak/Longevity：長時間劣化（リーク/遅延悪化）検知
- **T-38** Cross-version Compatibility：ローリングアップデート成立/不成立の明示
- **T-39** Audit Correctness：台帳追跡性（根拠イベントへ辿れる）保証
- **T-40** Guided Recovery UX：失敗タイプ別に次アクション/成功判定を自動提示

### 3.5 T-41〜T-50（本番同型・供給網・設定安全・カナリア・現実性）
- **T-41** Prod Parity Harness：本番同型DI/設定/起動順でテスト（乖離禁止）
- **T-42** Build/Artifact Integrity：SBOM/ハッシュ/生成物指紋で“同じ実行物”を証明
- **T-43** Supply-chain Regression：依存危険化/ロック改変/禁止依存の検知
- **T-44** AuthN/AuthZ Invariants：権限境界（private到達不可/失効/最小権限）保証
- **T-45** Retention & Privacy：保持/削除/匿名化が設計通り動くことを検証
- **T-46** Graceful Shutdown & Drain：停止→flush→checkpoint→再開で二重計上しない
- **T-47** Config Safety Suite：危険設定の起動拒否/矛盾検出/差分レポート
- **T-48** Canary Validation：段階リリース合否→拡大/ロールバックを機械化
- **T-49** Policy/Model Regression：戦略/モデル更新の安全ゲート（逸脱検知）
- **T-50** Fault Realism Calibration：実障害統計でfault分布を校正

### 3.6 T-51〜T-60（乖離禁止・監視品質・取引所網羅・予算・既知悪・SLO）
- **T-51** No-Test-Only-Behavior：テスト専用分岐の禁止（許可リスト制）
- **T-52** Alert Quality Suite：鳴るべき時に鳴り、平常時は鳴らない（誤検知抑制）
- **T-53** Venue Parity Matrix：取引所ごとの必須op/異常系/検証セットを固定
- **T-54** Cost & Capacity Budget：保存量/I/O/CPU/メモリ等のコスト予算ゲート
- **T-55** Release Quality Dossier：リリース毎の安全証明レポート生成
- **T-56** Operator Error Injection：誤操作シナリオ注入→安全停止/被害限定
- **T-57** Corruption/Bit-rot耐性：データ部分破損検知/隔離/再取得
- **T-58** Numeric Correctness Suite：丸め/tick/lot/手数料の数値正確性保証
- **T-59** Known-Bad Corpus：過去バグ最小再現ケースの固定・継続実行
- **T-60** SLO Verification：SLI算出→SLOゲート（欠損率/復旧時間等）

### 3.7 T-61〜T-70（移行・OS差・時刻ずれ・モード・再現・統計・探索）
- **T-61** Data Migration Compatibility：DB移行のforward/backwardとロールバック検証
- **T-62** OS Compatibility：Windows/Mac/CI差（パス/ロック/改行等）検証
- **T-63** NTP Drift/Time Skew：時刻同期喪失の検知→安全側移行
- **T-64** Mode Transition Correctness：paper/shadow/live等モード遷移の状態機械保証
- **T-65** One-command Repro：repro.ps1/sh自動生成（誰でも同じ失敗を再現）
- **T-66** Reliability Analytics：成功率/失敗モード/MTTRなど信頼性統計
- **T-67** Stochastic Chaos Campaign：夜間seed探索→縮小→既知悪化の自動循環
- **T-68** Adversarial Network：敵対的通信条件（断片化/改変/TLS異常等）
- **T-69** Malicious Venue Simulator：矛盾/不正応答の取引所を疑似化して検証
- **T-70** Model Checking：状態機械のモデル検査（禁止遷移到達不可の証明）

### 3.8 T-71〜T-80（静的保証・ドリフト・フラグ・DR・整合・監査強化・自動審査）
- **T-71** Static Assurance Gate：panic禁止/unsafe制限等の静的ゲート
- **T-72** Behavior Drift Detection：出力分布の静かな劣化検知（統計比較）
- **T-73** Flag Safety Suite：フラグ組合せの安全検証（危険組合せ禁止）
- **T-74** Disaster Recovery Drill：バックアップ起動/再同期/RPO-RTO証明
- **T-75** Cross-store Consistency：湖→DB→分析の多段整合を突合
- **T-76** Time-travel Debugging：任意時点へ戻して局所再現（チェックポイント＋ログ）
- **T-77** Least-Privilege Proof：最小権限でE2E成立、権限増加は承認制
- **T-78** Tamper-evident Audit：監査証跡の改ざん検知（ハッシュ連鎖等）
- **T-79** Unified Safety Budgets：性能/コスト/DQ/SLO/統治を統合予算で判定
- **T-80** Automated Release Review：予算＋DossierでPASS/WARN/FAIL自動判決

### 3.9 T-81〜T-90（外部世界の変化を閉じる）
- **T-81** Venue Spec Drift Watch：取引所仕様ドリフト検知→issue化
- **T-82** Shadow Replay in Production Shape：本番形シャドーで資金ゼロ検証
- **T-83** Rare Failure Capture：レア条件発生時に高解像度で再現材料を自動保存
- **T-84** Real-world Calibration：実測（遅延/429/切断/partial）でエミュ校正
- **T-85** Contract Snapshot Archive：外部仕様のスナップショット保管と差分履歴
- **T-86** Safety Case生成：ハザード→対策→検証リンクで“安全の論証”を固定
- **T-87** Approval Consistency：例外承認テンプレで人間判断のブレ防止
- **T-88** Provable Guardrails：ガードレール自己診断＋定期訓練で“効く”を証明
- **T-89** Causal Triage：障害の因果切り分け決定木＋類似検索
- **T-90** Change Resilience Gate：外部ドリフト情報を統合して安全側へ倒すゲート

### 3.10 T-91〜T-100（統治・監査・学習の自動化）
- **T-91** Exception Debt Control：例外を期限付き負債として管理（腐敗防止）
- **T-92** Review Quality Gate：重要変更のレビュー要件テンプレを強制
- **T-93** GameDay Automation：定期演習（障害注入→復旧）を自動化
- **T-94** Postmortem-as-Code：事故→再現→テスト化をテンプレ/自動提案
- **T-95** Design Intent Preservation：ADRとテストを紐付け“意図”を保存
- **T-96** Audit-ready Package：監査提出束（Safety Case/Dossier/証跡/承認）生成
- **T-97** Risk-to-Test Mapping：リスク分類→必須テスト自動付与
- **T-98** Compliance-oriented Checks：会計/規制観点の整合性チェック（必要範囲）
- **T-99** Decision UX：人間が見る量を最小化（理由トップN提示）
- **T-100** Quality Constitution for T：T自身の改変ルール（憲法）固定

### 3.11 T-101〜T-110（他ドメインが守る“品質契約”）
- **T-101** Testability Contract：DI/注入点/実I/O直結禁止の契約
- **T-102** Failure Taxonomy Contract：失敗分類・リトライ性・相関IDの契約
- **T-103** Idempotency Contract：冪等キー必須・収束性の契約
- **T-104** State Machine Contract：状態/遷移SSOT化・禁止遷移拒否の契約
- **T-105** Reproducibility Contract：seed/dataset/build指紋・再現材料記録の契約
- **T-106** Observability Contract++：観測（span/metric/alert）契約の強化版
- **T-107** Data Contract：データ進化（SemVer）・再計算可能性・整合検証義務
- **T-108** External Drift Response Contract：外部変更検知→安全側→更新→解除の一本道
- **T-109** Change Impact Contract：変更影響→必須テスト自動算出の契約
- **T-110** Quality Stop Authority：MUST違反は例外なく止める（緊急でもSafety免除なし）

### 3.12 T-111〜T-120（最後の抜け道封鎖）
- **T-111** Backward Incompatibility Sentinel：互換破壊の自動分類と停止
- **T-112** Fail-Closed Enforcement：危険条件で必ずSAFE/停止へ（fail-open禁止）
- **T-113** Dual Telemetry：重要指標の二重観測で“観測の故障”を検知
- **T-114** Meta-Test（Harness Verification）：テスト基盤自体の自己検証
- **T-115** Governance Drift Detector：統治劣化（訓練未実施/例外増殖等）検知
- **T-116** Anti-Bypass Controls：ゲート突破を技術的に不可能化
- **T-117** Minimal Proof Set：MUST最小集合を固定（肥大化防止）
- **T-118** Dependency Circuit Breaker Proof：依存障害時の隔離/遮断/回復を証明
- **T-119** Environment Drift Lockdown：環境指紋の収集とドリフト検知
- **T-120** Final Quality Verdict：Dossier+Budgets+Drift+Governanceで最終判決

### 3.13 T-121〜T-130（現実上限：侵害/停電/全面停止/前提崩壊）
- **T-121** Security IR Drill：侵害想定の初動（ローテ/無効化/SAFE/証跡）演習
- **T-122** Zero-Trust Boundary Tests：境界防御（期限切れ/奪取/端末紛失想定）検証
- **T-123** Venue Outage Total：取引所全面停止（5xx/timeout/照会不能）耐性
- **T-124** Hard Outage：停電/突然死→再起動→復元→整合回復
- **T-125** Compliance Drift Response：規制/税制等の変更時に安全側へ倒す手順固定
- **T-126** Loss Event Playbook Test：損失閾値超過時の停止/縮小/証跡/分析導線
- **T-127** Extreme Market Event Simulator：フラッシュクラッシュ/板消失/飛びの耐性
- **T-128** Price Source Failure：価格ソース破綻の隔離/多数決/停止
- **T-129** Assumption Break Monitor：前提崩壊検知→fail-closed→証跡
- **T-130** Safe Manual Intervention：手動介入の二重確認・監査・復帰整合強制

### 3.14 T-131〜T-150（テスト実務強化）
- **T-131** Spec-to-Test Scaffold：spec→テスト雛形/fixture自動生成
- **T-132** Record-and-Replay Tap：標準録画→リプレイ直結（軽量/フル）
- **T-133** Snapshot Testing：スキーマ/設定/出力のスナップショット差分管理
- **T-134** Artifact Diff Viewer：差分の“最初の分岐点”やトップ差分を可視化
- **T-135** Change-aware Test Selection：変更影響でテスト選別（PR高速化）
- **T-136** Distributed Test Execution：分散実行＋キャッシュ＋統合成果物
- **T-137** Failure Auto-Triage：失敗分類＋次アクション提示＋既知悪照合
- **T-138** Combinatorial Testing：pairwise等で組合せ網羅と時間抑制
- **T-139** State-space Explorer：状態空間探索で穴を炙り出す
- **T-140** Dataset Sanitizer：匿名化/縮約/規約チェックで安全にテスト化
- **T-141** Fee/Slippage Model Library：手数料/スリッページ/刻み等のSSOT実装
- **T-142** RNG Policy & Seed Space：seed探索戦略と再現seed運用
- **T-143** Uncertainty/Confidence Testing：揺らぎに対する許容範囲検証
- **T-144** Ephemeral Test Environments：PR毎の使い捨てE2E環境
- **T-145** Protocol Adapter Tests：WS/HTTP互換（断片化/圧縮/ping）検証
- **T-146** Time-series Consistency Suite：OHLCV/集計/補完の正しさ検証
- **T-147** Scenario Library：典型障害/運用シナリオの部品化
- **T-148** Artifact Lifecycle Manager：証跡のhot/warm/cold運用と圧縮
- **T-149** Pre-release Load Replay：疑似本番量の負荷リプレイで尖り検知
- **T-150** Quality Navigator：spec→test→runbook→監査→Dossier導線の一括ナビ

### 3.15 T-151〜T-170（自動生成・決定論・差分検出・UX強化）
- **T-151** Scenario Fuzzer：シナリオ自動生成→縮小→既知悪化
- **T-152** World Snapshot & Restore：世界全体のスナップショット/復元
- **T-153** Deterministic Scheduler：並行実行順を決定論制御し競合再現を強化
- **T-154** Event Trace Query：イベント列をSQL的に検索・抽出
- **T-155** Contract Test Auto-Refresh：取引所ドリフト→契約テスト更新案の自動生成
- **T-156** Venue Emulator Generator：OpenAPI/AsyncAPI等からエミュ雛形自動生成
- **T-157** Golden Synthesizer：条件指定でgolden合成＋Expected半自動生成
- **T-158** Differential Testing Harness：旧/新・A/B差分試験で静かな回帰を止める
- **T-159** Performance Model & Budget Simulator：全体遅延/詰まりのモデル化と予算予測
- **T-160** Leak Canary：漏洩カナリアで漏洩検出の自己保証
- **T-161** Safe Mutation Testing：テストの強さを変異注入で定量評価
- **T-162** Chaos Experiment Planner：注入計画の自動提案（価値最大化）
- **T-163** Time-skew Lab：時刻ずれを体系的に注入し影響を一括検証
- **T-164** Data Repair Tester：欠損/破損後の修復が正しいことを証明
- **T-165** Venue Coverage Linter：取引所カバレッジの静的リンター（漏れをPRで停止）
- **T-166** Safety Budget Dashboard：安全予算のPASS/WARN/FAILダッシュボード
- **T-167** Replay Speed Control：倍速/ステップ/条件ブレークでデバッグ加速
- **T-168** Test Debt Tracker：未検証/flake/例外/未カバーを負債として定量化
- **T-169** Shadow Safety Gate：shadow差分が大きい場合に安全側へ倒すゲート
- **T-170** Failure Marketplace：失敗の型カタログ化・検索・復旧導線

### 3.16 T-171〜T-190（Tを“品質プロダクト”にする機能）
- **T-171** Test IDE/CLI：対話型CLI（run/replay/diff等）で操作を統一
- **T-172** Artifact Search Index：成果物（ログ/イベント/メトリクス）の全文検索
- **T-173** Regression Bisector：回帰コミット/PRを自動バイセクト特定
- **T-174** Test Result Knowledge Base：失敗→原因→修正→防止テストをKB化
- **T-175** Coverage Heatmap：spec×重要度×カバレッジのヒートマップ
- **T-176** Risk-based Scheduling：重要度に応じた実行頻度/優先度の最適化
- **T-177** Flake Root-cause Miner：flakeの原因候補を解析し改善を支援
- **T-178** Resource Profiler：CPU/メモリ/FD/I/Oの自動収集と比較
- **T-179** Dependency Chaos Sandbox：依存系カオスの専用テンプレ環境
- **T-180** Multi-venue Replay Orchestrator：複数取引所同期リプレイのオーケストレーション
- **T-181** Checkpoint Format Standard：チェックポイント形式の標準化（圧縮/署名/互換）
- **T-182** Deterministic Network Stack：通信順序/遅延/再送の決定論ネットスタック
- **T-183** Golden Budget Auto-tuning：golden許容幅の提案（安全側・承認制）
- **T-184** Spec Change Wizard：仕様変更に必要な更新（tests/golden/contracts/runbook）をガイド
- **T-185** Safety-case Compiler：実行結果からSafety Caseを自動コンパイル/未証明を赤表示
- **T-186** Live Dry-run Mode：本番接続だが絶対発注しない“ハード保証”ドライラン
- **T-187** Schema Evolution Simulator：vN/vN+1混在や進化影響を自動シミュ
- **T-188** Automated Benchmark Governance：ベンチ測定法/入力/環境の規約と自動審査
- **T-189** Incident-to-Test Auto PR：事故材料から再現テストのPR雛形を自動生成
- **T-190** Quality SLO Dashboard：SLO/Budgets/例外/flake等を統合した品質ダッシュボード

### 3.17 T-191〜T-210（維持コストを下げる“運用機能”）
- **T-191** Test Budget Optimizer：制約時間内でテスト集合を最適化（重要優先・失敗率/変更影響反映）
- **T-192** Adaptive Quarantine：flaky/外部ドリフト/環境不安定を自動隔離しつつ最低限Safetyは実行
- **T-193** Test Flows as DAG：テスト実行をDAG化し最短経路・スキップ/短縮を自動化
- **T-194** Proof Artifact Signing：Dossier/Forensics/Golden結果を署名し改ざん検知（監査強化）
- **T-195** Replay Determinism Verifier：同一dataset/seedで完全一致するか検証し非決定要因を特定
- **T-196** Synthetic Traffic Generator：実運用に近いイベント分布/量の合成入力生成（T-149強化）
- **T-197** Multi-objective Budget Tuner：性能/コスト/DQ/安全/統治予算を多目的最適化（Pareto）
- **T-198** Investigation Notebook：失敗時に仮説→観測→確認→結論の調査ノートを自動生成
- **T-199** Cross-run Similarity Search：過去Runとの類似検索（ログ/イベント/メトリクス）で再発判定
- **T-200** Live Contract Probe：夜間に実サービス契約を軽く観測し差分化（T-81自動化）
- **T-201** Emulator Fidelity Score：実測とエミュの差をスコア化し校正を促す（T-84/T-50補強）
- **T-202** Test Data Lineage：datasetの録画/合成/縮小の系譜（lineage）を記録し説明責任を強化
- **T-203** Dynamic Safety Envelope：状況（ボラ/遅延/欠損）に応じて安全予算を動的に保守化
- **T-204** Operator Training Portal：GameDayシナリオ/結果/改善を一元化する訓練ポータル
- **T-205** Change-set Explainer：PR差分から影響・必要テスト・リスクを自動生成（レビュー支援）
- **T-206** Test Coverage Enforcement for Docs：仕様/Runbook/ADRに対応するテスト存在を強制
- **T-207** Privacy-preserving Replay：匿名化しつつ統計特性を保ったリプレイ（T-140高度化）
- **T-208** Energy/Power Budget Tests：電力/発熱/サーマル影響の予算テスト（家庭PC運用向け）
- **T-209** Multi-node Time Sync Lab：二重起動/複数ノードの時刻ズレ再現と整合検証（T-63拡張）
- **T-210** One-page Proof Generator：安全要点・予算・既知リスクを1枚に自動要約（判断最小化）

### 3.18 T-211〜T-230（研究寄り・高効果オプション）
- **T-211** Probabilistic Proof Runs：複数seed×条件で統計的に合格判定し“たまたま緑”を排除
- **T-212** Counterfactual Replay：同一イベント列で反実仮想分岐（遅延+X等）を生成し比較
- **T-213** Automated Invariant Discovery：ログ/状態から不変条件候補を自動抽出しPBTを強化
- **T-214** Learned Triage Model：過去失敗から学習した原因分類で初動トリアージ精度を向上
- **T-215** Semantic Diff for Events：意味論（互換/危険/必須性）で差分重大度を判定
- **T-216** Constraint-based Scenario Generator：制約を満たす最小シナリオ（429≥3等）を生成
- **T-217** Distributed Trace Replay：分散トレースを相関維持で再生し本番事故再現力を向上
- **T-218** Chaos Safety Sandbox：資産/鍵/権限を完全隔離した安全カオス環境
- **T-219** Multi-agent Review Assistant：影響/必要テスト/危険差分を自動指摘するレビュー補助
- **T-220** Auto-ADR Generator：重要変更からADRテンプレ自動生成し証明へリンク
- **T-221** Spec as Executable Runner：仕様（制約/状態機械）を実行し実装と突合
- **T-222** Market Microstructure Emulator：注文フロー/板更新/キャンセル密度等のミクロ構造再現
- **T-223** Long-horizon Simulation：週〜月相当の圧縮長期シミュで複合劣化/事象を評価
- **T-224** Spec Linter & Refactorer：spec曖昧/参照切れ/ID重複を検出し修正案提示
- **T-225** Code-to-Spec Traceability：コード→仕様の逆引きトレースで影響評価を強化
- **T-226** Proof of Safety Certificate：署名付き安全証明書（要点/予算/既知リスク）生成
- **T-227** External News/Reg Watch：取引所障害/規制改定ニュース監視をリスク判定に反映
- **T-228** Live Safety Shadowing：本番判断を影で再計算し差分警告（発注なし）
- **T-229** Test Cost Accounting：テスト実行コスト（時間/電力/保存/API）を計測・最適化
- **T-230** Quality DSL：予算/ゲート/例外/シナリオをDSL宣言し一括生成・検証

### 3.19 T-231〜T-250（品質統治のOS化：最終拡張）
- **T-231** Policy-as-Code Engine：品質ポリシー（予算/ゲート/例外/リスク）を統一エンジンで実行
- **T-232** Proof Graph：spec→ハザード→対策→テスト→結果→証拠の証明グラフ化
- **T-233** Quality Diff of Diffs：PR差分の“差分パターン”分析で危険変更を早期検知
- **T-234** Evidence Minimizer：監査/再現に必要な最小証跡セットを自動抽出し保管最適化
- **T-235** Contract Canary Pack：最重要契約だけを超軽量で常時監視（高速ドリフト検知）
- **T-236** Failure Injection DSL：障害注入をDSL化し横断・統一（network/deps/time/resource）
- **T-237** Replay-to-Unit Extractor：E2E失敗から局所ユニットテストを自動生成
- **T-238** Noisy Neighbor Simulator：CPU/IO/メモリ干渉（同居プロセス）を再現し家庭PC運用に寄せる
- **T-239** Clock/Randomness Leak Detector：実時刻/暗黙乱数の混入を静的/動的に検知して停止
- **T-240** Reconcile Correctness Kit：reconcile仕様化・差分検証・安全停止条件・収束性を専用強化
- **T-241** Ledger Consistency Prover：台帳保存則（非負/二重計上禁止/因果追跡）を統合証明
- **T-242** Async Backpressure Lab：キュー詰まり/遅延伝播/バッチ境界崩れを体系注入し耐性証明
- **T-243** SLA for Quality Tooling：T基盤（CI/キャッシュ/UI）のSLO化でT自身の信頼性を担保
- **T-244** Spec Migration Tooling：spec構造変更の自動変換で参照切れ/ID衝突を防止
- **T-245** Emergency Patch Validator：緊急パッチ用の最小検証セットを自動構成し品質を落とさない
- **T-246** Reality Gap Score：実運用 vs エミュ/シャドーの乖離を数値化し品質判断に統合
- **T-247** Confidential Computing Mode：秘密を外に出さない機密テスト運用（成果物完全マスク）
- **T-248** Multi-tenant Safety：複数Bot/戦略同時稼働の干渉（リスク枠/帯域/注文レート）検証
- **T-249** Quality Rollback：品質劣化検知→証拠付き自動ロールバック＋原因PR推定
- **T-250** Global Quality Timeline：変更/外部ドリフト/例外/訓練/障害を1本の品質タイムラインに統合

---

## 4. Capability Index（ID保持・全件）
> 要求どおり、ID（T-xx）を必ず保持し、索引として再掲する。

- T-01 Taxonomy
- T-02 Test Harness Core
- T-03 Deterministic Fixture
- T-04 CEX Emulator
- T-05 Market Simulator
- T-06 On-chain/Reorg Emulator
- T-07 Fault Injection Framework
- T-08 Property-based Testing
- T-09 Fuzz Testing
- T-10 Golden E2E
- T-11 Performance/Latency Regression
- T-12 Flaky Control
- T-13 Dataset/Artifact Management
- T-14 CI Integration Gate
- T-15 Replay Fidelity Suite
- T-16 Time Semantics
- T-17 Data Quality Gate
- T-18 Secret Leak Tests
- T-19 Schema/Contract Diff
- T-20 Network/TLS/Proxy/Tunnel E2E
- T-21 Runbook-as-Test
- T-22 Poisoned Data
- T-23 Concurrency & Linearizability
- T-24 Strategy Validation Frame
- T-25 Safety Invariants
- T-26 Observability Contract
- T-27 Forensics Bundle
- T-28 Delta Debugging/Shrinker
- T-29 Reference Oracle
- T-30 Exactly/At-least-once Proof
- T-31 Snapshot/State Invariants
- T-32 Metamorphic Testing
- T-33 Spec Coverage Matrix
- T-34 Dependency Contract Regression
- T-35 Resource Exhaustion
- T-36 Blue/Green（二重起動）テスト
- T-37 Soak/Longevity
- T-38 Cross-version Compatibility
- T-39 Audit Correctness
- T-40 Guided Recovery UX
- T-41 Prod Parity Harness
- T-42 Build/Artifact Integrity
- T-43 Supply-chain Regression
- T-44 AuthN/AuthZ Invariants
- T-45 Retention & Privacy
- T-46 Graceful Shutdown & Drain
- T-47 Config Safety Suite
- T-48 Canary Validation
- T-49 Policy/Model Regression
- T-50 Fault Realism Calibration
- T-51 No-Test-Only-Behavior
- T-52 Alert Quality Suite
- T-53 Venue Parity Matrix
- T-54 Cost & Capacity Budget
- T-55 Release Quality Dossier
- T-56 Operator Error Injection
- T-57 Corruption/Bit-rot耐性
- T-58 Numeric Correctness Suite
- T-59 Known-Bad Corpus
- T-60 SLO Verification
- T-61 Data Migration Compatibility
- T-62 OS Compatibility
- T-63 NTP Drift/Time Skew
- T-64 Mode Transition Correctness
- T-65 One-command Repro
- T-66 Reliability Analytics
- T-67 Stochastic Chaos Campaign
- T-68 Adversarial Network
- T-69 Malicious Venue Simulator
- T-70 Model Checking
- T-71 Static Assurance Gate
- T-72 Behavior Drift Detection
- T-73 Flag Safety Suite
- T-74 Disaster Recovery Drill
- T-75 Cross-store Consistency
- T-76 Time-travel Debugging
- T-77 Least-Privilege Proof
- T-78 Tamper-evident Audit
- T-79 Unified Safety Budgets
- T-80 Automated Release Review
- T-81 Venue Spec Drift Watch
- T-82 Shadow Replay in Production Shape
- T-83 Rare Failure Capture
- T-84 Real-world Calibration
- T-85 Contract Snapshot Archive
- T-86 Safety Case生成
- T-87 Approval Consistency
- T-88 Provable Guardrails
- T-89 Causal Triage
- T-90 Change Resilience Gate
- T-91 Exception Debt Control
- T-92 Review Quality Gate
- T-93 GameDay Automation
- T-94 Postmortem-as-Code
- T-95 Design Intent Preservation
- T-96 Audit-ready Package
- T-97 Risk-to-Test Mapping
- T-98 Compliance-oriented Checks
- T-99 Decision UX
- T-100 Quality Constitution for T
- T-101 Testability Contract
- T-102 Failure Taxonomy Contract
- T-103 Idempotency Contract
- T-104 State Machine Contract
- T-105 Reproducibility Contract
- T-106 Observability Contract++
- T-107 Data Contract
- T-108 External Drift Response Contract
- T-109 Change Impact Contract
- T-110 Quality Stop Authority
- T-111 Backward Incompatibility Sentinel
- T-112 Fail-Closed Enforcement
- T-113 Dual Telemetry
- T-114 Meta-Test（Harness Verification）
- T-115 Governance Drift Detector
- T-116 Anti-Bypass Controls
- T-117 Minimal Proof Set
- T-118 Dependency Circuit Breaker Proof
- T-119 Environment Drift Lockdown
- T-120 Final Quality Verdict
- T-121 Security IR Drill
- T-122 Zero-Trust Boundary Tests
- T-123 Venue Outage Total
- T-124 Hard Outage
- T-125 Compliance Drift Response
- T-126 Loss Event Playbook Test
- T-127 Extreme Market Event Simulator
- T-128 Price Source Failure
- T-129 Assumption Break Monitor
- T-130 Safe Manual Intervention
- T-131 Spec-to-Test Scaffold
- T-132 Record-and-Replay Tap
- T-133 Snapshot Testing
- T-134 Artifact Diff Viewer
- T-135 Change-aware Test Selection
- T-136 Distributed Test Execution
- T-137 Failure Auto-Triage
- T-138 Combinatorial Testing
- T-139 State-space Explorer
- T-140 Dataset Sanitizer
- T-141 Fee/Slippage Model Library
- T-142 RNG Policy & Seed Space
- T-143 Uncertainty/Confidence Testing
- T-144 Ephemeral Test Environments
- T-145 Protocol Adapter Tests
- T-146 Time-series Consistency Suite
- T-147 Scenario Library
- T-148 Artifact Lifecycle Manager
- T-149 Pre-release Load Replay
- T-150 Quality Navigator
- T-151 Scenario Fuzzer
- T-152 World Snapshot & Restore
- T-153 Deterministic Scheduler
- T-154 Event Trace Query
- T-155 Contract Test Auto-Refresh
- T-156 Venue Emulator Generator
- T-157 Golden Synthesizer
- T-158 Differential Testing Harness
- T-159 Performance Model & Budget Simulator
- T-160 Leak Canary
- T-161 Safe Mutation Testing
- T-162 Chaos Experiment Planner
- T-163 Time-skew Lab
- T-164 Data Repair Tester
- T-165 Venue Coverage Linter
- T-166 Safety Budget Dashboard
- T-167 Replay Speed Control
- T-168 Test Debt Tracker
- T-169 Shadow Safety Gate
- T-170 Failure Marketplace
- T-171 Test IDE/CLI
- T-172 Artifact Search Index
- T-173 Regression Bisector
- T-174 Test Result Knowledge Base
- T-175 Coverage Heatmap
- T-176 Risk-based Scheduling
- T-177 Flake Root-cause Miner
- T-178 Resource Profiler
- T-179 Dependency Chaos Sandbox
- T-180 Multi-venue Replay Orchestrator
- T-181 Checkpoint Format Standard
- T-182 Deterministic Network Stack
- T-183 Golden Budget Auto-tuning
- T-184 Spec Change Wizard
- T-185 Safety-case Compiler
- T-186 Live Dry-run Mode
- T-187 Schema Evolution Simulator
- T-188 Automated Benchmark Governance
- T-189 Incident-to-Test Auto PR
- T-190 Quality SLO Dashboard
- T-191 Test Budget Optimizer
- T-192 Adaptive Quarantine
- T-193 Test Flows as DAG
- T-194 Proof Artifact Signing
- T-195 Replay Determinism Verifier
- T-196 Synthetic Traffic Generator
- T-197 Multi-objective Budget Tuner
- T-198 Investigation Notebook
- T-199 Cross-run Similarity Search
- T-200 Live Contract Probe
- T-201 Emulator Fidelity Score
- T-202 Test Data Lineage
- T-203 Dynamic Safety Envelope
- T-204 Operator Training Portal
- T-205 Change-set Explainer
- T-206 Test Coverage Enforcement for Docs
- T-207 Privacy-preserving Replay
- T-208 Energy/Power Budget Tests
- T-209 Multi-node Time Sync Lab
- T-210 One-page Proof Generator
- T-211 Probabilistic Proof Runs
- T-212 Counterfactual Replay
- T-213 Automated Invariant Discovery
- T-214 Learned Triage Model
- T-215 Semantic Diff for Events
- T-216 Constraint-based Scenario Generator
- T-217 Distributed Trace Replay
- T-218 Chaos Safety Sandbox
- T-219 Multi-agent Review Assistant
- T-220 Auto-ADR Generator
- T-221 Spec as Executable Runner
- T-222 Market Microstructure Emulator
- T-223 Long-horizon Simulation
- T-224 Spec Linter & Refactorer
- T-225 Code-to-Spec Traceability
- T-226 Proof of Safety Certificate
- T-227 External News/Reg Watch
- T-228 Live Safety Shadowing
- T-229 Test Cost Accounting
- T-230 Quality DSL
- T-231 Policy-as-Code Engine
- T-232 Proof Graph
- T-233 Quality Diff of Diffs
- T-234 Evidence Minimizer
- T-235 Contract Canary Pack
- T-236 Failure Injection DSL
- T-237 Replay-to-Unit Extractor
- T-238 Noisy Neighbor Simulator
- T-239 Clock/Randomness Leak Detector
- T-240 Reconcile Correctness Kit
- T-241 Ledger Consistency Prover
- T-242 Async Backpressure Lab
- T-243 SLA for Quality Tooling
- T-244 Spec Migration Tooling
- T-245 Emergency Patch Validator
- T-246 Reality Gap Score
- T-247 Confidential Computing Mode
- T-248 Multi-tenant Safety
- T-249 Quality Rollback
- T-250 Global Quality Timeline

---

## 5. TODO（不足の明示）
- TODO: 各T項目の「MUST/SHOULD/MAY」「受入条件」「測定指標」「参照する契約（Canonical Model/Contract）」が別SSOTに存在する場合、相互リンクを追加
- TODO: TドメインのNon-negotiable（譲れない原則）が別文書にある場合、ここに転記ではなく参照として紐付け
- TODO: TドメインのBehavior/Tests（テスト仕様そのもの）が別文書にある場合、T-xx ↔ spec ↔ tests のトレーサビリティ（T-33等）へ接続する参照を追加
