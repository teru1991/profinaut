# Level 1 SSOT Outline — U: Release / Supply Chain / Hardening（v1.7 / U-01〜U-120）

> 元資料：実装目標機能詳細設計（最終清書 v1.7 / U-01〜U-120）:contentReference[oaicite:0]{index=0}  
> ドメイン文字：**U**（Release / Supply Chain / Hardening）

---

## 0. SSOT準拠（Uの定義）

### 0.1 Uの必達（SSOT）
- **SBOM、依存スキャン、署名付きリリース**:contentReference[oaicite:1]{index=1}  
- **最小権限、秘密情報の赤塗り、監査ログ保全**:contentReference[oaicite:2]{index=2}  

### 0.2 SSOTの正本（TODO）
- TODO: UにおけるSSOTの格納場所（リポジトリ/パス/ブランチ/タグ運用）
- TODO: SSOT更新手順（変更提案→レビュー→承認→署名→昇格）
- TODO: SSOTバージョニング規約（SemVer等）と互換性方針

---

## 1. 目的（Non-negotiable）

1. 改ざん・混入・漏洩・誤配布・脆弱依存を工程と実行時の両面で防ぐ:contentReference[oaicite:3]{index=3}  
2. リリース成果物が誰でも同一手順で検証可能（証明可能）:contentReference[oaicite:4]{index=4}  
3. 例外や緊急対応も統制された手順で実施され、証跡が残る:contentReference[oaicite:5]{index=5}  
4. 事故時に停止・切り戻し・前進修正・再署名・再配布を最短で実行できる:contentReference[oaicite:6]{index=6}  
5. 取引システムとして性能劣化・導入失敗・人的ミスも“供給網品質”として抑止する:contentReference[oaicite:7]{index=7}  
6. 定義した制御が実際に効いている（Assurance）ことを機械的に証明できる:contentReference[oaicite:8]{index=8}  

---

## 2. スコープ（Uが責任を持つ範囲）

- リリース定義SSOT、成果物・設定・運用資産の整合性保証:contentReference[oaicite:9]{index=9}  
- provenance（出自）・SBOM（内容）・署名（真正性）・透明性（差替え検知）:contentReference[oaicite:10]{index=10}  
- 依存スキャン（SCA）＋到達可能性補助＋例外管理（期限つき）:contentReference[oaicite:11]{index=11}  
- 秘密情報赤塗り（ログ/診断/設定/エラー/全出力経路）:contentReference[oaicite:12]{index=12}  
- 最小権限ハードニング（実行環境/コンテナ/ネットワーク/危険設定禁止）:contentReference[oaicite:13]{index=13}  
- 監査ログ保全（ビルド/署名/配布/例外/強操作/意思決定/削除）:contentReference[oaicite:14]{index=14}  
- ロールバック/ロールフォワード、鍵漏洩対応、緊急運用統制:contentReference[oaicite:15]{index=15}  
- 更新機構の安全化（導入側検証・アップデート信頼）:contentReference[oaicite:16]{index=16}  
- 性能/可用性/導入品質をリリースゲートに統合:contentReference[oaicite:17]{index=17}  
- 制御カバレッジ・抜け道検知・監査の監査（Assurance）:contentReference[oaicite:18]{index=18}  

---

## 3. 必須成果物（Uで必ず揃うもの）

- release.manifest.yaml（リリース定義SSOT）:contentReference[oaicite:19]{index=19}  
- release_evidence_index.json（証跡インデックス：成果物URL+hash+署名参照）:contentReference[oaicite:20]{index=20}  
- provenance.json:contentReference[oaicite:21]{index=21}  
- sbom.(cdx|spdx).json:contentReference[oaicite:22]{index=22}  
- toolchain_sbom.json:contentReference[oaicite:23]{index=23}  
- vuln_report.json + policy_decision.json（SCA）:contentReference[oaicite:24]{index=24}  
- 署名（成果物＋メタデータ一式）＋透明性ログ記録（存在する場合）:contentReference[oaicite:25]{index=25}  
- verify-release CLI（導入前/導入時検証）:contentReference[oaicite:26]{index=26}  
- audit_events.log（append-only監査イベント）:contentReference[oaicite:27]{index=27}  
- exception_ledger.jsonl（例外台帳SSOT）:contentReference[oaicite:28]{index=28}  
- decision_log.jsonl（意思決定ログSSOT）:contentReference[oaicite:29]{index=29}  
- security_posture_report.md（姿勢レポート）:contentReference[oaicite:30]{index=30}  
- release_notes.md（自動生成＋レビュー強制）:contentReference[oaicite:31]{index=31}  
- perf_report.json（性能回帰）:contentReference[oaicite:32]{index=32}  
- capacity_degradation_report.json（容量・劣化検証）:contentReference[oaicite:33]{index=33}  
- control_coverage.json（U制御カバレッジ）:contentReference[oaicite:34]{index=34}  
- approval_snapshot.(json|zip)（stable最終承認スナップショット）:contentReference[oaicite:35]{index=35}  

