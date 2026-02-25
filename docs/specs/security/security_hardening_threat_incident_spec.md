# Security Hardening / Threat Model / Incident Evidence Core Spec v1.0（固定仕様）
Defense-in-depth / Threat model / Detection & containment / Evidence preservation

- Document ID: SEC-HARDEN-THREAT-IR-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): S（Security Hardening / Threat / Incident）
- Depends-on（Fixed）:
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
  - Governance/Release safety: `docs/specs/governance/governance_change_release_safety_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/support_bundle_manifest.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
- Policy separation（固定しない）:
  - 具体の許容回数/閾値/ブロック時間/通知先/鍵ローテ頻度 → `docs/policy/**`
  - 手順（連絡/封じ込め/復旧/報告/提出）→ `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、取引・秘密・証拠・運用を守るために **Defense-in-depth（多層防御）** を固定し、
攻撃や事故が起きても **被害を最小化し、証拠を保全し、再現可能**にする。

必達要件（固定）：
1) **Threat model is explicit**：守る対象と脅威が明示される
2) **Defense-in-depth**：単一防御に依存しない（漏れた前提で次層が止める）
3) **Fail safe**：不明/検知不能は安全側（SAFE/EMERGENCY_STOPへ寄る）
4) **No secret leakage**：ログ/監査/バンドル/エクスポートに秘密を入れない
5) **Containment first**：疑義が出たら封じ込め（killswitch/BLOCK、鍵失効、アクセス遮断）
6) **Evidence preservation**：改ざん耐性ある形で証拠を保全（audit/replay/bundle）
7) **Reproducibility**：あとから「何が起きたか」を説明できる（タイムライン・参照）
8) **Least privilege**：権限は最小、特権は監査・期限付き

---

## 1. 保護対象（Assets）（固定）
最低限、以下を “保護対象” として扱う：

- Secrets（API keys, tokens, signing keys, DB creds）
- Funds / Positions / Orders（資産・注文・ポジション）
- Evidence（audit_event, integrity_report, replay_pointers, support bundles）
- Control plane authority（操作権）
- Data integrity（raw/canonical/ledger）
- Availability（継続稼働）
- Privacy（個人情報/機微情報：含む場合）

固定ルール：
- Evidence は「守る対象」かつ「提出物」であり、改ざん/消去が致命的

---

## 2. 脅威モデル（Threats）（固定枠組み）
### 2.1 Threat categories（固定）
- Credential theft（鍵漏洩）
- Privilege escalation（権限昇格）
- Remote execution / supply chain（RCE/依存汚染）
- Data tampering（データ改ざん/削除）
- Replay / spoofing（なりすまし/再送）
- DoS / resource exhaustion（過負荷・枯渇）
- Misconfiguration / operator error（誤操作・誤設定）
- Insider risk（内部不正）
- Observability blind spot（監視欠損）

### 2.2 Security assumptions（固定）
- 外部は信用しない（RPC/Exchange/Providerは嘘をつく可能性）
- 内部も侵害されうる（ゼロトラスト寄り）
- ログ/監査/証拠は攻撃者に狙われる

---

## 3. Defense-in-depth（固定：防御層）
最低限、以下の層を持つ（実装は自由だが意味は固定）：

1) **Identity & Access**：最小権限、短命セッション、break-glass管理
2) **Secret handling**：secret_refのみ、forbidden-key scan、ローテ/失効
3) **Network boundary**：管理面の最小公開、内部通信の認証
4) **Runtime safety**：Safety Mode + Kill-Switch + Pre-trade gate
5) **Change governance**：gate、段階導入、ロールバック、監査
6) **Observability honesty**：監視欠損を UNKNOWN として安全へ連動
7) **Evidence chain**：audit/replay/bundle の保全
8) **Data integrity**：append-only、hash/manifest、restatementモデル

---

## 4. Detection（検知）（固定）
### 4.1 必須検知シグナル（固定カテゴリ）
- Auth failures spike（署名失敗/認証失敗）
- Forbidden-key scan triggers（秘密混入検知）
- Unexpected config changes（SSOT hash/plan hash drift）
- Gate FAIL/UNKNOWN（監視欠損含む）
- Integrity FAIL/UNKNOWN（欠損/改ざん疑い）
- Unusual execution patterns（注文急増、異常レート）
- Resource exhaustion（disk near-full, IO stall）
- Suspicious access（権限拒否連発、未知のactor）

### 4.2 検知不能＝安全ではない（固定）
監視が欠損した場合：
- “健康” とせず UNKNOWN
- crosscut safety により SAFE 側へ寄る（最低限）

---

## 5. Containment（封じ込め）（固定）
疑義が発生したら、以下の封じ込め手段を即時適用できること：

- Execution Kill-Switch を `BLOCK` へ（最優先）
- Safety Mode を `SAFE` / `EMERGENCY_STOP` へ（状況により）
- セッション失効（actor/session revocation）
- Secret revoke/rotate（secret_ref単位）
- 外部公開の遮断（tunnel/ingress stop）
- 対象ストリーム/サービスの quarantine

固定ルール：
- 封じ込めは “強める方向” は常に可能
- “緩める方向” は dangerous op（challenge/confirm + audit）

---

## 6. Incident Evidence（証拠保全）（固定）
### 6.1 必須証拠（固定）
インシデント時に最低限保持すべき証拠：
- audit_event（タイムライン）
- safety_state transitions
- gate_results / integrity_report（該当window）
- replay_pointers（入力範囲参照）
- support_bundle_manifest（生成した場合）
- relevant logs/metrics snapshots（redacted）

固定ルール：
- 証拠は secret-free
- 証拠の削除/改ざんは dangerous op 扱い（原則禁止）

### 6.2 Evidence chain（固定）
- すべての証拠は相互参照できる（refs）
- “いつ/誰が/何を” を監査で辿れる

---

## 7. Incident Classification（固定枠組み）
- `SEV0`：資産喪失/秘密漏洩/監査破綻の疑い（EMERGENCY_STOP許容）
- `SEV1`：重大な整合性破綻/広範囲停止
- `SEV2`：部分劣化/限定範囲
- `SEV3`：軽微/情報

固定ルール：
- 分類は運用値で変えても良いが “SEV0は最優先封じ込め” の意味は固定

---

## 8. Audit（固定）
最低限、以下を audit_event に残す（秘密値なし）：
- `security.alert.raised`（symptom + evidence refs）
- `security.containment.applied`（killswitch/safety/rotate/revoke）
- `security.secret.guard.triggered`（forbidden-key scan）
- `security.access.denied`（重要拒否）
- `security.incident.opened/closed`（sev + timeline refs）
- `support_bundle.created`（manifest ref）
- `integrity.record` / `gate.record`

---

## 9. Recovery（復旧）（固定）
復旧は “封じ込め解除” を含むため危険操作になり得る。
固定要件：
- 復旧前に evidence が確保されていること
- 復旧は段階的（BLOCK→CLOSE_ONLY→ALLOW 等）にできる
- 復旧解除は dangerous op（challenge/confirm + audit）

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) forbidden-key scan が働き、秘密がログ/監査/バンドルに混入しない
2) 認証失敗急増で封じ込め（BLOCK/SAFE）へ遷移できる
3) 監視欠損が UNKNOWN として表面化し SAFE 側へ寄る
4) インシデント時に必要証拠（audit/gate/integrity/replay/bundle）が揃う
5) 封じ込め強化は即時可能、緩和は challenge/confirm 必須
6) ロールバック/復旧の証跡が audit_event に残る

---

## 11. Policy/Runbookへ逃がす点
- 閾値、ブロック時間、通知先、ローテ頻度、当番
- 具体手順（連絡、封じ込め、復旧、報告、提出）
→ Policy/Runbookへ（意味は変えない）

---
End of document
