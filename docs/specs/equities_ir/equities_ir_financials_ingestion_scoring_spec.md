# Equities / IR / Financial Statements Ingestion & Scoring Core Spec v1.0（固定仕様）
IR ingestion / Filings normalization / Restatement-aware facts / Evidence replay / Scoring reproducibility

- Document ID: EQ-IR-FIN-INGEST-SCORE-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): O（Equities / IR / Financials）
- Depends-on（Fixed）:
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
- Policy separation（固定しない）:
  - 取得ソース優先順位、レート制限値、保持期間、スコア重み/閾値 → `docs/policy/**`
  - 失敗時手順/差し替え対応/監査提出手順 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
株式のIR/決算は、後から訂正・差替え・再提出が起きる。
本仕様は、IR/決算データを **欠損・改定・重複・形式差**を前提に取り込み、
それでも **真実を隠さず、再現・証明可能**にし、投資判断用スコアを “再現可能” に生成する土台を固定する。

必達要件（固定）：
1) **Raw-first evidence**：原文（PDF/HTML/XBRL/JSON等）の証拠を保存し参照できる
2) **Restatement-aware facts**：訂正/差替えは上書きせず、訂正イベントで表現する
3) **Normalization boundary**：raw（証拠）と facts（正規化）と scores（派生）を分離する
4) **No silent loss**：取得失敗/欠損/不明区間は integrity_report に必ず出す
5) **Replayable**：replay_pointers で「どの証拠・どの版・どのルールでスコアを出したか」を追える
6) **Auditability**：取得/解析/差替え/スコア算出は監査イベントで追跡できる
7) **Safety honesty**：監視欠損/整合性不明は SAFE 側へ（crosscut準拠）
8) **No secrets**：APIキー等は secret_ref のみ（平文禁止）

---

## 1. 範囲（in / out）
### 1.1 In
- 企業マスタ（ticker/市場/法人番号等）の識別と正規化境界
- IR/提出書類の取得（複数ソース、形式差の吸収）
- Raw evidence 保存（原文/メタ）
- Facts（財務数値/ガイダンス/イベント）の正規化
- Restatement（訂正・差替え）のモデル
- Scoring（投資格付け）の再現可能な枠組み
- Integrity/Gate inputs（欠損/遅延/改定/重複の表面化）
- replay pointers / audit events

### 1.2 Out
- 個別の投資戦略や売買ロジック（別ドメイン）
- UI詳細（ただし説明可能性要件は固定）
- 法規制の国別詳細（将来拡張）

---

## 2. Core Concepts（固定）
### 2.1 Issuer Identity（固定）
企業は少なくとも以下で識別できる：
- `country`（JP/US等）
- `market`（TSE/NASDAQ等）
- `ticker`（または銘柄コード）
- `issuer_id`（内部一意ID：変更に耐える）
- `name`（正規化名：参考）

固定ルール：
- ticker変更/統廃合があっても issuer_id は不変で追跡できる

### 2.2 Document / Filing Identity（固定）
提出物（IR/決算/適時開示/10-K等）は最低限：
- `doc_id`（内部一意）
- `issuer_id`
- `doc_type`（earnings_release / financial_statement / disclosure / 10-K / 10-Q / 8-K 等）
- `period`（FY/Quarter、対象期間）
- `published_at_utc`
- `source`（取得ソース識別）
- `source_doc_ref`（URL/キー/識別子）
- `content_hash`（証拠の同一性：改ざん検知）
- `version`（同一doc_type+periodの版：訂正で増える）
- `quality`（OK/DEGRADED/UNKNOWN）

### 2.3 Layers（固定）
- Raw Evidence：原文（PDF/HTML/XBRL/JSON）、メタ、hash（正本）
- Facts：抽出・正規化された数値/テキスト（派生だが再現可能）
- Scores：facts を用いた評価（派生。ルールと版が必要）

---

## 3. Raw Evidence（固定）
### 3.1 Raw-first（固定）
- 原文は保存できる（または安定参照で追跡できる）
- 原文の同一性は hash で検証できる
- 原文欠落は integrity に反映される（“取得できなかった” を隠さない）

### 3.2 Secret-free（固定）
- クッキー/トークン/認可ヘッダ等は保存しない
- URLに秘密が含まれる可能性がある場合は赤塗り（forbidden-key scan）

---

## 4. Normalization（Facts）（固定）
### 4.1 Facts の最小モデル（固定）
facts は少なくとも以下のカテゴリを扱える：
- Financial statements（PL/BS/CFの主要項目）
- KPIs（EPS, revenue growth, margin 等）
- Guidance / outlook（ガイダンス）
- Corporate actions（split, buyback, dividends）
- Events（M&A、重要契約、訴訟など：テキスト）