### 3.1 成果物フォーマット/スキーマ（TODO）
- TODO: 各成果物の必須フィールド、署名対象範囲、ハッシュ計算方式、保管先
- TODO: 生成者（CIジョブ/手動/ツール）と生成タイミング（build/release/promote/install）

---

## 4. 機能（Capabilities）

> 既存のカテゴリ（A〜I）を 4.x として保持し、配下にU-xxを列挙する。:contentReference[oaicite:36]{index=36}  

### 4.1 A. コア（SSOT必達：U-01〜U-10）
- U-01 Release Manifest SSOT（定義固定）:contentReference[oaicite:37]{index=37}  
- U-02 Build Provenance（出自生成）:contentReference[oaicite:38]{index=38}  
- U-03 SBOM生成・保管・差分:contentReference[oaicite:39]{index=39}  
- U-04 SCA＋ポリシーゲート（例外台帳）:contentReference[oaicite:40]{index=40}  
- U-05 署名付きリリース（成果物＋メタデータ）:contentReference[oaicite:41]{index=41}  
- U-06 verify-release（検証CLI）:contentReference[oaicite:42]{index=42}  
- U-07 Redaction（秘密赤塗り）:contentReference[oaicite:43]{index=43}  
- U-08 Runtime Hardening（最小権限＋自己診断）:contentReference[oaicite:44]{index=44}  
- U-09 Audit Events（監査ログ保全）:contentReference[oaicite:45]{index=45}  
- U-10 Release Gate（最後の関所）:contentReference[oaicite:46]{index=46}  

### 4.2 B. 供給網強化（U-11〜U-20）
- U-11 Repo Integrity Gate:contentReference[oaicite:47]{index=47}  
- U-12 Dependency Allowlist / Lockdown:contentReference[oaicite:48]{index=48}  
- U-13 Hardened CI Runner（権限分離）:contentReference[oaicite:49]{index=49}  
- U-14 Transparency Log＋Revocation:contentReference[oaicite:50]{index=50}  
- U-15 Release Promotion（再ビルド禁止昇格）:contentReference[oaicite:51]{index=51}  
- U-16 Reproducible Build Check:contentReference[oaicite:52]{index=52}  
- U-17 Secret Scanner（混入検知）:contentReference[oaicite:53]{index=53}  
- U-18 Config/Artifact Integrity:contentReference[oaicite:54]{index=54}  
- U-19 Audit Schema固定:contentReference[oaicite:55]{index=55}  
- U-20 Forbidden/Content Policy Check:contentReference[oaicite:56]{index=56}  

### 4.3 C. 運用・法務・事故対応（U-21〜U-35）
- U-21 License Gate:contentReference[oaicite:57]{index=57}  
- U-22 Two-person rule（多段承認）:contentReference[oaicite:58]{index=58}  
- U-23 Artifact Registry SSOT:contentReference[oaicite:59]{index=59}  
- U-24 Time & Expiry Policy:contentReference[oaicite:60]{index=60}  
- U-25 Post-release Integrity Monitor:contentReference[oaicite:61]{index=61}  
- U-26 Rollback Playbook:contentReference[oaicite:62]{index=62}  
- U-27 Release Channels:contentReference[oaicite:63]{index=63}  
- U-28 Build Flag Policy:contentReference[oaicite:64]{index=64}  
- U-29 Egress Inventory:contentReference[oaicite:65]{index=65}  
- U-30 Supplier Risk Policy:contentReference[oaicite:66]{index=66}  
- U-31 Environment Parity:contentReference[oaicite:67]{index=67}  
- U-32 Failure Policy:contentReference[oaicite:68]{index=68}  
- U-33 Threat Model:contentReference[oaicite:69]{index=69}  
- U-34 Secure Baseline Snapshot:contentReference[oaicite:70]{index=70}  
- U-35 Key Compromise Runbook:contentReference[oaicite:71]{index=71}  

