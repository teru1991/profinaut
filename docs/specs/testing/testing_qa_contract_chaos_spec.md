# Testing Strategy / QA / Contract & Chaos Core Spec v1.0（固定仕様）
Contract tests / Replay tests / Property & fuzz / Chaos & resilience / Evidence-gated QA

- Document ID: TEST-QA-CONTRACT-CHAOS-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): U（Testing / QA）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Governance/Release safety: `docs/specs/governance/governance_change_release_safety_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Collector: `docs/specs/market_data/collector_framework_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
  - Security hardening: `docs/specs/security/security_hardening_threat_incident_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/*.schema.json`（特に audit_event / safety_state / startup_report / gate_results / integrity_report / replay_pointers / support_bundle_manifest）
- Policy separation（固定しない）:
  - カバレッジ閾値、テスト実行頻度、時間制限、データ保持 → `docs/policy/**`
  - 実行手順（CI/ローカル/本番前チェック）→ `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、システムが **正しい契約を守り、壊れ方が安全で、再現可能**であることをテストで証明する枠組みを固定する。

必達要件（固定）：
1) **Contracts are enforced**：契約（JSON Schema）を破る変更は通さない
2) **Replayable QA**：重大障害は再現データ（replay pointers）で回帰テスト化できる
3) **Chaos resilience**：外部障害（429/切断/遅延/順序乱れ/部分停止）に耐えることを証明する
4) **Security invariants**：秘密漏洩ゼロ・危険操作制御・監査完全性を破らない
5) **Evidence-gated**：テスト結果は gate_results として証拠化され、リリース判断に使われる
6) **Fail-safe**：テストが “検知不能” なら合格にしない（UNKNOWNを許容しない）

---

## 1. 範囲（in / out）
### 1.1 In
- Contract tests（schema/SSOT）
- Integration tests（collector/storage/execution/control plane）
- Replay tests（実データ再現）
- Property/fuzz tests（境界条件）
- Chaos tests（障害注入）
- Security tests（forbidden-key scan 等）
- Performance regression（最低限の予算逸脱検知）
- Test evidence（gate_results / audit records）

### 1.2 Out
- 特定のCI製品設定の細部（runbooks）
- 具体の閾値（policy）

---

## 2. Test Taxonomy（固定：テスト体系）
最低限、以下のカテゴリを持つ：

1) **Contract tests**
2) **Schema drift / SSOT tests**
3) **Unit tests**
4) **Integration tests**
5) **Replay tests**
6) **Property-based / fuzz tests**
7) **Chaos tests**
8) **Security tests**
9) **Performance regression tests**
10) **Docs validity tests**（参照切れ、SSOT増殖）

固定ルール：
- 重要変更は “複数カテゴリ” を跨いで検証される
- 合格の根拠（evidence）は保存される

---

## 3. Contract Tests（固定）
### 3.1 Schema validation（固定）
- `docs/contracts/*.schema.json` は常に整合し、破損しない
- 生成物（audit_event/gate_results/integrity_report/replay_pointers/support_bundle_manifest/startup_report/safety_state）は schema に適合することを検証する

### 3.2 Backward compatibility（固定枠組み）
- 破壊的変更は新schema_versionへ（versioning_policy準拠）
- 既存データが読めること（最低限の読み取り互換）をテストで担保できる

---

## 4. SSOT / Docs Tests（固定）
- docs/specs（固定仕様）と docs/contracts（契約）が矛盾しない
- 正本が一本化されている（legacy/stub で参照切れが起きない）
- forbidden-key（秘密直書き）検知が docs にも適用される（少なくとも settings/descriptor/policy 等）

---

## 5. Integration Tests（固定）
### 5.1 Collector（固定観点）
- reconnect/resubscribe が idempotent
- snapshot+delta が破綻したら再同期
- silent drop しない（欠損が integrity に出る）
- quarantine が機能する

### 5.2 Storage（固定観点）
- raw append-only（破壊操作はdangerous op）
- backlog/backpressure が観測できる
- integrity_report が生成される

### 5.3 Execution（固定観点）
- SAFE/EMERGENCY_STOP で実行が止まる
- killswitch（CLOSE_ONLY/FLATTEN/BLOCK）が強制される
- intent_id の idempotency（重複発注しない）

### 5.4 Control Plane（固定観点）
- dangerous op が challenge/confirm 必須
- command_id が idempotent
- auditが欠けたら実行できない

---

## 6. Replay Tests（固定）
### 6.1 Replayable dataset（固定）
- replay_pointers により、入力範囲（raw/ledger/window）が固定参照できる
- replayにより少なくとも “evidence replay（Type B）” が成立する

### 6.2 Regression（固定）
- 重大バグ/障害は replay dataset を追加し、再発防止テストに組み込む
- replayの結果差分が出る場合、差分説明が残る（audit）

---

## 7. Property-based / Fuzz Tests（固定）
最低限、以下の性質を検証する：
- dedupe の冪等性（同じイベントを何度流しても計上は1回）
- 順序乱れ耐性（順番をシャッフルしても整合性が保たれる or 検出される）
- 無効入力耐性（InvalidRequestは安全に拒否）
- 禁止キー混入（secret漏洩ゼロ）

---

## 8. Chaos Tests（固定）
### 8.1 Chaos scenarios（固定カテゴリ）
- WS切断/遅延/順序乱れ/重複
- 429/rate-limit storm
- 一部DB停止/遅延（storage backpressure）
- observability停止（metrics/log ingest down）
- clock skew（時刻異常）
- disk near-full / IO stall

### 8.2 Expected behavior（固定）
- silent drop禁止（欠損は integrity に出る）
- UNKNOWN/FAIL は SAFE 側へ（crosscut）
- quarantine により局所隔離
- 重要経路（execution）は BLOCK/CLOSE_ONLY に縮退

---

## 9. Security Tests（固定）
最低限、以下を検証する：
- forbidden-key scan が働き、秘密がログ/監査/バンドルに入らない
- dangerous op が challenge/confirm 無しに通らない
- 権限不足で操作できない（access denied が監査に残る）
- export/report が secret-free

---

## 10. Performance Regression（固定）
- 重要budget（ingest/execution/persist）の遅延が悪化したら検知できる
- backpressure が働き、破綻しない（silent drop無し）
- 観測不能（UNKNOWN）なら “合格” にしない

---

## 11. Test Evidence / Gate Results（固定）
テスト結果は `gate_results` に集約できる（CI_GATE）。
固定要求：
- PASS/WARN/FAIL/UNKNOWN のいずれか
- window / build hash / ssot hash / policy snapshot ref を参照
- audit_event から gate_results_ref に辿れる

---

## 12. テスト失敗時の証拠（固定）
重大失敗（P0/P1）時は support bundle を生成できる（policyでトリガ設定）。
- support_bundle_manifest_ref を audit_event に残す
- bundle は secret-free

---

## 13. テストのDoD（固定）
最低限、以下を満たさない変更はリリース不可：
1) contract tests がPASS
2) 主要integration tests がPASS
3) replay regression がPASS（存在する場合）
4) security invariants がPASS
5) observability欠損時に UNKNOWN を合格扱いしない
6) gate_results に証拠が残る

---

## 14. Policy/Runbookへ逃がす点
- カバレッジ閾値、実行頻度、時間制限、データ保持
- CI/ローカル実行手順
→ Policy/Runbookへ（意味は変えない）

---
End of document
