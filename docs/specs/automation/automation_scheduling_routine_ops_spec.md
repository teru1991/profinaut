# Automation / Scheduling / Routine Ops Core Spec v1.0（固定仕様）
Scheduled jobs / Routine operations / Auto-recovery / Safe automation boundaries

- Document ID: AUTO-SCHED-OPS-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): W（Automation / Ops）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Governance/Release safety: `docs/specs/governance/governance_change_release_safety_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Data Catalog/Lineage: `docs/specs/data_governance/data_catalog_lineage_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/support_bundle_manifest.schema.json`
- Policy separation（固定しない）:
  - 実行頻度、時間窓、リトライ回数、閾値、通知先、保持期間 → `docs/policy/**`
  - 手順（停止/復旧/障害対応/承認フロー）→ `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
定期処理・自動復旧は、人手を減らしつつ安全性を上げるための仕組みだが、誤作動すれば被害が拡大する。
本仕様は、Automation を **安全境界つき**で実行し、証拠と監査を残し、異常時は縮退/停止できる不変条件を固定する。

必達要件（固定）：
1) **Automation is explicit**：何がいつ動くかが明示される（隠れジョブ禁止）
2) **Least privilege**：ジョブは最小権限で実行される（特権の常用禁止）
3) **Dangerous automation is gated**：破壊的/緩和系は dangerous op（challenge/confirm）
4) **Idempotent jobs**：二重実行しても二重効果を起こさない
5) **Evidence-first**：ジョブ実行は audit_event / replay_pointers / integrity_report で証拠化される
6) **Fail-safe**：監視欠損/整合性不明は “成功扱い” せず、安全側へ（SAFE）
7) **Auto-recovery is bounded**：自動復旧は回数/範囲が制限され、暴走しない
8) **No secrets**：ログ/監査/バンドルに秘密を入れない（secret_refのみ）

---

## 1. 範囲（in / out）
### 1.1 In
- ジョブ定義（スケジュール、入力、出力、権限、上限）
- ルーチン運用（整合チェック、コンパクション、バックアップ検証等）
- 自動復旧（再接続、再同期、再インデックス等）の枠組み
- ジョブ安全境界（dangerous op、kill-switch、safety coupling）
- 実行証拠（audit/replay/integrity）
- ジョブの停止/縮退/隔離（quarantine）

### 1.2 Out
- 特定スケジューラ製品（cron、k8s、GH actions等）の設定細部
- 具体の頻度/閾値（policy）
- 具体の手順（runbooks）

---

## 2. Job Identity（固定）
すべてのジョブは一意に識別できる：
- `job_id`（安定）
- `job_version`（SemVer）
- `job_class`（MAINTENANCE / VALIDATION / RECOVERY / REPORTING / BACKUP 等）
- `owner`（team/service）
- `environment`（dev/stage/prod）
- `mode`（paper/shadow/live）
- `schedule_ref`（cron/interval: 実装自由）
- `capabilities`（必要権限/必要入力）

固定ルール：
- job_id の再利用は禁止
- バージョンが上がれば audit で追跡できる

---

## 3. Job Inputs/Outputs（固定）
### 3.1 Inputs pinning（固定）
ジョブが参照する入力は曖昧参照を禁止し、最低限以下を満たす：
- window（start/end UTC）
- dataset refs（data catalogのdataset_id）
- policy snapshot ref（適用ルール）
- binary hash/config hash（可能な範囲）

### 3.2 Outputs evidence（固定）
ジョブの成果物は少なくとも以下を持つ：
- output refs（データセット/レポート/ログ）
- replay_pointers_ref（入力範囲）
- integrity_report_ref（該当する場合）
- quality（OK/DEGRADED/UNKNOWN）

---

## 4. Idempotency（固定）
- 同じ `job_run_id`（idempotency key）で二重効果を起こさない
- 再試行は “副作用が二重” にならない設計（append-only/marker等）
- 二重実行が起きた場合は audit_event に `duplicate_suppressed` を残せる

---

## 5. Safety coupling（固定）
- safety_mode SAFE/EMERGENCY_STOP のとき、危険ジョブ（破壊的/緩和系）は実行しない
- observability UNKNOWN（監視欠損）は成功扱いしない
- integrity FAIL/UNKNOWN のときは、ジョブは “修復” 以外を停止/縮退できる

固定ルール：
- 自動化で gate を無効化しない（無効化は dangerous op）

---

## 6. Dangerous Ops in Automation（固定）
以下は少なくとも dangerous op として扱えることが固定要求：
- retention/compaction/削除系
- quarantine解除
- kill-switch緩和
- gate無効化/緩和
- secrets rotate/revoke 実行
- DB restore/repair 等の破壊的操作

固定ルール：
- dangerous op は challenge/confirm + audit が必須
- 自動ジョブが勝手に confirm を実行できない（人間介在 or 事前承認トークン等：実装自由だが意味固定）

---

## 7. Auto-recovery（固定）
### 7.1 Bounded recovery（固定）
自動復旧は “範囲と回数” が制限される（値はPolicy）。
- retry storm を起こさない（backoff + jitter）
- 連続失敗時は quarantine へ（局所隔離）

### 7.2 Recovery actions（固定枠組み）
- reconnect/resubscribe
- resync/reindex
- backlog drain（安全な範囲）
- failover（primary→secondary）

固定ルール：
- フェイルオーバー/復旧は静かに起きず、audit/integrityに残る

---

## 8. Quarantine（固定）
ジョブ自体も隔離できる：
- 異常な頻度で失敗
- 異常なリソース消費
- 想定外の作用（予期せぬデータ削除試行など）

固定ルール：
- quarantine解除は dangerous op になり得る
- quarantine入退は監査対象

---

## 9. Evidence & Lineage（固定）
### 9.1 Catalog/Lineage（固定）
ジョブは生成物に lineage を付与できる：
- input_dataset_ids → output_dataset_id
- transform_id（job_id）
- transform_version（job_version）
- window
- policy snapshot ref
- replay pointers ref

### 9.2 Evidence chain（固定）
ジョブ実行は以下に辿れる：
- audit_event（start/end/fail）
- replay_pointers（入力）
- integrity_report（品質）
- support bundle（必要時）

---

## 10. Audit（固定）
最低限、以下を audit_event に残す（秘密値なし）：
- `job.run.start`（job_id/version + window + inputs refs）
- `job.run.end`（status + outputs refs + quality）
- `job.run.fail`（error.kind + evidence refs）
- `job.run.retry`（count + reason）
- `job.run.quarantine.enter/exit`
- `job.dangerous_op.challenge/confirm/reject`（該当時）
- `job.failover`（primary→secondary）
- `integrity.record` / `gate.record`
- `support_bundle.created`（必要時）

---

## 11. Failure modes（固定）
- inputs pinning 失敗 → 実行拒否（UNKNOWNを成功扱いしない）
- observability missing → UNKNOWN → 安全側へ
- 連続失敗 → quarantine
- 破壊的操作が SAFE/EMERGENCY_STOP 下で実行されそう → 拒否 + audit

---

## 12. テスト/検証観点（DoD）
最低限これが検証できること：

1) job_id が一意で、実行が監査される
2) 二重実行でも二重効果が起きない（idempotent）
3) inputs が window/dataset refs で pinning される（曖昧参照禁止）
4) dangerous job が challenge/confirm 無しに実行できない
5) 連続失敗で quarantine に入る（解除は危険操作扱い）
6) 実行が audit/replay/integrity で証拠化される
7) 監視欠損が UNKNOWN として扱われ成功扱いされない

---

## 13. Policy/Runbookへ逃がす点
- 実行頻度、時間窓、リトライ回数、閾値、通知先、保持期間
- 停止/復旧/承認フロー/障害対応
→ Policy/Runbookへ（意味は変えない）

---
End of document