### 4.4 D. 抜け穴潰し最終強化（U-36〜U-52）
- U-36 Attack Surface Minimization:contentReference[oaicite:72]{index=72}  
- U-37 Diff Risk Scoring:contentReference[oaicite:73]{index=73}  
- U-38 Malware Heuristics Gate:contentReference[oaicite:74]{index=74}  
- U-39 Hermetic Inputs Policy:contentReference[oaicite:75]{index=75}  
- U-40 Key Usage Policy:contentReference[oaicite:76]{index=76}  
- U-41 Evidence WORM:contentReference[oaicite:77]{index=77}  
- U-42 Release Notes as Code:contentReference[oaicite:78]{index=78}  
- U-43 Breaking Change Gate:contentReference[oaicite:79]{index=79}  
- U-44 Emergency Release Mode:contentReference[oaicite:80]{index=80}  
- U-45 Forward Fix Policy:contentReference[oaicite:81]{index=81}  
- U-46 Retention Policy:contentReference[oaicite:82]{index=82}  
- U-47 Security Regression Suite:contentReference[oaicite:83]{index=83}  
- U-48 Install-time Verification:contentReference[oaicite:84]{index=84}  
- U-49 Security Posture Report:contentReference[oaicite:85]{index=85}  
- U-50 Deprecation & Replacement Plan:contentReference[oaicite:86]{index=86}  
- U-51 Release Network Segmentation:contentReference[oaicite:87]{index=87}  
- U-52 Decision Log:contentReference[oaicite:88]{index=88}  

### 4.5 E. 標準準拠・深度テスト・更新機構（U-53〜U-70）
- U-53 Secure Update（TUF相当）:contentReference[oaicite:89]{index=89}  
- U-54 Binary Transparency:contentReference[oaicite:90]{index=90}  
- U-55 Step Attestation（in-toto相当）:contentReference[oaicite:91]{index=91}  
- U-56 SLSA目標の明文化:contentReference[oaicite:92]{index=92}  
- U-57 Toolchain SBOM:contentReference[oaicite:93]{index=93}  
- U-58 Build Script Immutability:contentReference[oaicite:94]{index=94}  
- U-59 Split Build / Split Sign:contentReference[oaicite:95]{index=95}  
- U-60 Runtime Attestation:contentReference[oaicite:96]{index=96}  
- U-61 Sandbox Policy:contentReference[oaicite:97]{index=97}  
- U-62 Fuzzing / Property Tests:contentReference[oaicite:98]{index=98}  
- U-63 Critical Duality（二重検証）:contentReference[oaicite:99]{index=99}  
- U-64 Upper Bound Policy:contentReference[oaicite:100]{index=100}  
- U-65 Supply Chain Drill（演習）:contentReference[oaicite:101]{index=101}  
- U-66 Evidence Restore（証跡復旧）:contentReference[oaicite:102]{index=102}  
- U-67 Forbidden Contents拡張:contentReference[oaicite:103]{index=103}  
- U-68 Signed Dependencies:contentReference[oaicite:104]{index=104}  
- U-69 Security Changelog Gate:contentReference[oaicite:105]{index=105}  
- U-70 Policy Engine:contentReference[oaicite:106]{index=106}  

### 4.6 F. 人・環境・長期運用の穴埋め（U-71〜U-88）
- U-71 Cache Poisoning Defense:contentReference[oaicite:107]{index=107}  
- U-72 Dependency Confusion / Typosquat対策:contentReference[oaicite:108]{index=108}  
- U-73 Key Recovery Policy:contentReference[oaicite:109]{index=109}  
- U-74 Privileged Action Controls（特権操作統制）:contentReference[oaicite:110]{index=110}  
- U-75 Release Freeze Window:contentReference[oaicite:111]{index=111}  
- U-76 Offline Verification Mode:contentReference[oaicite:112]{index=112}  
- U-77 Secure Deletion & Legal Hold:contentReference[oaicite:113]{index=113}  
- U-78 Threshold by Environment:contentReference[oaicite:114]{index=114}  
- U-79 Signed Config Bundles:contentReference[oaicite:115]{index=115}  
- U-80 Diff Review Enforcement:contentReference[oaicite:116]{index=116}  
- U-81 Verifier Compatibility Contract:contentReference[oaicite:117]{index=117}  
- U-82 Audit Completeness Check:contentReference[oaicite:118]{index=118}  
- U-83 Reachability Assist:contentReference[oaicite:119]{index=119}  
- U-84 Non-determinism Analyzer:contentReference[oaicite:120]{index=120}  
- U-85 Docs/Runbook/Policy署名:contentReference[oaicite:121]{index=121}  
- U-86 Access Review:contentReference[oaicite:122]{index=122}  
- U-87 Registry Outage Mode:contentReference[oaicite:123]{index=123}  
- U-88 Supply Chain SLOs:contentReference[oaicite:124]{index=124}  

