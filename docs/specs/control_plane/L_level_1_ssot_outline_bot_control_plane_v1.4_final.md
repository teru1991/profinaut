# Level 1 SSOT Outline — L: Bot Control Plane（ボット管理・運用）SSOT v1.4 FINAL（変換版）

> 元文書:「L: Bot Control Plane — 実装目標機能 詳細設計（SSOT） v1.4 FINAL」:contentReference[oaicite:0]{index=0}  
> 本書は上記SSOTを **統一フォーマット（Level 1 SSOT Outline）** に再編したもの。**仕様の追加は行わない**（不足は TODO を残す）。

---

## 0. メタ情報
### 0.1 ドメイン
- ドメイン文字: **L**
- 対象大機能: **Bot Control Plane（ボット管理・運用の唯一の制御面）**

### 0.2 正本宣言（SSOT）
- 本ドメイン L は「ボットを安全に運用するための唯一の制御面」であり、Registry / Lifecycle / Safe Rollout / 資金割当 / 停止条件 / 監査追跡 / 強操作（Break-glass）を **強制** する。:contentReference[oaicite:1]{index=1}

### 0.3 更新履歴（要約）
- v1.0〜v1.4 FINAL の追加要素が列挙されている（詳細は元文書参照）。:contentReference[oaicite:2]{index=2}

---

## 1. Non-negotiable（絶対条件）
> L が満たすべき絶対条件（運用・安全・監査・供給網・人間ミス前提など）。:contentReference[oaicite:3]{index=3}

1. **全運用操作は監査可能**（who/when/what/why/approval/expiry/trace）。
2. **全操作は冪等**（request_id + fingerprint + dedupeで保証）。
3. **全起動は Preflight Gate 通過が必須**（通過しない限り実行されない）。
4. **Safety Controller（E）が最上位権限**（停止/SAFE_MODE等）で、Lは必ず追随。
5. **desired と actual の差分（drift）検知** と規定動作。
6. **Reconciliation / State Repair** による desired/actual 不整合修復。
7. **Split-brain 防止**（Lease/Fencing）。
8. **Stop/Pause の停止完了証明**（Stop Attestation）。
9. **強操作は Break-glass 経路のみ**、時間制限・承認・監査が必須。
10. **Two-person integrity**（申請者≠承認者強制、承認は差分指紋に紐づく）。
11. **Bulk Ops Safety**（対象スナップショット固定、進捗/中断/再試行）。
12. **Immutable artifacts**（digest不変参照、タグ禁止、配備時に検証）。
13. **Runtime safety contracts を Dual-fence で強制**（agent/runtime側でも強制）。
14. **Time-boxed overrides**（期限付き・自動失効）。
15. **安全が証明できること**（Release Safety Certificate / Tamper Evidence / 監査保持）。
16. **供給網事故前提**（署名・SBOM・検証ゲート）。
17. **人間ミス前提**（危険操作の摩擦・影響プレビュー・訓練モード）。
18. **Chaos/GameDays と Self-healing**（継続的破壊試験・上限固定）。
19. **Trust Levels**（信頼度に応じた許可/上限/監視変化）。

---

## 2. 境界（責務・非責務）
### 2.1 Lの責務（Control Plane）
- Bot定義（Registry）、Versioning（SemVer）、依存宣言、RBAC
- ライフサイクル（deploy/start/pause/stop/retire）
- Safe Rollout（shadow/canary/progressive + 自動停止/ロールバック）
- bot別の資金割当・停止条件（Risk Gate連動）
- 監査（命令→判断→発注→結果の完全追跡）
- Change Management/Approvals、Templates、Changelog品質ゲート
- Drift検知、Reconcile、Split-brain防止（Lease/Fencing）
- Incident Mode / Freeze、Maintenance Mode、Gold signals段階縮退
- Break-glass & Strong Ops
- Key-set switching/Rotation Orchestration、KeySet hygiene
- Scheduling/Limits、Quota & Budget Guard、Feature Flags
- Compliance Export、Audit Retention & Tamper Evidence
- Policy Simulation、Auto Triage、Policy Conflict Resolution
- Safe Sandbox/Paper、Experiments/A-B、Training mode
- Stop Attestation、UX dry-run/impact preview
- Immutable artifacts / digest verification
- Runtime safety contracts（Dual-fence）
- Control Plane DR & Recovery
- Version Lineage & Provenance
- Data Access Governance
- Time-boxed Overrides
- v1.4 FINAL追加（L-49〜L-56）：
  - Formal Verification Targets / Release Safety Certificate / Supply-chain Security Gates
  - Human Error Guardrails / Secrets/Privacy Audit / Chaos/GameDays Program
  - Self-Healing Policy / Bot Trust Levels :contentReference[oaicite:4]{index=4}

