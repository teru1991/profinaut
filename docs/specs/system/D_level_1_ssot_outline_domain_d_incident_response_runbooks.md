# Level 1 SSOT Outline — Domain D (Incident Response / Runbooks)
出典: D.txt :contentReference[oaicite:0]{index=0}

## 0. Meta
- Domain: **D**
- Title: Incident Response / Runbooks
- Source Version: 「実装目標 機能洗い出し 最終清書版 v1.2（SSOT）」
- Scope Note: 本書は「全ドメイン共通の運用SSOT（Runbook + Incident + Evidence）」を定義する

---

## 1. Non-negotiables
### 1.1 目的と到達点（Non-negotiable）
- Dは、Profinaut/UCELを含む全ドメインに対して、障害時に **安全・正確・再現可能・監査可能** な対応を可能にする「運用SSOT（Runbook + Incident + Evidence）」を提供する :contentReference[oaicite:1]{index=1}

### 1.2 必達要件
- 障害対応を **安全→証跡→観測→復旧→検証→学習** の順に迷わず実行できる :contentReference[oaicite:2]{index=2}
- Runbook/Incident/証跡は **機械的に検証** される（lint/CI Gate/contract） :contentReference[oaicite:3]{index=3}
- degraded運転や切替は勝手に暴れず、発生・解除ともに **必ず証跡** が残る :contentReference[oaicite:4]{index=4}
- 事後検証は形骸化せず、改善が **Runbook/Policy/Test/監視/実装** に必ず反映される :contentReference[oaicite:5]{index=5}
- 将来の実行可能Runbook化に耐えるため、自動化の安全境界（OK/NG）を仕様として固定する :contentReference[oaicite:6]{index=6}
- **Windows本番運用** で確実に実行できる（PowerShell正本） :contentReference[oaicite:7]{index=7}

---

## 2. Scope
### 2.1 Dが担う（必達スコープ）
#### 2.1.1 Runbook体系
- 入口（症状別）/索引/分類/相互参照 :contentReference[oaicite:8]{index=8}
- テンプレ（必須章立て）とメタデータ（owner, maturity, verified/drilled） :contentReference[oaicite:9]{index=9}
- 陳腐化防止（期限切れ検知、廃止/deprecate運用） :contentReference[oaicite:10]{index=10}

#### 2.1.2 Incident運用
- インシデント台帳（ID規約、親子/関連付け） :contentReference[oaicite:11]{index=11}
- タイムライン（作業ログ）＋意思決定ログ（Decision log） :contentReference[oaicite:12]{index=12}
- スコープ宣言（Impact/Scope）と復旧判定（RESOLVED/CLOSED） :contentReference[oaicite:13]{index=13}

#### 2.1.3 Evidence（証跡）統合
- Support bundle（秘匿/保全/監査耐性/チェーン・オブ・カストディ） :contentReference[oaicite:14]{index=14}
- bundle容量・保持・圧縮・秘匿失敗時fail-closed :contentReference[oaicite:15]{index=15}
- bundle保存先障害時の代替保存（ローカル退避→後で移送） :contentReference[oaicite:16]{index=16}

#### 2.1.4 安全運転
- Degraded運転（状態機械） :contentReference[oaicite:17]{index=17}
- Safety Actions Matrix（状態×許可行為の完全表） :contentReference[oaicite:18]{index=18}
- Change Freeze（変更凍結）と例外ルール :contentReference[oaicite:19]{index=19}
- Backout/Rollback（戻しを仕様化） :contentReference[oaicite:20]{index=20}
- 高危険操作の最終確認（1人運用向けチェックリスト） :contentReference[oaicite:21]{index=21}

#### 2.1.5 切替・依存・現実運用
- Failover/Failback（自動/半自動/手動の安全条件） :contentReference[oaicite:22]{index=22}
- Split-brain防止（二重起動・二重実行・二重書込み） :contentReference[oaicite:23]{index=23}
- 依存先障害（取引所/Cloudflare/ISP/DNS）トリアージ :contentReference[oaicite:24]{index=24}
- 外部仕様変更（取引所API変更/廃止）を“障害扱い”する運用 :contentReference[oaicite:25]{index=25}
- Partial Failure（部分故障）標準対応 :contentReference[oaicite:26]{index=26}
- 429（Rate Limit）を独立カテゴリとして扱う :contentReference[oaicite:27]{index=27}

