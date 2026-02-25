# Tax / Compliance / Legal Reporting Core Spec v1.0（固定仕様）
Reproducible tax lots / Compliance evidence / Export safety / Restatement-aware reporting

- Document ID: TAX-COMPLIANCE-LEGAL-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): Q（Tax / Compliance / Legal）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Accounting/PnL Ledger: `docs/specs/accounting/order_trade_ledger_pnl_spec.md`
  - Reporting/Dashboard Truth: `docs/specs/reporting/reporting_dashboard_explainability_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/support_bundle_manifest.schema.json`
- Policy separation（固定しない）:
  - 国/税制ルール、課税区分マッピング、丸め、採用方法（FIFO/平均等）、提出フォーマット → `docs/policy/**`
  - 提出/監査対応/問い合わせ手順 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
税務・コンプライアンスは「後から説明できること」が価値であり、再現不能は致命傷。
本仕様は、台帳（Ledger）を正本として **再現可能な税務計算**と **監査可能なコンプライアンス証拠**を生成し、
エクスポートを安全に行える “不変条件” を固定する。

必達要件（固定）：
1) **Ledger is the truth**：税計算・報告は台帳入力を唯一の正本とする
2) **Deterministic-by-input**：同じ台帳入力＋同じ税ルール版で同じ結果、または差分説明が残る
3) **Restatement-aware**：訂正（restate）を上書きでなく追加イベントとして扱い、報告も追随できる
4) **Evidence-linked**：すべての報告は replay_pointers で入力範囲とルール版へ辿れる
5) **No silent loss**：欠損/不明/監視欠損は integrity_report に必ず出る
6) **Access controlled**：税務・個人情報の閲覧/出力は最小権限、監査ログ必須
7) **Export safe**：秘密・PII漏洩を防ぐ（redaction/forbidden-key scan）
8) **Fail-safe**：不明区間（UNKNOWN）を“確定値”として提出しない（品質注記）

---

## 1. 範囲（in / out）
### 1.1 In
- 台帳（orders/fills/balance/fees/funding/restatement）に基づく税務計算の枠組み
- Tax lot（取得単位）の生成と消込（売却）モデル
- 国/税制ルールを “Policy版” として適用する仕組み（意味は固定）
- コンプライアンス証拠（取引履歴、残高変動、根拠参照）
- 申告/提出用レポートの生成（CSV/JSON/PDF等：形式はPolicy）
- 監査・再現（audit/replay pointers）
- 出力のアクセス制御とエクスポート安全

### 1.2 Out
- 国別の細かな条文解釈（Policy/Runbookで更新）
- UI設計の詳細
- 法的助言（この仕様はシステムの再現性・証拠性を固定するのみ）

---

## 2. Core Concepts（固定）
### 2.1 Tax jurisdiction / regime（固定概念）
- `jurisdiction`（国・地域）
- `regime_id`（税制セットの識別子）
- `regime_version`（SemVer：Policyで管理）
固定ルール：
- 計算結果は必ず (regime_id, regime_version, policy_snapshot_ref) を保持し、後から再現可能にする

### 2.2 Tax lot（固定：取得単位）
Tax lot は “取得した資産の塊” を表現し、最低限：
- `lot_id`
- `asset`（通貨/銘柄）
- `acquired_at_utc`
- `qty_acquired`
- `cost_basis`（評価通貨・単価/総額）
- `source_refs`（ledger entry ids）
- `quality`（OK/DEGRADED/UNKNOWN）

### 2.3 Lot disposal（固定：売却消込）
消込は “どのlotからどれだけ売却したか” を示す：
- `disposal_id`
- `disposed_at_utc`
- `qty_disposed`
- `proceeds`（受取額）
- `fees`（該当する場合）
- `matched_lots[]`（lot_id + qty）
- `method`（FIFO/平均等：Policy）
- `quality`
- `source_refs`（ledger entry ids）

---

## 3. 入力正本：Ledger（固定）
### 3.1 必須入力
税計算は `docs/specs/accounting/order_trade_ledger_pnl_spec.md` の Ledger を入力正本とする。
固定ルール：
- Ledger window（期間）を固定参照し、曖昧参照（latest等）を禁止
- ledgerの restatement は入力に反映され、結果も再計算可能であること

### 3.2 欠損/不明の扱い（固定）
- ledger quality=UNKNOWN/DEGRADED の入力は、そのまま税結果にも品質として反映
- 不明を“確定値”として提出しない（注記・差分説明）

---

## 4. 税務計算の枠組み（固定）
### 4.1 計算ステージ（固定）
税計算は最低限以下の段階に分離される：
1) Ledger ingest window pinning（入力範囲固定）
2) Asset flow normalization（入出金/交換/手数料を資産フローへ正規化）
3) Tax lot generation（取得lot生成）
4) Lot matching（売却への消込：methodはPolicy）
5) Gain/Loss computation（実現損益、手数料、各種区分）
6) Report assembly（提出形式へ整形）
7) Evidence outputs（replay pointers / audit / integrity notes）