### 2.2 Lの非責務（ただし連携必須）
- 実際の発注執行: **I（Execution Platform）**
- 中央リスク判定: **J（Risk / Policy Gate）**
- 台帳・資金移動: **K（Portfolio / Treasury）**
- 実戦略の意思決定本体: **M（Strategy Runtime）**
- Market/IR/Onchain収集: **H（Market Data Platform）** :contentReference[oaicite:5]{index=5}

---

## 3. 用語（要点）
- Two-person integrity / Command staging / Bulk ops / Immutable artifacts
- Runtime safety contracts / Gold signals / Lease/Fencing / Stop attestation
- Release safety certificate / SBOM / Trust levels :contentReference[oaicite:6]{index=6}

---

## 4. Capabilities（サブドメイン）
### 4.1 サブドメイン一覧
- **L-01〜L-48**: v1.3の定義を正本とする（詳細は本入力には含まれない）:contentReference[oaicite:7]{index=7}  
  - TODO: L-01〜L-48 の各機能定義本文（v1.3 SSOT）を別入力として取り込み、同フォーマットへ分解。
- **v1.4 FINAL 追加**：:contentReference[oaicite:8]{index=8}
  - **L-49** Formal Verification Targets
  - **L-50** Release Safety Certificate
  - **L-51** Supply-chain Security Gates
  - **L-52** Human Error Guardrails
  - **L-53** Secrets/Privacy Audit
  - **L-54** Chaos/GameDays Program
  - **L-55** Self-Healing Policy
  - **L-56** Bot Trust Levels

### 4.2 L-49〜L-56（概要）
- L-49: 重要性質の機械検証対象（state_machine / lease_fencing / dedupe_idempotency / bulk_ops / stop_attestation）を固定し、参照（spec_ref）を保持、失敗時にrelease blocking gateにできる。:contentReference[oaicite:9]{index=9}
- L-50: リリース前/時に安全証明書を発行し、検証参照を束ね、manifest hash + 署名参照、重大インシデントでrevoke可能。:contentReference[oaicite:10]{index=10}
- L-51: artifactはdigest固定 + 署名検証、SBOM/脆弱性評価、deny理由コードとtriage、例外はTime-boxed overrideで期限必須。:contentReference[oaicite:11]{index=11}
- L-52: 高危険操作の friction（staging/二者承認/dry-run 等）、影響プレビュー、hard block条件で拒否（理由コード）。:contentReference[oaicite:12]{index=12}
- L-53: 監査ログ/エクスポート/トレースの秘密混入スキャン、高確度検知で即時措置（incident enable/rotate/stop/export revoke等）、全て監査イベント化。:contentReference[oaicite:13]{index=13}
- L-54: 定期破壊シナリオ実施で復旧/停止証明/監査証跡を継続的に証明し、失敗を改善へフィードバック。:contentReference[oaicite:14]{index=14}
- L-55: 自動復旧の段階/上限/回数/冷却/承認境界をSSOTで固定し誤爆抑制。:contentReference[oaicite:15]{index=15}
- L-56: 新規Botはexperimental/probation開始、実績・証明書・事故ゼロ期間で昇格、逸脱/違反/事故で降格（自動も可）、信頼度でrollout/limits/強操作/監視義務が変化。:contentReference[oaicite:16]{index=16}

---

## 5. Canonical Data Model（正規データモデル）
> v1.3までのEntitiesは維持。本入力は **v1.4追加分のみ** を定義。:contentReference[oaicite:17]{index=17}

### 5.1 FormalVerificationTarget（L-49）
- fv_id
- scope: state_machine | lease_fencing | dedupe_idempotency | bulk_ops | stop_attestation
- properties[]
- spec_ref（TLA+/モデル/証明スクリプト参照）
- verified_at, verified_by（自動/手動）
- blocking_gate: true（失敗時はrelease不可にできる）:contentReference[oaicite:18]{index=18}

### 5.2 ReleaseSafetyCertificate（L-50）
- cert_id
- bot_id, version
- issued_at
- attested_items（参照束ね）:
  - preflight_ok_ref / compatibility_ok_ref / artifact_digest_verified_ref / supply_chain_ok_ref
  - runbook_attached_ref / changelog_gate_ok_ref / rollout_plan_ref（任意）
  - stop_attestation_ref（必要時）
- manifest_hash
- signature_ref
- status: issued | revoked :contentReference[oaicite:19]{index=19}

### 5.3 SupplyChainGateReport（L-51）
- scg_id
- artifact_id
- sbom_ref
- signature_verified: bool
- vulnerability_summary_ref（重大/高/中/低）
- policy_decision: allow | deny | allow_with_exception
- exception_ref（Time-boxed overrides連動可）
- produced_at :contentReference[oaicite:20]{index=20}