#### 2.1.6 整合・データ完全性
- Reconciliation（注文/約定/残高/ポジ/手数料/入出金）手順と証跡化 :contentReference[oaicite:28]{index=28}
- 取引所側の遅延反映・訂正・取消遅延を前提にした再照合ルール :contentReference[oaicite:29]{index=29}
- 欠損/重複/補修（backfill/rebuild/mark/quarantine） :contentReference[oaicite:30]{index=30}
- 破壊的復旧（replay/reprocess/再投入）の境界と承認 :contentReference[oaicite:31]{index=31}

#### 2.1.7 実行可能性
- Windows本番での運用手順（PowerShell正本） :contentReference[oaicite:32]{index=32}
- Runbook自動化の境界（Read-onlyは可、危険操作は原則不可） :contentReference[oaicite:33]{index=33}
- JP/EN方針（最低限：見出し二言語など） :contentReference[oaicite:34]{index=34}

#### 2.1.8 人間の限界対策
- P0クイック手順（初動10手） :contentReference[oaicite:35]{index=35}
- Do-Not-Do（やってはいけないこと） :contentReference[oaicite:36]{index=36}
- 疲労・睡眠不足時の停止基準（迷ったらHALT→証跡→休む） :contentReference[oaicite:37]{index=37}

### 2.2 Dが参照する（主担当外）
- 観測（Prom/Loki/Grafana等）の設計と実装 :contentReference[oaicite:38]{index=38}
- 安全装置（Kill Switchなど）の実装 :contentReference[oaicite:39]{index=39}
- 監査ログ基盤（Audit Event Log）の実装 :contentReference[oaicite:40]{index=40}
- Secret管理/Vault等の実装 :contentReference[oaicite:41]{index=41}
- Dはそれらの **運用導線** と **証跡要件** を固定する :contentReference[oaicite:42]{index=42}

---

## 3. Canonical Models
### 3.1 Incident（最低限）
- incident_id（規約） :contentReference[oaicite:43]{index=43}
- severity（P0/P1/P2：主観排除のルール） :contentReference[oaicite:44]{index=44}
- status：OPEN→ACKED→MITIGATING→MONITORING→RESOLVED→CLOSED :contentReference[oaicite:45]{index=45}
- impact/scope（どこまで影響） :contentReference[oaicite:46]{index=46}
- current_safety_state（NORMAL/SAFE_MODE/…） :contentReference[oaicite:47]{index=47}
- timeline（作業ログ）＋ decision_log :contentReference[oaicite:48]{index=48}
- evidence_refs（bundle、監査照会、差分、再現データ） :contentReference[oaicite:49]{index=49}
- related_incidents（親子/関連） :contentReference[oaicite:50]{index=50}

### 3.2 Runbook（最低限）
- runbook_id, kind（TRIAGE/MITIGATION/RECOVERY/DRILL/GOVERNANCE） :contentReference[oaicite:51]{index=51}
- applies_to, triggers, required_evidence :contentReference[oaicite:52]{index=52}
- dangerous_ops（T0/T1/T2） :contentReference[oaicite:53]{index=53}
- platform（windows_powershell/bash 等） :contentReference[oaicite:54]{index=54}
- supported_modes（dev/stage/prod、paper/shadow/live） :contentReference[oaicite:55]{index=55}
- owner, maturity, last_verified, last_drilled :contentReference[oaicite:56]{index=56}
- 安全ラベル（Read-only / Reversible / Irreversible） :contentReference[oaicite:57]{index=57}

### 3.3 Postmortem（最低限）
- incident紐付け :contentReference[oaicite:58]{index=58}
- 原因＋「検知/対応/復旧/学習」の欠陥分析 :contentReference[oaicite:59]{index=59}
- action itemsは Runbook/Policy/Test/監視/実装 のどれかに必ず落とす :contentReference[oaicite:60]{index=60}

---

