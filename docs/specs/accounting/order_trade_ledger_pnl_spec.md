# Order/Trade Ledger & PnL / Accounting Core Spec v1.0（固定仕様）
Immutable event ledger / PnL reproducibility / Restatement / Evidence

- Document ID: ACCT-LEDGER-PNL-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): K（Accounting / Ledger / PnL）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
  - Risk/Portfolio: `docs/specs/risk/portfolio_risk_management_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/integrity_report.schema.json`
- Policy separation（固定しない）:
  - 手数料体系の運用値、評価通貨、税区分、保持期間 → `docs/policy/**`
  - 訂正手順/監査対応/エスカレーション → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、注文・約定・資産変動を **改ざん耐性のある台帳**として固定し、
PnL/成績/残高が「あとから証明・再現」できる状態を保証する。

必達要件（固定）：
1) **Immutable ledger**：台帳は追記のみ（append-only）。過去を上書きしない
2) **Deterministic PnL by input**：同じ台帳入力と同じ評価ルールで同じPnL、または差分説明が残る
3) **Idempotent ingest**：重複取り込み・再送で二重計上しない
4) **Restatement is explicit**：訂正は “訂正イベント” で表現し、履歴と理由を残す
5) **Evidence-first**：replay_pointers で入力範囲と生成物を追跡できる
6) **No secrets**：秘密値は一切含めない
7) **Isolation**：env（dev/stage/prod）と mode（paper/shadow/live）は交差しない
8) **Performance**：高頻度でも破綻しない（集計は段階化・インデックス化は実装自由）

---

## 1. 範囲（in / out）
### 1.1 In
- 注文イベント（place/amend/cancel）
- 約定イベント（fills/trades）
- 残高/資産変動イベント（deposit/withdraw/fees/funding/interest等）
- ポジション・評価情報（必要な範囲で参照）
- 台帳の正規化・重複排除・整合チェック
- PnL（実現/未実現）・成績指標の算出枠組み
- Restatement（訂正）モデル
- 監査・再現（audit/replay pointers）

### 1.2 Out
- 税務の国別詳細（別ドメインで扱える）
- UI（可視化）
- 戦略判断

---

## 2. Ledgerモデル（固定：概念契約）
### 2.1 Ledger Entry の基本形（固定）
台帳は “エントリの列” であり、各エントリは最低限以下を持てる：

- `entry_id`（内部一意・event_uid相当）
- `entry_type`（ORDER / FILL / BALANCE / ADJUSTMENT / RESTATEMENT）
- `venue` / `account`
- `market_type` / `symbol`（該当する場合）
- `ts_event_utc`（取引所の事実時刻。無い場合は受信時刻）
- `ts_ingest_utc`（取り込み時刻）
- `qty` / `price` / `notional`（該当する場合）
- `fee`（通貨/金額。Policyにより扱いが変わっても “値として記録” は固定）
- `side`（buy/sell、該当する場合）
- `ref_ids`（外部ID：order_id、trade_id、tx_id 等）
- `quality`（OK/DEGRADED/UNKNOWN）
- `trace_id` / `run_id` / `schema_version`

固定ルール：
- `entry_id` は安定的に生成し、重複排除に使えること
- `quality=UNKNOWN` を許容するが、隠蔽しない

### 2.2 Entry types（固定）
- ORDER：注文の状態変化（accepted/rejected/open/partially_filled/canceled等の遷移を表現）
- FILL：約定（取引所の “事実” の最小単位）
- BALANCE：資産変動（fee/funding/interest/pnl transfer等）
- ADJUSTMENT：内部調整（ただし “過去の上書き” ではなく追加エントリ）
- RESTATEMENT：訂正（後述）

---

## 3. Idempotency / Dedup（固定）
### 3.1 原則（固定）
- 同じ外部事実は二重計上しない
- 再送/再接続/再取り込みは正常系であり、台帳は吸収する

### 3.2 Dedupe keys（固定の優先順位）
- FILL：外部 trade_id/exec_id があるならそれを最優先
- ORDER：外部 order_id + status transition + timestamp（または sequence）
- BALANCE：外部 tx_id があるならそれ、無い場合は (type, ts, amount, currency, payload_hash)

