# Portfolio / Risk Management Core Spec v1.0（固定仕様）
Unified exposure model / Limits / Emergency reduction / Evidence

- Document ID: RISK-PORTFOLIO-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): J（Risk / Portfolio）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
  - Strategy Runtime: `docs/specs/strategy_runtime/bot_sdk_plugin_boundary_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/safety_state.schema.json`
- Policy separation（固定しない）:
  - 上限値（最大露出、最大損失、最大注文サイズ等）→ `docs/policy/**`
  - 緊急時運用/復旧手順 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
Riskは「戦略の暴走・外部障害・想定外」を止める最終防壁である。
本仕様は、ポジションと露出を統一モデルで扱い、上限と緊急縮退を固定保証する。

必達要件（固定）：
1) **Unified exposure model**：取引所/市場/商品差を吸収し、同じ計算基準で露出を出す
2) **Limits are enforceable**：上限違反は pre-trade gate で確実に拒否/縮退
3) **Emergency reduction**：異常時は CLOSE_ONLY/FLATTEN/BLOCK に確実に遷移できる
4) **Evidence**：露出計算の根拠（inputs/assumptions）を監査・再現できる
5) **Fail-safe**：ポジション不明/価格不明/監視欠損は安全側（新規増を拒否）
6) **No secret leakage**：秘密値は含まない
7) **Performance**：計算はホットパスを阻害しない（重いものは分離）

---

## 1. 範囲（in / out）
### 1.1 In
- ポジション/残高/注文（open orders）の統一表現
- 露出（デルタ/ノーション/レバ）計算
- リスク上限（Policy値）適用
- 緊急縮退（Kill-Switch提案/強制）
- 実行ゲートへの提供（risk snapshot）
- 監査用の計算根拠（input refs）

### 1.2 Out
- 具体の戦略ロジック
- 具体の市場データ収集（Collector）
- 具体の注文送信（Execution Safety）

---

## 2. 統一ポジションモデル（固定）
### 2.1 最低限の共通フィールド（固定）
- `venue`
- `account`（将来複数口座対応のため概念として持つ）
- `market_type`（spot/linear/inverse/option 等）
- `symbol`
- `position_side`（long/short/flat）
- `qty`（数量）
- `entry_price`（平均建値：取得不能なら unknown）
- `mark_price`（評価価格：入力品質付き）
- `notional`（評価額）
- `leverage`（該当する場合）
- `margin_mode`（isolated/cross 等）
- `timestamp_utc`
- `quality`（OK/DEGRADED/UNKNOWN）

### 2.2 Open Orders（固定）
- `order_id` / `client_order_id`
- `side` / `qty` / `price`
- `reduce_only`
- `status`
- `timestamp_utc`

---

## 3. 露出モデル（固定）
### 3.1 Exposure types（固定）
- `gross_notional`：総ノーション
- `net_notional`：ネット（ロング/ショート差）
- `delta_exposure`：方向性露出（可能なら）
- `leverage_exposure`：レバ関連
- `concentration`：単一symbol/venueへの偏り
- `liquidity_risk`：板厚/スリッページ見積（入力品質次第でUNKNOWN許容）

### 3.2 不確実性（固定）
- mark_price や qty が UNKNOWN の場合：
  - exposure は UNKNOWN を許容する
  - UNKNOWN を “安全” と扱わず、新規増を拒否する

---

## 4. Risk Limits（固定）
### 4.1 Limit categories（固定枠組み）
（具体値は Policy）
- Max gross exposure（総露出上限）
- Max net exposure（ネット上限）
- Max per-symbol exposure
- Max per-venue exposure
- Max leverage
- Max drawdown / loss（評価損失上限）
- Max order size / rate（注文上限）
- Max open orders

### 4.2 Enforcement（固定）
- pre-trade gate は risk snapshot を参照し、上限違反は必ず拒否
- “上限違反の恐れ” がある場合は constraints 付き許可（reduce-only強制 等）
- Limit判定ができない（UNKNOWN）場合：
  - 新規増を拒否（CLOSE_ONLY 相当）

---

## 5. Emergency reduction（固定）
### 5.1 Trigger sources（固定）
以下のいずれかで緊急縮退できる：
- Safety Mode（SAFE/EMERGENCY_STOP）
- Integrity FAIL/UNKNOWN
- Venue health collapse（auth failure/429 storm）
- Risk limit hard breach（重大超過）
- Operator dangerous op（明示）

### 5.2 Actions（固定語彙）
Execution Kill-Switch へ反映する：
- `CLOSE_ONLY`
- `FLATTEN`
- `BLOCK`
（上げる方向は常に許可、下げる方向は dangerous op）

---

## 6. Risk Snapshot（固定）
### 6.1 Snapshot contents（固定要求）
risk snapshot は最低限以下を含む：
- positions summary（quality付き）
- open orders summary
- computed exposures（UNKNOWN含む）
- limit evaluation result（OK/WARN/BREACH/UNKNOWN）
- evidence refs（prices source, positions source, window）

### 6.2 Freshness（固定）
- snapshot が古い/不明なら UNKNOWN 扱い
- UNKNOWN は安全側（新規増拒否）

---

## 7. Audit / Evidence（固定）
最低限、以下を audit_event に残す：
- `risk.snapshot.recorded`（要約 + refs）
- `risk.limit.breached`（どのlimit、どれだけ）
- `risk.killswitch.recommended`（CLOSE_ONLY/FLATTEN/BLOCK）
- `risk.data.unknown`（positions/prices不明）
- `execution.gate.decision`（risk理由コード含む）

秘密値は含めない。

---

## 8. テスト/検証観点（DoD）
最低限これが検証できること：

1) positions/prices が UNKNOWN のとき新規増が拒否される
2) limit breach で pre-trade gate が拒否/縮退する
3) killswitch が確実に強化方向へ入る（CLOSE_ONLY/FLATTEN/BLOCK）
4) snapshot と limit評価が監査可能（根拠refsあり）
5) safety SAFE/EMERGENCY_STOP で実行が止まる

---

## 9. Policy/Runbookへ逃がす点
- 各limitの具体値、通知、評価頻度、許容誤差
- 緊急時の運用（flatten順序、手動介入）
→ Policy/Runbookへ（意味は変えない）

---
End of document