## 4. Architecture
### 4.1 体系設計（Spec / Contracts / Policy / Plans / Runbooks / Gates）
#### 4.1.1 Spec（固定）
- Incident Response Core（定義・状態遷移・記録粒度・Decision log・Change Freeze） :contentReference[oaicite:61]{index=61}
- Degraded運転（状態機械） :contentReference[oaicite:62]{index=62}
- Safety Actions Matrix（状態×許可行為） :contentReference[oaicite:63]{index=63}
- Evidence Support Bundle（秘匿/保全/監査耐性） :contentReference[oaicite:64]{index=64}
- Recovery Acceptance（照合OK含む復旧判定） :contentReference[oaicite:65]{index=65}
- Post-Recovery Watch（復旧後監視強化） :contentReference[oaicite:66]{index=66}
- Backout/Rollback（戻し） :contentReference[oaicite:67]{index=67}
- Time Integrity（時計ズレ） :contentReference[oaicite:68]{index=68}
- Split-brain防止（冗長/二重起動） :contentReference[oaicite:69]{index=69}
- Reconciliation（照合） :contentReference[oaicite:70]{index=70}
- Data Integrity Incident（欠損/重複/補修/隔離、破壊的復旧境界） :contentReference[oaicite:71]{index=71}
- External Dependency Change（外部仕様変更） :contentReference[oaicite:72]{index=72}
- Incident Linking（親子/関連） :contentReference[oaicite:73]{index=73}
- Runbook Automation Safety（自動化境界） :contentReference[oaicite:74]{index=74}
- Runbook Lint Rules（必須章立て、危険操作導線、期限切れ、廃止運用） :contentReference[oaicite:75]{index=75}

#### 4.1.2 Contracts（schema）
- incident/runbook/postmortem/degraded/evidence/coverage/drills のschemaを用意しCIで検証 :contentReference[oaicite:76]{index=76}

#### 4.1.3 Policy（運用値）
- severity mapping（主観排除） :contentReference[oaicite:77]{index=77}
- notification policy（通知疲れ防止） :contentReference[oaicite:78]{index=78}
- change freeze policy（例外条件） :contentReference[oaicite:79]{index=79}
- failover/failback policy（安全条件） :contentReference[oaicite:80]{index=80}
- evidence policy + retention + size limits + fail-closed :contentReference[oaicite:81]{index=81}
- time policy（許容ドリフト） :contentReference[oaicite:82]{index=82}
- alert noise policy（silence期限、誤検知記録、調整DoD） :contentReference[oaicite:83]{index=83}

#### 4.1.4 Plans（計画）
- Runbook coverage plan（P0穴ゼロ） :contentReference[oaicite:84]{index=84}
- verification plan（期限切れWARN→FAIL） :contentReference[oaicite:85]{index=85}
- drill schedule + acceptance（合格条件） :contentReference[oaicite:86]{index=86}
- security rotation plan（鍵ローテ訓練） :contentReference[oaicite:87]{index=87}
- alert tuning plan（誤検知/ノイズ改善の継続） :contentReference[oaicite:88]{index=88}

#### 4.1.5 Runbooks（手順）
- Registry（runbooks.yaml：全登録SSOT） :contentReference[oaicite:89]{index=89}
- Coverage Matrix（症状/アラート ↔ runbook、P0未紐付けFAIL） :contentReference[oaicite:90]{index=90}
- Drill Registry（drills.yaml） :contentReference[oaicite:91]{index=91}
- Templates（runbook/incident/postmortem） :contentReference[oaicite:92]{index=92}
- Runbook必須章立て：Triage / Safety / Evidence / Mitigation / Recovery / Validation / Backout / Post-Recovery Watch / Related / Do-Not-Do / Prerequisites :contentReference[oaicite:93]{index=93}
- Quick：P0初動10手、凍結、HALT+evidence、高危険確認 :contentReference[oaicite:94]{index=94}
- Platformカテゴリ：time drift、split-brain復旧、依存先障害、cloudflared、外部仕様変更、ディスク逼迫、evidence失敗、429、部分故障 :contentReference[oaicite:95]{index=95}
- Executionカテゴリ：照合、注文失敗、取消失敗、遅延、復旧判定 :contentReference[oaicite:96]{index=96}
- Marketdataカテゴリ：欠損補修、重複対策、隔離、再処理境界 :contentReference[oaicite:97]{index=97}
- Securityカテゴリ：鍵漏洩疑い、不正注文疑い（疑い段階の最小行動） :contentReference[oaicite:98]{index=98}
- Windowsカテゴリ：運用コマンド集（PowerShell正本） :contentReference[oaicite:99]{index=99}