### 4.2 Deterministic-by-input（固定）
同じ：
- ledger window ref
- regime_version（税ルール版）
- method（FIFO/平均等）
- policy_snapshot_ref（丸め/区分/通貨等）
なら、結果は同一、もしくは差分説明が残る。

### 4.3 Unknown-handling（固定）
- 価格/レートが必要なのに入力がUNKNOWNの場合：
  - 推定値で埋めて確定しない
  - 「不足入力」「影響範囲」を明示し quality=DEGRADED/UNKNOWN とする
  - どの入力が不足かを evidence refs として残す

---

## 5. Compliance evidence（固定）
### 5.1 監査/提出のための証拠（固定要求）
少なくとも以下を “証拠として辿れる” ようにする：
- 取引履歴（fills）
- 注文履歴（orders）
- 資産変動（fees/funding/interest/deposit/withdraw）
- 税計算の lot と消込の根拠（ledger refs）
- ルール版（regime_version / policy_snapshot_ref）

### 5.2 追跡可能性（固定）
- すべての重要出力（report/lot/disposal）は replay_pointers_ref を持つ
- audit_event から report_id と replay_pointers_ref に辿れる

---

## 6. Restatement（訂正）（固定）
訂正は避けられない（取引所訂正、遅延到着、手数料訂正、ルール改定）。
固定ルール：
- 過去の報告を上書きしない
- 変更は “新しい版の報告” と “差分説明” として表現する

### 6.1 Restate record（固定要件）
- `tax_restatement_id`
- `from_report_id` → `to_report_id`
- `reason`（ledger restatement / late arrival / regime update）
- `diff_summary`（どれがどう変わった）
- `evidence_refs`（ledger window / policy snapshot / raw refs）
- `actor`（auto/manual）

---

## 7. Export safety / Access control（固定）
### 7.1 Access control（固定）
税務レポート・lot・取引詳細の閲覧/出力は最小権限で制御する：
- view-only（集計のみ、個別明細は不可 などの分離が可能）
- operator（生成・再計算の起動）
- admin（危険操作・訂正の適用）
- break-glass（緊急のみ：期限・範囲・監査必須）

固定ルール：
- 権限不足の操作は UI/APIとも拒否
- 重要拒否（denied）は audit_event に残す

### 7.2 Export safety（固定）
- export（CSV/JSON/PDF等）は secret-free（forbidden-key scan）
- 個人情報/機微情報が入る可能性がある場合：
  - マスキング/赤塗りが適用される
  - 赤塗りの適用状況が監査できる
- export 操作は必ず audit_event を残す（誰が、何を、いつ、どの範囲）

---

## 8. Integrity / Gate inputs（固定）
税計算/提出は少なくとも以下を integrity signals として出せる：
- ledger input completeness（欠損/unknown）
- fx rate / conversion inputs quality（必要なら。欠損はunknown）
- restatement counts（訂正頻度）
- report generation failures
- observability missing intervals

固定ルール：
- 監視欠損は UNKNOWN
- UNKNOWN は crosscut safety により SAFE 側へ（最低限）
- UNKNOWN を含む報告は “品質注記付き” として扱う（提出前に明示）

---

## 9. Replay pointers（固定）
replay_pointers は少なくとも参照できる：
- jurisdiction/regime/version
- ledger window ref（期間/パーティション）
- conversion inputs ref（FX/macroが必要なら）
- method（FIFO等）+ policy snapshot ref
- outputs（report_id / lots / disposals）
- binary_hash/config_hash（可能な範囲）

---

## 10. Audit（固定）
最低限、以下を audit_event として残す（秘密値なし）：
- `tax.report.generate.start/end`（jurisdiction + window + replay_pointers_ref）
- `tax.lots.generated`（counts + quality distribution）
- `tax.disposals.matched`（method + counts）
- `tax.report.exported`（format + actor + scope）
- `tax.report.restatement.applied`（from→to + reason）
- `tax.access.denied`（重要拒否）
- `integrity.record` / `gate.record`
- `support_bundle.created`（必要時）

---

## 11. テスト/検証観点（DoD）
最低限これが検証できること：

1) ledger window が曖昧参照を許さず pinning される
2) 同じ ledger window + regime_version + method + policy で同じ結果、または差分説明が残る
3) restatement が上書きでなく “新しい報告 + 差分説明” で表現される
4) report/lot/disposal が replay_pointers_ref を持ち、入力へ辿れる
5) export が secret-free（禁止キー検知）
6) 権限不足の閲覧/出力が拒否され audit_event に残る
7) UNKNOWN/欠損が integrity に必ず出る（silent loss無し）

---

## 12. Policy/Runbookへ逃がす点
- 国別税制ルール（区分/丸め/評価通貨/手数料扱い/方法）
- 提出フォーマットの細部
- 承認フロー、監査提出、問い合わせ手順
→ Policy/Runbookへ（意味は変えない）

---
End of document
