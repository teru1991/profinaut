# Error Catalog SSOT v1.0（固定）
Document ID: SYS-ERROR-CATALOG-SSOT
Status: Canonical / Fixed Contract
Scope: 標準エラーモデル（A）に基づく “説明可能な一意ID” の体系を固定する（Yの中核）

## 0. 目的（Non-negotiable）
エラーが “一意IDで説明できない” と、運用・サポート・再現が破綻する。
本書は以下を固定する：
- error_code の命名規約（唯一の正）
- error_code ↔ 標準エラーモデル（kind）↔ retryable ↔ safety impact ↔ runbook の導線
- secret-free（秘密を含めない）を強制する
- UNKNOWN を常態化させない（UNKNOWN増加は設計欠陥として扱う）

---

## 1. 参照（正本）
- Standard Error Model（固定）: `docs/specs/platform_foundation_spec.md`（Section: Standard Error Model）
- Terminology（固定）: `docs/specs/system/terminology.md`
- Safety（固定）: `docs/specs/crosscut/safety_interlock_spec.md`
- Audit/Replay（固定）: `docs/specs/crosscut/audit_replay_spec.md`
- Support Bundle（固定）: `docs/specs/crosscut/support_bundle_spec.md`
- Runbooks Index（可変）: `docs/runbooks/README.md`

---

## 2. エラーコード形式（固定：唯一の正）
### 2.1 形式
ERR-<DOMAIN>-<SUBSYSTEM>-<KIND>-<NNNN>

- DOMAIN: A〜Y のドメイン文字（domains_ssotに一致）
- SUBSYSTEM: 1〜16文字（英大文字/数字/アンダースコア）。例：COLLECTOR, OMS, UI, STORAGE
- KIND: 標準エラーモデルの分類（固定語彙）
  - TRANSIENT / RATELIMITED / AUTH / INVALIDREQUEST / PROTOCOL / INTEGRITY / RESOURCEEXHAUSTED / SAFETY / UNKNOWN
- NNNN: 0001〜（同一DOMAIN+SUBSYSTEM+KIND内で単調増加）

例：
- ERR-H-COLLECTOR-RATELIMITED-0001
- ERR-I-OMS-AUTH-0003
- ERR-K-LEDGER-INTEGRITY-0007
- ERR-E-SAFETY-SAFETY-0001

### 2.2 禁止
- 秘密値・鍵・トークン・署名・生payloadを code に含めない
- ドメイン外の用語を勝手に定義しない（terminology優先）

---

## 3. エラーエントリの必須フィールド（固定）
各 error_code は、最低限以下を持つ（SSOT上は “表” として管理する）：

- code
- title（短い人間向け要約）
- kind（固定語彙）
- severity（INFO/WARN/ERROR/CRITICAL）
- retryable_default（true/false）
- safety_impact_default（NONE / SAFE / CLOSE_ONLY / FLATTEN / HALT）
- operator_action（NONE / RUNBOOK / BREAK_GLASS）
- runbook_ref（docs/runbooks/**.md または該当セクション）
- evidence（audit_event / gate_results / integrity_report / replay_pointers / support_bundle）
- redaction（REQUIRED / OPTIONAL）
- notes（補足：再現条件、典型原因）

---

## 4. 運用導線（固定）
### 4.1 発生時に必ず残すもの（secret-free）
- logs: trace_id, run_id, code, kind, retryable, component, subsystem
- audit_event: “監査対象” なら audit_event へ（秘密なし）
- support_bundle: 重大（ERROR以上）なら bundle で追跡できること

### 4.2 Runbook 連動（必須）
- operator_action=RUNBOOK の場合、runbook が存在しない状態で main に入れてはならない（Gate FAIL 推奨）
- runbook は docs/runbooks/README.md から辿れること（索引必須）

### 4.3 Safety 連動（必須）
- safety_impact_default != NONE の場合、Safety interlock の “どの条件で遷移するか” を Core Spec に書く
- 実装は Safety spec に従い、勝手に意味を増やさない（上書き禁止）

---

## 5. レジストリ（初期セット：空で良いが “枠” は固定）
### 5.1 登録表（追加していくSSOT）
| code | title | kind | severity | retryable_default | safety_impact_default | operator_action | runbook_ref | evidence | redaction | notes |
|---|---|---|---|---:|---|---|---|---|---|---|
| (例) ERR-H-COLLECTOR-RATELIMITED-0001 | WS 429 / rate limit | RATELIMITED | WARN | true | NONE | RUNBOOK | docs/runbooks/<...>.md | gate_results, support_bundle | REQUIRED | backoff/jitter必須 |
| (例) ERR-I-OMS-AUTH-0001 | API auth failure | AUTH | ERROR | false | HALT | RUNBOOK | docs/runbooks/<...>.md | audit_event, support_bundle | REQUIRED | key失効/権限不足 |

---

## 6. DoD（エラー体系が機能している条件）
1) 重大エラーが code で一意に特定できる
2) code から runbook と evidence が辿れる
3) secret-free（赤塗り）違反が Gate で落ちる
4) UNKNOWN が増えたら “設計欠陥” として扱われる導線がある

---
End of document