#### 4.1.6 Quality Gates（CI）
- docs/** 変更時に必ず実施するゲート:
  - 変更ファイル一覧をサマリ（更新通知） :contentReference[oaicite:100]{index=100}
  - runbook lint（必須章立て、危険操作導線、Backout/Watch/Related、Prerequisites/Do-Not-Do） :contentReference[oaicite:101]{index=101}
  - schema検証（contracts準拠） :contentReference[oaicite:102]{index=102}
  - registry整合（runbooks.yaml登録必須） :contentReference[oaicite:103]{index=103}
  - coverage穴検出（特にP0未紐付けFAIL） :contentReference[oaicite:104]{index=104}
  - last_verified/last_drilled期限切れ（WARN→FAIL） :contentReference[oaicite:105]{index=105}
  - runbook廃止運用（deprecated stub整合）も検証 :contentReference[oaicite:106]{index=106}

---

## 5. Operational Rules
### 5.1 事故防止の最重要規約（強制）
- Backout必須：危険操作は戻し手順と証跡が必須 :contentReference[oaicite:107]{index=107}
- 高危険最終確認：HALT解除、鍵、failback、保存先切替はチェックリスト必須 :contentReference[oaicite:108]{index=108}
- Change Freeze：インシデント中の変更禁止と例外形式 :contentReference[oaicite:109]{index=109}
- Post-Recovery Watch：復旧直後の監視強化と解除条件 :contentReference[oaicite:110]{index=110}
- Decision log：判断理由を短文で必ず残す :contentReference[oaicite:111]{index=111}
- Time integrity：時計ズレ確認の標準化 :contentReference[oaicite:112]{index=112}
- Split-brain防止：二重実行・二重書込みを絶対に起こさない :contentReference[oaicite:113]{index=113}
- Quarantine：疑わしいものを隔離して延焼を止める :contentReference[oaicite:114]{index=114}
- 429/Partial Failure：独立カテゴリとして扱う :contentReference[oaicite:115]{index=115}
- 破壊的再処理境界：replay/reprocessは承認と証跡が必須 :contentReference[oaicite:116]{index=116}
- bundle保存不能時の代替：証跡ゼロを防ぐ :contentReference[oaicite:117]{index=117}
- 疲労前提：迷ったらHALT→証跡→休む、の基準を明記 :contentReference[oaicite:118]{index=118}

---

## 6. Definition of Done
### 6.1 Dの洗い出し完了（最終DoD）
- 必要spec/contract/policy/plan/runbook/gateの置き場所が全て確定し穴がない :contentReference[oaicite:119]{index=119}
- P0相当カテゴリが coverage_matrix で必ずRunbookに辿れる :contentReference[oaicite:120]{index=120}
- 危険操作・戻し・解除条件・証跡・判断理由が仕様として強制できる :contentReference[oaicite:121]{index=121}
- Windows本番運用で実行可能 :contentReference[oaicite:122]{index=122}
- docs更新はCIサマリで必ず可視化され、品質保証が働く :contentReference[oaicite:123]{index=123}
- 事後検証が改善に必ずつながる（「気をつける」で終わらない） :contentReference[oaicite:124]{index=124}

---

## 7. Capability Index（ID保持）
- Domain: **D**
- IDs（A-xxx / Sxx / T-xx / F-xx / Y-xx 等）: **該当IDの明示なし（原文に存在しないため追加しない）** :contentReference[oaicite:125]{index=125}

---

## 8. TODO（不足・未確定のまま残す）
- TODO: Runbook/Incident/Postmortem の **具体的テンプレ本文**（章ごとの必須フィールド、記入例）
- TODO: 「dangerous_ops（T0/T1/T2）」の **判定基準**（操作分類ルール、例）
- TODO: Safety Actions Matrix の **完全表（状態×許可行為）** の実データ
- TODO: degraded運転の **状態一覧・遷移条件**（state machineの具体）
- TODO: Evidence Support Bundle の **ファイル構造・秘匿方式・署名/ハッシュ・保全手順**（チェーン・オブ・カストディの具体）
- TODO: severity mapping（P0/P1/P2）の **客観ルール**（定量条件、例外）
- TODO: CI Gate の **実装仕様**（lintルール詳細、schema一覧、FAIL条件の厳密定義）
- TODO: runbooks.yaml / drills.yaml / coverage_matrix の **スキーマ定義**（contractsの中身）
- TODO: Windows本番運用（PowerShell正本）の **コマンド集と安全ガード** の具体
