# Domain O — Deterministic Replay / Audit Event Log（Level 1 SSOT Outline）

> Source: O.txt :contentReference[oaicite:0]{index=0}

## 1.1 目的と到達点（Non-negotiable）
- **目的**：システム全体の再現性（Determinism）／説明責任（Accountability）／証跡保全（Forensics）を担保する。監査イベントログ（append-only）と決定論リプレイ（replay+diff）を中核に、verify/bundle/attestation/report/migration/scrub/query completeness/access/sanitize/budget/partial failure/auto escalation/compliance gate/KPI/breach policy/cadence/alert/incident ledger/SBOMまで一気通貫で成立させる。拡張（O-101〜O-108）で外部共有や高度ガバナンスにも対応する。 :contentReference[oaicite:1]{index=1}
- **到達点（完了ライン）**：canonical schemaでの重要イベント記録、hash chain+checkpoint検証、任意run replayと主要出力自動検証、秘密情報ゼロの機械保証、各種障害でも証跡破綻しない、過去run可読と検索完全性の説明、Decision Explanation Capsule即時提示、CI/リリースでO準拠強制、監視→インシデント運用履歴まで監査で閉じる、拡張も同規律で維持。 :contentReference[oaicite:2]{index=2}

## 1.2 スコープとSSOT境界
- **OがSSOTとして保持（MUST）**：run内順序、事実event、完全性証跡、検証結果、メタ監査、gap taxonomy/expected event contract、query completeness/explain、migration/compat matrix、granularity/budget/auto escalation、access/sanitized export、各種ledger（revision/ML audit/feature flag/release/deletion/license/ingestion/transport/time source/ID collision/boundary checksums 等）、compliance gate/KPI/breach policy/cadence、alert/incident連携、policy reference hash binding。 :contentReference[oaicite:3]{index=3}
- **他ドメイン参照**：H/N（市場データ本体は参照ID＋整合性をOで扱う）、I（注文実行応答の最小安全digestはO保持）、J（適用ルール集合・説明カプセルはO保持）、C（metrics整合結果はO保持）、D/Y（bundle/attestation/report仕様はOが定義）。 :contentReference[oaicite:4]{index=4}

## 1.3 用語（O内SSOT）
- run_id / event_id（ULID推奨）/ seq / event_time・recv_time・persist_time / integrity_root
- dataset_ref・code_ref・config_ref・env_ref（hash binding可能に）
- repro_tier（Tier0〜3）/ forensic・perf（二層）/ completeness_level（L0〜L3）/ EXTENSION（採用時にmanifest/契約/CIへ統合） :contentReference[oaicite:5]{index=5}

## 1.4 参照アーキテクチャ（MUST）
- Emitter SDK（Rust/Python）
- WAL Writer（append-only）
- Integrity Builder（hash/checkpoint/署名/アンカー）
- Index/Query（検索＋完全性宣言）
- Replay Engine（pure/hybrid/audit）
- Verifier/Diff（差分最小化）
- Export/Bundle/Attestation/Report（テンプレ固定）
- Governance（CIゲート、KPI、cadence、migration、scrub） :contentReference[oaicite:6]{index=6}

## 1.5 Run開始時固定宣言セット（MUST）
RunStarted直後（または同一Txn）に宣言・固定：
- Repro tier / Ordering policy・Durability mode・Unit policy
- Audit granularity・Audit budget
- External dependency profile / Expected event contract / Global ID policy
- Transport path・Time source / Feature flag snapshot・Run linked to release
- Retention policy / Access policy & field visibility ref
- 必要時：Migration plan / Sanitized export profile
- policy_refはhash binding（PolicyRefResolved）で固定 :contentReference[oaicite:7]{index=7}

## 1.6 Canonical Event Schema（MUST）
### 1.6.1 共通ヘッダ（MUST）
- schema_version
- run_id, event_id, seq
- event_type, severity
- event_time/recv_time/persist_time
- trace_id/span_id
- producer(name+version), producer_instance_id, boot_id
- code_ref/config_ref/dataset_ref/env_ref
- env_fingerprint/runtime_fingerprint
- build_artifact_digest/build_profile（toolchain/flags含む）
- repro_tier
- ordering_policy_ref, durability_mode
- audit_granularity_profile_ref, audit_budget_profile_ref
- unit_policy_ref
- access_policy_version, field_visibility_profile_ref
- prev_hash/hash
- correlation_id/parent_event_id
- txn_id（採用時）
- redaction_level（原則NONE） :contentReference[oaicite:8]{index=8}