固定ルール：
- Dedupe不能（外部ID欠損かつ曖昧）な場合は quality=DEGRADED/UNKNOWN とし、Integrityに影響させる
- Dedupe抑止はメトリクス/監査で観測できること（silentに捨てない）

---

## 4. Ordering / Causality（固定）
台帳は必ずしも “到着順=事実順” ではない。
固定要件：
- `ts_event_utc` と `ts_ingest_utc` を分離して記録
- 事実順の再構成に必要な情報（sequence / external ids）が無い場合は UNKNOWN を表明

---

## 5. PnLモデル（固定：枠組み）
### 5.1 PnL分類（固定）
- Realized PnL（実現）
- Unrealized PnL（未実現）
- Fees（手数料）
- Funding/Interest（資金調達/利息）
- Other adjustments（調整）

固定ルール：
- PnLは “Ledger入力” を唯一の正本として計算される
- 価格参照（mark）を使う未実現PnLは、価格入力の quality を必ず保持し UNKNOWN を許容

### 5.2 Cost basis（固定）
コスト計算方式（FIFO/LIFO/平均等）は Policy で選べるが、
固定条件として：
- 選択した方式とその版（policy snapshot ref）を結果に必ず刻む
- 同じ方式＋同じ台帳＋同じ価格入力で同じPnL、または差分説明が残る

---

## 6. Restatement（訂正：固定）
### 6.1 なぜ必要か
取引所側の遅延訂正、欠落の後追い、シンボル変更、手数料訂正等で、
過去の解釈が変わり得る。

### 6.2 固定ルール（重要）
- 過去のエントリを上書きしない
- 訂正は `RESTATEMENT` エントリとして追加し、以下を必ず持つ：
  - `restates_entry_ids[]`（対象）
  - `reason`（人間可読）
  - `diff`（どのフィールドがどう変わったか）
  - `actor`（人/サービス）
  - `evidence_refs`（外部根拠、replay pointers等）
- Restatement の実行は dangerous op 扱いになり得る（Policy/運用次第）  
  ただし “記録されること” は固定要求。

---

## 7. Integrity / Evidence（固定）
### 7.1 integrity signals（固定）
- expected vs observed fills（execution/collector側との突合）
- missing intervals（台帳欠損疑い）
- duplicates suppressed（重複抑止数）
- late arrivals（遅延到着）
- restatement counts
- price input quality（未実現PnLの品質）

これらは integrity_report の根拠となる。

### 7.2 replay pointers（固定）
PnL集計やレポート出力は replay_pointers を生成し、
少なくとも以下を参照できる：
- ledger input window（期間/パーティション）
- policy snapshot ref（cost basis/fee rules）
- price inputs ref（mark prices source window）
- outputs ref（pnl report artifacts）
- binary_hash/config_hash

---

## 8. Audit（固定）
最低限、以下を audit_event に残す：
- `ledger.ingest.start/end`（window/venue）
- `ledger.dedupe.suppressed`（counts + reason）
- `ledger.integrity.note`（missing/unknown）
- `pnl.compute.start/end`（window + refs）
- `pnl.compute.divergence`（差分説明）
- `ledger.restatement.applied`（対象 + reason + refs）
秘密値は含めない。

---

## 9. Isolation（固定）
- env（dev/stage/prod）を交差させない
- paper/shadow/live を交差させない
- 交差の疑いは Integrity FAIL として扱えること

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) 同じ台帳入力＋同じpolicyでPnLが一致、または差分説明が残る
2) trade_id が重複しても二重計上しない（dedupe）
3) 遅延到着/順序乱れがあっても quality と根拠が残る
4) restatement が “追加イベント” として表現され、上書きされない
5) replay_pointers で入力範囲と結果が辿れる
6) env/mode の交差が起きない

---

## 11. Policy/Runbookへ逃がす点
- fee/funding/interest の扱い（評価通貨・丸め・手数料計算の運用値）
- cost basis の選択
- 保持期間・集計頻度
- restatement 運用（承認フロー、監査対応）
→ Policy/Runbookへ（意味は変えない）

---
End of document