facts エントリは最低限：
- `fact_id`（内部一意）
- `issuer_id`
- `period`
- `fact_type`（revenue, op_income, eps 等）
- `value` + `unit` + `currency`
- `as_reported`（報告値）
- `normalized`（正規化値：例 単位換算）
- `source_doc_id`（根拠doc）
- `source_locator`（doc内位置：ページ/タグ等。可能なら）
- `quality`（OK/DEGRADED/UNKNOWN）

### 4.2 Quality（固定）
- OCR/パース失敗や曖昧抽出は UNKNOWN/DEGRADED として表面化
- UNKNOWN を “0” として扱わない
- quality はスコア計算に影響し、最終スコアにも品質注記が付く

---

## 5. Restatement（訂正・差替え）（固定）
### 5.1 原則（固定）
- 上書き禁止（append-only）
- 変更は “新しい version の doc” と “restate event” で表現する

### 5.2 Restate event（固定要件）
訂正が発生したら、少なくとも以下を記録できる：
- `restate_id`
- `issuer_id`
- `from_doc_id` / `to_doc_id`
- `period` / `doc_type`
- `reason`（人間可読）
- `diff_summary`（変更点要約：何が変わった）
- `evidence_refs`（source refs / hashes）
- `actor`（自動/手動）

固定ルール：
- restate の適用は audit_event に残す
- 旧版は “無効化” できても “削除” しない（証拠として残す）

---

## 6. Scoring（固定：再現可能な枠組み）
### 6.1 Score identity（固定）
スコア出力は最低限：
- `score_id`
- `issuer_id`
- `score_model_id`（モデル名）
- `score_model_version`（SemVer）
- `window`（対象期間）
- `inputs_ref`（facts window / doc versions）
- `policy_snapshot_ref`（重み/閾値等：Policy）
- `output`（score + sub-scores）
- `quality`（OK/DEGRADED/UNKNOWN）
- `explain`（理由コード + evidence refs）

### 6.2 Deterministic-by-input（固定）
同じ inputs_ref + 同じ model_version + 同じ policy_snapshot_ref なら：
- 同じ score を出す
- 変わるなら差分理由を記録（入力改定、モデル改定、policy改定）

### 6.3 Explainability（固定）
スコアは最低限 “説明可能”：
- サブスコア（例：収益性/成長性/健全性/株主還元）
- 理由コード
- 参照facts/doc（evidence refs）

---

## 7. Integrity / Gate inputs（固定）
### 7.1 integrity signals（固定）
- expected filings vs observed（対象企業・期間に対する欠落）
- fetch failures / parse failures
- duplicate detections（同一docの重複）
- restatement counts（訂正頻度）
- staleness（最新更新からの遅れ）
- observability missing intervals

これらは integrity_report の根拠。

### 7.2 Observability honesty（固定）
監視欠損は UNKNOWN。UNKNOWN は SAFE 側へ（crosscut）

---

## 8. Replay pointers（固定）
replay_pointers は少なくとも参照できる：
- issuer set / window
- raw evidence keys（doc_id の集合と保存参照）
- facts window（fact_id集合、またはパーティション）
- score outputs（score_id集合）
- policy snapshot ref / model version
- binary_hash / config_hash（可能な範囲）

---

## 9. Audit（固定）
最低限、以下を audit_event に残す（秘密値なし）：
- `equities.ir.fetch.start/end`（issuer/window/source）
- `equities.ir.doc.stored`（doc_id + hash）
- `equities.ir.parse.fail`（error.kind + doc_id）
- `equities.ir.facts.upserted`（counts + quality）
- `equities.ir.restatement.applied`（from→to + reason）
- `equities.score.compute.start/end`（model/version + refs）
- `equities.score.divergence`（差分説明）
- `integrity.record` / `gate.record`

---

## 10. Safety coupling（固定）
- integrity UNKNOWN/FAIL（大量欠損/監視欠損）時は SAFE 側へ
- スコア出力は “UNKNOWN/DEGRADED” を付け、確信が無いのに断定表示しない

---

## 11. テスト/検証観点（DoD）
最低限これが検証できること：

1) raw evidence が保存/参照され、hashで同一性が検証できる
2) 訂正が上書きでなく restatement として表現される
3) parse失敗や欠損が integrity_report に必ず出る（silent loss無し）
4) 同じ inputs_ref+model_version+policy_snapshot_ref で同じスコア、または差分説明が残る
5) スコアが説明可能（理由コード+evidence refs）
6) 監視欠損で gate UNKNOWN → safety SAFE 側へ寄る

---

## 12. Policy/Runbookへ逃がす点
- 取得ソース優先順位、レート制限、保持期間
- スコア重み/閾値、評価通貨、丸め
- 失敗時/差替え時の運用手順、監査提出
→ Policy/Runbookへ（意味は変えない）

---
End of document