### 1.6.2 canonicalization & digest（MUST）
- 主要入力・状態・判断は正規化しdigest化。diffはdigest差分から原因候補提示。 :contentReference[oaicite:9]{index=9}
- TODO: 正規化ルール（文字列/数値/配列順序/浮動小数等）定義
- TODO: digest/hashアルゴリズム、エンコード、互換ポリシー

## 1.7 コア機能（MUST）
- Append-only WAL：セグメント形式（length-prefix）、回転/保持（hot/warm/cold）、クラッシュ復旧（partial write検出） :contentReference[oaicite:10]{index=10}
- Integrity：hash chain、checkpoint、verify（VALID/INVALID/INCOMPLETE）、Continuous verify（run中部分検証） :contentReference[oaicite:11]{index=11}
- Determinism Contract：clock注入、rng注入、順序完全化、整数化、外部I/O禁止（replay）、非決定要因検知（NondeterminismDetected） :contentReference[oaicite:12]{index=12}
- Replay/Verify/Diff：pure/hybrid/audit replay、重要イベント比較、差分最小化、因果鎖再構成 :contentReference[oaicite:13]{index=13}
- DLP：構造防止＋検知＋allowlist、違反は拒否/隔離、bundle/report生成時scan必須 :contentReference[oaicite:14]{index=14}
- TODO: 各機能のI/O（API）、データ保持先、障害時の具体的復旧手順の詳細

## 1.8 監査品質強化（MUST/SHOULD）
- MUST：Expected event contract / Gap taxonomy / Minimal external transcript / Semantic validation / Verifier provenance & self-test / Integrity summary / Repro tier自動降格/回復
- SHOULD：Dual evaluation（二重計算）/ Trusted timestamp拡張 :contentReference[oaicite:15]{index=15}

## 1.9 現場耐性（MUST）
- Audit granularity & budget（超過宣言）（MUST）
- Txn markers（SHOULD）
- Partial failure semantics（MUST）
- Auto escalation（MUST）
- Critical event mirroring（SHOULD）
- Rate limit ledger（MUST）
- Transport audit（MUST） :contentReference[oaicite:16]{index=16}

## 1.10 権限・外部共有・人間要因（MUST）
- Field visibility map + access decision audit（MUST）
- Sanitized export contract（MUST）
- Two-person rule（SHOULD）
- Deletion transparency ledger（MUST）
- Feature flag audit（MUST）
- Release boundary markers（MUST） :contentReference[oaicite:17]{index=17}

## 1.11 長期運用（MUST）
- Scrub（破損検知）（MUST）
- Migration playbook & compatibility matrix（MUST）
- Query completeness & explain query（MUST）
- Policy reference hash binding（MUST） :contentReference[oaicite:18]{index=18}

## 1.12 監視・インシデント統合（MUST）
- Alert-to-audit link（MUST）
- Incident case ledger（MUST） :contentReference[oaicite:19]{index=19}

## 1.13 外部境界（MUST）
- Time source evidence（MUST）
- Global ID policy & collision audit（MUST）
- Boundary checksums（MUST）
- Data license & attribution evidence（MUST） :contentReference[oaicite:20]{index=20}

## 1.14 出力パッケージ（MUST）
- Bundle completeness level（L0〜L3）（MUST）
- Attestation package（MUST）
- Human report SSOT（テンプレ固定＋hash）（MUST） :contentReference[oaicite:21]{index=21}

## 1.15 組織規律（MUST）
- Compliance gate（CI/Release強制）（MUST）
- Audit coverage KPI（MUST）
- Breach handling policy（MUST）
- Integration templates（MUST）
- Operational cadence SSOT（MUST） :contentReference[oaicite:22]{index=22}

## 1.16 ML使用時追加（MUST if used）
- ModelRefAttached / FeatureSetDeclared / InferenceExecuted
- 拡張で drift/fairness（O-107）を追加可能 :contentReference[oaicite:23]{index=23}

## 1.17 OPTIONAL拡張（O-87〜O-94）
- OPTIONAL。採用時はmanifest/契約/CIへ組み込み必須。 :contentReference[oaicite:24]{index=24}
- O-87 Retro-Detail Buffer
- O-88 Snapshot Delta Encoding
- O-89 Critical Event App-Signing
- O-90 Materialized Causality Graph
- O-91 Counterfactual Explanation Expansion
- O-92 Segment/Window Replay
- O-93 Multi-source Consistency Check
- O-94 Chain Fork Isolation on Corruption :contentReference[oaicite:25]{index=25}