### 4.7 G. 取引システム級（性能・可用性・導入品質）（U-89〜U-100）
- U-89 Performance Regression Gate（性能SLOゲート）:contentReference[oaicite:125]{index=125}  
- U-90 Signed Benchmark Environment（署名済みベンチ環境）:contentReference[oaicite:126]{index=126}  
- U-91 Capacity & Degradation Tests（容量・劣化検証）:contentReference[oaicite:127]{index=127}  
- U-92 Operational Footgun Guard（危険操作封じ）:contentReference[oaicite:128]{index=128}  
- U-93 Observability Supply Chain（観測I/Fの契約固定）:contentReference[oaicite:129]{index=129}  
- U-94 Repro Support Sandbox（安全な調査環境）:contentReference[oaicite:130]{index=130}  
- U-95 Emergency Kill-Switch Distribution（緊急停止の署名配布）:contentReference[oaicite:131]{index=131}  
- U-96 Zero Manual Release（人的手順ゼロ化）:contentReference[oaicite:132]{index=132}  
- U-97 Release Rehearsal（完全リハーサル）:contentReference[oaicite:133]{index=133}  
- U-98 Self-healing Install（導入失敗の自己回復）:contentReference[oaicite:134]{index=134}  
- U-99 Cost Regression Guard（コスト回帰ガード）:contentReference[oaicite:135]{index=135}  
- U-100 Adoption Levels（段階適用モード）:contentReference[oaicite:136]{index=136}  

### 4.8 H. Uを破綻させない“固定化”（SSOT化）（U-101〜U-110）
- U-101 Policy Catalog SSOT（policy_set_versionを刻む）:contentReference[oaicite:137]{index=137}  
- U-102 Release Evidence Index（証跡インデックス必須化）:contentReference[oaicite:138]{index=138}  
- U-103 Exception Ledger SSOT（例外台帳フォーマット統一）:contentReference[oaicite:139]{index=139}  
- U-104 Audit Required Set（必須監査イベントの規範化）:contentReference[oaicite:140]{index=140}  
- U-105 Verification Compatibility Contract SSOT（検証仕様の版管理）:contentReference[oaicite:141]{index=141}  
- U-106 Roles & Permissions SSOT（署名/配布/例外/削除等のロール固定）:contentReference[oaicite:142]{index=142}  
- U-107 Emergency Ops SSOT（緊急緩和/緊急停止の運用規範）:contentReference[oaicite:143]{index=143}  
- U-108 Perf Baseline SSOT（性能基準セットの版管理）:contentReference[oaicite:144]{index=144}  
- U-109 Install & Update SSOT（導入/更新手順の正本化＋署名対象）:contentReference[oaicite:145]{index=145}  
- U-110 Adoption Level + Coverage（達成レベルと適用範囲の可視化）:contentReference[oaicite:146]{index=146}  

### 4.9 I. Assurance（実際に“効いている”ことの保証）（U-111〜U-120）
- U-111 Control Coverage（制御適用範囲の機械測定）:contentReference[oaicite:147]{index=147}  
- U-112 Bypass Detection Tests（抜け道混入テストでGate検証）:contentReference[oaicite:148]{index=148}  
- U-113 Adversarial Review Mode（攻撃者視点レビューの制度化）:contentReference[oaicite:149]{index=149}  
- U-114 Risk Label Gate（高リスク変更で検査強度自動増）:contentReference[oaicite:150]{index=150}  
- U-115 Verifier Unavailability Policy（検証不能時の判定規約）:contentReference[oaicite:151]{index=151}  
- U-116 Dependency Update Rollout（依存更新の段階ロールアウト）:contentReference[oaicite:152]{index=152}  
- U-117 Audit of Audit（監査ログの監査＝証跡健全性検査）:contentReference[oaicite:153]{index=153}  
- U-118 Approval Snapshot（stable最終承認スナップショット署名保存）:contentReference[oaicite:154]{index=154}  
- U-119 Change-to-Release Trace（チケット→PR→監査→証跡の連結）:contentReference[oaicite:155]{index=155}  
- U-120 Complexity Budget（複雑性予算で運用崩壊を防止）:contentReference[oaicite:156]{index=156}  