### 5.4 HumanErrorGuardrailPolicy（L-52）
- guardrail_id
- high_risk_actions[]
- friction_level_by_action
- contextual_warnings
- hard_block_conditions
- updated_at :contentReference[oaicite:21]{index=21}

### 5.5 SecretsPrivacyAuditReport（L-53）
- spa_id
- scope（audit logs / exports / traces）
- findings: suspected_secret_leak / pii_leak（将来必要なら）/ masking_failure
- recommended_actions（rotate/stop/incident-enable）
- executed_actions_ref
- produced_at :contentReference[oaicite:22]{index=22}

### 5.6 ChaosGameDayPlan（L-54）
- chaos_id
- scenario_type: cp_down | agent_partition | lease_fail | exchange_reject_storm | data_gap | partial_deploy
- schedule_ref
- success_criteria
- runbook_ref
- last_run_at, last_result_ref :contentReference[oaicite:23]{index=23}

### 5.7 SelfHealingPolicy（L-55）
- shp_id
- auto_actions_ordered[]
- max_attempts / cooldown
- requires_human_approval_above_level
- escalation_to_incident: true
- audit_required: true :contentReference[oaicite:24]{index=24}

### 5.8 BotTrustLevel（L-56）
- trust_id
- levels: experimental / probation / trusted / critical
- per_level_limits: max_exposure / max_symbols / max_order_rate / rollout_max_stage / allowed_actions / monitoring_requirements
- promotion_rules
- demotion_rules :contentReference[oaicite:25]{index=25}

---

## 6. Canonical Interfaces（API）
> v1.3のAPIに追加（v1.4分）。:contentReference[oaicite:26]{index=26}

- POST /formal_verification/targets/set
- POST /release/cert/issue
- POST /release/cert/revoke
- POST /supply_chain/gate/run
- POST /guardrails/policy/set
- POST /secrets_privacy/audit/run
- POST /chaos/plans/create
- POST /chaos/plans/{id}/run
- POST /self_healing/policy/set
- POST /trust_levels/set
- POST /trust_levels/promote|demote

TODO:
- 各APIのRequest/Responseスキーマ
- 認証/認可（RBAC）と監査イベントの関連付け仕様
- idempotency（request_id/fingerprint/dedupe）のAPIレベル要件の具体

---

## 7. Canonical Behaviors（実行フロー）
> v1.3までを正本とし、v1.4追加分のフローを記載。:contentReference[oaicite:27]{index=27}

### 7.1 Formal Verification（L-49）
- 対象性質（state/lease/dedupe/bulk/stop）を固定し、証明参照を保持
- Gate利用: 未検証/失敗 → release禁止（運用ポリシで選択）:contentReference[oaicite:28]{index=28}

### 7.2 Release Safety Certificate（L-50）
- release前またはrelease時に証明書発行
- 「検証済み参照」を束ね、manifest hash + 署名参照を付与
- 重大インシデントで revoke 可能（監査必須）:contentReference[oaicite:29]{index=29}

### 7.3 Supply-chain Gates（L-51）
- digest固定 + 署名検証
- SBOM生成/参照、脆弱性サマリ評価
- deny時: 理由コード + triage
- 例外: Time-boxed override（期限必須）:contentReference[oaicite:30]{index=30}

### 7.4 Human Error Guardrails（L-52）
- 高危険操作に friction（staging必須、二者承認必須、dry-run必須等）
- 操作前に影響プレビュー（対象/資金枠/依存/停止証明見込み）
- hard block 条件で拒否（理由コード）:contentReference[oaicite:31]{index=31}

### 7.5 Secrets/Privacy Audit（L-53）
- 監査ログ/エクスポート/トレースに秘密混入をスキャン
- 高確度検知時: incident enable / rotate / stop / export revoke 等（ポリシにより）
- 全て監査イベント化 :contentReference[oaicite:32]{index=32}

### 7.6 Chaos/GameDays（L-54）
- 定期的に破壊シナリオを実施し、復旧/停止証明/監査証跡を継続的に証明
- 失敗は Runbook 改善と Self-healing 境界見直しに反映 :contentReference[oaicite:33]{index=33}

### 7.7 Self-healing Policy（L-55）
- 自動復旧は段階的（上限/回数/冷却/承認境界を固定）
- “どこまで自動でやるか” をSSOTで固定し誤爆抑制 :contentReference[oaicite:34]{index=34}

### 7.8 Trust Levels（L-56）
- 新規Botは experimental/probation から開始
- 実績・証明書・事故ゼロ期間で昇格
- 逸脱/違反/事故で降格（自動も可能）
- 信頼度で rollout/limits/強操作許可/監視義務が変化 :contentReference[oaicite:35]{index=35}

---