## 1.18 推奨追加（O-95〜O-100：SHOULD）
- O-95 Delta Export & Partial Sync
- O-96 At-Rest Encryption Profiles
- O-97 Threat Model Declaration
- O-98 Schema Codegen SSOT
- O-99 What-If Replay Sandbox
- O-100 Fault Injection & Chaos Evidence :contentReference[oaicite:26]{index=26}

## 1.19 EXTENSION（O-101〜O-108：拡張）
- 採用時は、O-67（manifest）へ追加し、ExpectedEventContract・CI・KPI・BreachPolicyへ統合。 :contentReference[oaicite:27]{index=27}
- O-101 Privacy-Preserving Audit Stats
- O-102 Confidential Computing Ready（TEE Evidence）
- O-103 Encrypted Search / Searchable Encryption
- O-104 Multi-party Consensus Anchoring
- O-105 RCA Hypothesis Artifact
- O-106 Runbook Automation Evidence
- O-107 ML Drift/Fairness Governance
- O-108 Living Documentation Generator :contentReference[oaicite:28]{index=28}

## 1.20 API/CLI（MUST）
- verify / continuous-verify
- replay / diff
- export（L0..L3）/ attestation / report
- query（--explain --completeness）
- scrub / migrate / rebuild-index
- compliance-check / coverage
- alert/incident link / ratelimit
- （採用時）replay-window / what-if / fault-injection / delta-sync / encrypted-search 等 :contentReference[oaicite:29]{index=29}
- TODO: 各コマンドの引数・戻り値・エラーコード・出力フォーマット（SSOT）

## 1.21 テスト（MUST）
CIで最低限：
- schema互換
- expected event contract（missing required 0）
- DLP（秘密混入0）
- determinism golden
- tamper
- crash recovery
- continuous verify
- query completeness/explain
- migration/matrix/rebuild
- scrub/破損検知
- alert/incident整合
- rate limit ledger
- policy hash binding
- access/sanitize/deletion/approval/flags/release整合
- 外部境界（transport/time/id/checksum/license）
- OPTIONAL/SHOULD/EXTENSION採用時：manifestで必須化し専用テスト追加 :contentReference[oaicite:30]{index=30}
- TODO: テストベクタ（golden）と期待結果のSSOT配置（例：O-74）詳細

## 1.22 機能分解（Capability Index：O-01〜O-108）
> **IDは必ず保持**（O-01〜O-108）。 :contentReference[oaicite:31]{index=31}

### O-01〜O-12（基礎コア）
- O-01 Canonical Event Schema SSOT
- O-02 Event Emitter SDK
- O-03 Append-only WAL Writer
- O-04 Integrity（hash/checkpoint/verify）
- O-05 Index/Query
- O-06 Determinism Contract
- O-07 Replay Engine
- O-08 Replay Verifier/Diff
- O-09 Export/Support Bundle
- O-10 Proof Strengthening
- O-11 Ops/Observability
- O-12 Access Audit & Retention :contentReference[oaicite:32]{index=32}

### O-13〜O-18（再現性/調査強化）
- O-13 Reproducibility Tier
- O-14 Nondeterminism Detector
- O-15 Forensic Minimum Set
- O-16 Causal Linking
- O-17 Failover-Safe Audit Log
- O-18 INCOMPLETE Handling :contentReference[oaicite:33]{index=33}

### O-19〜O-27（本番耐性/提出）
- O-19 Dual-Plane Logging
- O-20 Critical Event Mirroring
- O-21 Capacity-Safe Policy
- O-22 Dedup & Idempotency
- O-23 Cross-Run Causality
- O-24 State Snapshot
- O-25 Meta-Audit
- O-26 External Report Format
- O-27 Encrypted Bundle Option :contentReference[oaicite:34]{index=34}

### O-28〜O-35（信頼性強化）
- O-28 Producer Identity & Attestation
- O-29 Time Discipline & Clock State
- O-30 Crash Forensics
- O-31 Invariant Violation Auditing
- O-32 Telemetry Gap Auditing
- O-33 External Dependency Freeze Profile
- O-34 Continuous Verification
- O-35 Human Report SSOT :contentReference[oaicite:35]{index=35}