---

## 5. Done（完全実装の定義）

1. 全成果物が署名され verify-release で検証できる:contentReference[oaicite:157]{index=157}  
2. SBOM＋provenance（＋可能ならattestation）が必ず生成され hash整合:contentReference[oaicite:158]{index=158}  
3. SCAは必須ゲート、例外は期限つき台帳＋監査＋（stable相当は）二者承認:contentReference[oaicite:159]{index=159}  
4. ログ/診断/設定/エラー/出力経路から秘密漏洩ゼロ（回帰テスト保証）:contentReference[oaicite:160]{index=160}  
5. 最小権限基準があり、逸脱は起動時に検知（Failure Policyに従う）:contentReference[oaicite:161]{index=161}  
6. build/署名/配布/削除/例外/緊急/鍵/凍結/権限が監査＋意思決定ログに残る:contentReference[oaicite:162]{index=162}  
7. 配布後も整合性監視・停止・切り戻し・前進修正・鍵漏洩対応が可能:contentReference[oaicite:163]{index=163}  
8. ビルド入力（依存/外部参照/ネットワーク/キャッシュ）が固定され、混入余地が最小:contentReference[oaicite:164]{index=164}  
9. 長期運用（互換性契約、証跡復旧、SLO監視）で安全が劣化しない:contentReference[oaicite:165]{index=165}  
10. 性能/可用性/導入品質がリリースゲートで担保される:contentReference[oaicite:166]{index=166}  
11. Uの制御が適用されていること（Coverage）と抜け道が塞がっていること（Bypass Test）を証明できる:contentReference[oaicite:167]{index=167}  
12. Uの複雑性が予算内で維持され、運用崩壊しない:contentReference[oaicite:168]{index=168}  

---

## 6. Capability Index（U-01〜U-120）

> IDは保持し、カテゴリ（A〜I）へ所属づける（要件：Capability Indexに入れる）。

- **A. コア（U-01〜U-10）**：U-01, U-02, U-03, U-04, U-05, U-06, U-07, U-08, U-09, U-10:contentReference[oaicite:169]{index=169}  
- **B. 供給網強化（U-11〜U-20）**：U-11, U-12, U-13, U-14, U-15, U-16, U-17, U-18, U-19, U-20:contentReference[oaicite:170]{index=170}  
- **C. 運用・法務・事故対応（U-21〜U-35）**：U-21…U-35:contentReference[oaicite:171]{index=171}  
- **D. 抜け穴潰し最終強化（U-36〜U-52）**：U-36…U-52:contentReference[oaicite:172]{index=172}  
- **E. 標準準拠・深度テスト・更新機構（U-53〜U-70）**：U-53…U-70:contentReference[oaicite:173]{index=173}  
- **F. 人・環境・長期運用の穴埋め（U-71〜U-88）**：U-71…U-88:contentReference[oaicite:174]{index=174}  
- **G. 取引システム級（U-89〜U-100）**：U-89…U-100:contentReference[oaicite:175]{index=175}  
- **H. 固定化（SSOT化）（U-101〜U-110）**：U-101…U-110:contentReference[oaicite:176]{index=176}  
- **I. Assurance（U-111〜U-120）**：U-111…U-120:contentReference[oaicite:177]{index=177}  

---

## 7. Level 2 Deep Spec 判定

- 入力には **Non-negotiable（目的）** は存在するが:contentReference[oaicite:178]{index=178}、**Canonical Model/Contract** と **Behavior/Tests** が「揃っている」と断定できるだけの明示が見当たらないため、**Level 2 Deep Spec は出力しない**（TODOのみ保持）。


分類理由: リリース運用・ビルド・デプロイ・システム強化（Hardening）に直接関わるため system に分類。