## 8. Reason Codes（理由コード）
- L.FORMAL_VERIFICATION.MISSING_OR_FAILED
- L.RELEASE_CERT.ISSUED
- L.RELEASE_CERT.REVOKED
- L.SUPPLY_CHAIN.DENIED
- L.GUARDRAIL.BLOCKED
- L.SECRETS_PRIVACY.LEAK_SUSPECTED
- L.CHAOS.RUN_FAILED
- L.SELF_HEALING.ESCALATED
- L.TRUST_LEVEL.PROMOTED
- L.TRUST_LEVEL.DEMOTED :contentReference[oaicite:36]{index=36}

---

## 9. Observability（指標）
- formal_verification_fail_rate
- release_cert_issued_count / revoked_count
- supply_chain_denied_count
- guardrail_block_count
- secrets_privacy_leak_suspected_count
- chaos_run_success_rate / mttr_from_chaos
- self_healing_auto_action_count / escalation_count
- trust_level_distribution (gauge) :contentReference[oaicite:37]{index=37}

TODO:
- 各メトリクスのラベル設計（bot_id, version, tenant等）
- SLO/アラート閾値（Gold signalsとの紐づけ）

---

## 10. Acceptance / DoD（完成条件）
- L-01〜L-48（v1.3）に加えて、以下が成立していること：:contentReference[oaicite:38]{index=38}
  - L-49: 検証対象固定 + spec_ref管理（gate可能）
  - L-50: 証明書の発行/失効 + 署名/ハッシュで改竄耐性
  - L-51: Supply-chain gate（署名/SBOM/脆弱性）がリリースに統合
  - L-52: 危険操作ガードレールが機能し誤操作が構造的に減る
  - L-53: 秘密/プライバシ監査が継続実行でき、検知時措置が定義済み
  - L-54: Chaos/GameDays が計画→実行→評価→改善で回る
  - L-55: Self-healing の上限と承認境界が固定、誤爆せず復旧
  - L-56: Trust Levels により段階的にしか本番化できない

---

## 11. Capability Index（ID保持・索引）
### 11.1 サブドメインID
- L-01〜L-48（v1.3 正本）※本文未入力 → TODO
- L-49 Formal Verification Targets
- L-50 Release Safety Certificate
- L-51 Supply-chain Security Gates
- L-52 Human Error Guardrails
- L-53 Secrets/Privacy Audit
- L-54 Chaos/GameDays Program
- L-55 Self-Healing Policy
- L-56 Bot Trust Levels :contentReference[oaicite:39]{index=39}

### 11.2 エンティティ（v1.4追加）
- FormalVerificationTarget（L-49）
- ReleaseSafetyCertificate（L-50）
- SupplyChainGateReport（L-51）
- HumanErrorGuardrailPolicy（L-52）
- SecretsPrivacyAuditReport（L-53）
- ChaosGameDayPlan（L-54）
- SelfHealingPolicy（L-55）
- BotTrustLevel（L-56）:contentReference[oaicite:40]{index=40}

### 11.3 APIエンドポイント（v1.4追加）
- POST /formal_verification/targets/set
- POST /release/cert/issue
- POST /release/cert/revoke
- POST /supply_chain/gate/run
- POST /guardrails/policy/set
- POST /secrets_privacy/audit/run
- POST /chaos/plans/create
- POST /chaos/plans/{id}/run
- POST /self_healing/policy/set
- POST /trust_levels/set
- POST /trust_levels/promote|demote :contentReference[oaicite:41]{index=41}

### 11.4 理由コード
- L.FORMAL_VERIFICATION.MISSING_OR_FAILED
- L.RELEASE_CERT.ISSUED
- L.RELEASE_CERT.REVOKED
- L.SUPPLY_CHAIN.DENIED
- L.GUARDRAIL.BLOCKED
- L.SECRETS_PRIVACY.LEAK_SUSPECTED
- L.CHAOS.RUN_FAILED
- L.SELF_HEALING.ESCALATED
- L.TRUST_LEVEL.PROMOTED
- L.TRUST_LEVEL.DEMOTED :contentReference[oaicite:42]{index=42}

### 11.5 メトリクス
- formal_verification_fail_rate
- release_cert_issued_count / revoked_count
- supply_chain_denied_count
- guardrail_block_count
- secrets_privacy_leak_suspected_count
- chaos_run_success_rate / mttr_from_chaos
- self_healing_auto_action_count / escalation_count
- trust_level_distribution (gauge) :contentReference[oaicite:43]{index=43}

---

## 12. TODO（不足の明示）
- TODO: v1.3 正本（L-01〜L-48）の本文取り込み（本入力に含まれないため）。
- TODO: APIの詳細契約（スキーマ、エラー、認可、監査イベント、冪等の具体）。
- TODO: テスト/検証項目（Behavior/Testsとしての具体ケース、回帰、カオス計測の判定式）。※本入力はDoD/メトリクスはあるが、テストケースの明文化は未提示。:contentReference[oaicite:44]{index=44}