### O-36〜O-43（欠損/契約/環境/操作）
- O-36 Gap Taxonomy & Responsibility
- O-37 Expected Event Contract
- O-38 Minimal External Transcript
- O-39 Environment & Runtime Fingerprint
- O-40 Ordering Policy SSOT
- O-41 Durability Mode SSOT
- O-42 Time Anomaly Impact
- O-43 Operator Action Auditing :contentReference[oaicite:36]{index=36}

### O-44〜O-53（証明/長期運用/検索）
- O-44 Evidence Bundle Completeness
- O-45 Dual Evaluation
- O-46 Verifier Provenance & Self-Test
- O-47 Semantic Validation
- O-48 Trusted Timestamp Extension
- O-49 Scrub & Corruption Defense
- O-50 Audit Granularity Profile
- O-51 Integrity Summary
- O-52 Migration Playbook & Compatibility Matrix
- O-53 Query Completeness & Explain Query :contentReference[oaicite:37]{index=37}

### O-54〜O-66（現場運用の最終強化）
- O-54 Audit Budgeting & Drop Declaration
- O-55 Event Transaction Markers
- O-56 Role-bound Field Visibility & Access Decision Audit
- O-57 Sanitized Export Contract
- O-58 SBOM & Dependency Evidence
- O-59 Partial Failure Semantics
- O-60 Auto Escalation Rules
- O-61 Audit Attestation Package
- O-62 Unit & Conversion Audit
- O-63 Metrics-to-Events Reconciliation
- O-64 Decision Explanation Capsule
- O-65 Data Revision Ledger
- O-66 ML Feature & Model Audit :contentReference[oaicite:38]{index=38}

### O-67〜O-86（規律＋外部境界／監視リンク）
- O-67 Compliance Gate
- O-68 Audit Coverage Metrics
- O-69 Breach Handling Policy
- O-70 Integration Templates
- O-71 Operational Cadence SSOT
- O-72 Feature Flag Audit
- O-73 Release Boundary Markers
- O-74 Determinism Test Vectors SSOT
- O-75 Two-person Rule
- O-76 Deletion Transparency Ledger
- O-77 Ingestion/Transport Audit
- O-78 Time Source Evidence
- O-79 Global ID Policy & Collision Audit
- O-80 Boundary Checksums
- O-81 Data License & Attribution Evidence
- O-82 Alert-to-Audit Link
- O-83 Incident Case Ledger
- O-84 Rate Limit Ledger
- O-85 Repro Tier Auto Downgrade/Restore
- O-86 Policy Reference Resolver（Hash Binding） :contentReference[oaicite:39]{index=39}

### O-87〜O-94（OPTIONAL）
- O-87 Retro-Detail Buffer
- O-88 Snapshot Delta Encoding
- O-89 Critical Event App-Signing
- O-90 Materialized Causality Graph
- O-91 Counterfactual Explanation Expansion
- O-92 Segment/Window Replay
- O-93 Multi-source Consistency Check
- O-94 Chain Fork Isolation on Corruption :contentReference[oaicite:40]{index=40}

### O-95〜O-100（SHOULD）
- O-95 Delta Export & Partial Sync
- O-96 At-Rest Encryption Profiles
- O-97 Threat Model Declaration
- O-98 Schema Codegen SSOT
- O-99 What-If Replay Sandbox
- O-100 Fault Injection & Chaos Evidence :contentReference[oaicite:41]{index=41}

### O-101〜O-108（EXTENSION）
- O-101 Privacy-Preserving Audit Stats
- O-102 Confidential Computing Ready（TEE Evidence）
- O-103 Encrypted Search / Searchable Encryption
- O-104 Multi-party Consensus Anchoring
- O-105 RCA Hypothesis Artifact
- O-106 Runbook Automation Evidence
- O-107 ML Drift/Fairness Governance
- O-108 Living Documentation Generator :contentReference[oaicite:42]{index=42}

## 1.23 完了条件（Oを“完全実装済み”と見なす判定）
- O-01〜O-86 実装＋CIゲート（O-67）で強制
- 主要runが最低Tier要件を満たし、Tier自動降格/回復（O-85）が機能
- verify/continuous verify/attestation/report が運用導線として成立
- 長期運用（scrub/migration/query completeness）が回っている
- 外部境界（transport/time/id/checksum/license）が監査できる
- alert/incident が run と結びつき、運用履歴まで追跡できる
- OPTIONAL/SHOULD/EXTENSION採用時：manifest/契約/CI/KPI/運用ケイデンスに統合 :contentReference[oaicite:43]{index=43}
