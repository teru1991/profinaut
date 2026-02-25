# UCEL Execution Connector Spec v1.0（固定仕様）
Unified Execution Connector & OMS/EMS Contract（I）

- Document ID: UCEL-EXE-CONNECTOR-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): I（Execution Platform）
- Depends-on: UCEL-SDK-CORE-SPEC, `docs/specs/crosscut/safety_interlock_spec.md`
- Contracts SSOT:
  - 監査：`docs/contracts/audit_event.schema.json`
  - 安全状態：`docs/contracts/safety_state.schema.json`
- Goal:
  - "唯一の発注出口"として、OrderIntent→Order→Fill→Balance/Position を安全・冪等・照合可能に成立させる
- Non-goals:
  - 戦略判断（M/L）
  - 資金台帳（K）
  - 閾値（Policy）

---

## 0. 不変原則（Non-negotiable）
1. **Single Exit**：実弾発注は必ずこの経路を通る（直叩き禁止）。  
2. **Intent-first**：内部SSOTは OrderIntent。外部order_idは結果。  
3. **Idempotent-by-design**：再試行で二重発注しない。  
4. **Fail-closed**：不明/危険/観測不能は縮退（Safetyが優先）。  
5. **Reconciliation mandatory**：注文/約定/残高を必ず照合し矛盾は収束させる。  
6. **Paper/Shadow/Live parity**：同一経路で切替可能。  
7. **No secrets**：秘密非漏洩（secret_ref、赤塗り）。

---

## 1. 責務境界（固定）
Execution Connectorが提供する：
- OrderIntent受付（API/SDK）
- 注文送信/訂正/取消/照会
- Fill取得（WS/REST差分吸収）
- balances/positions/open orders取得
- error taxonomy＋retryable判断
- idempotency/dedup/replay protection
- reconciliation＋self-heal
- Safety Interlock強制（HALT等の優先）
- 観測（発注遅延、失敗率、照合差分）
- 監査（Intent/遷移/乖離/強制縮退）

提供しない：
- リスク判断（J）
- 資金配分（K）
- bot管理（L）
- UI（S）

---

## 2. 内部SSOT：OrderIntent（固定）
必須概念：
- intent_id（決定的：冪等性の核）
- trace_id/run_id/adapter_id/schema_version
- account_id（秘密なし参照）
- venue/market/symbol
- side/type/qty/price/tif
- reduce_only/close_only（該当時）
- client_order_id（外部へ渡す冪等キー：intent_idから決定的生成）
- policy_tags（適用されたルール参照）
- created_at_utc

---

## 3. 注文状態機械（固定）
状態（最小）：
- NEW → SENT → ACKED → OPEN
- PARTIALLY_FILLED → FILLED
- CANCEL_REQUESTED → CANCELED
- REJECTED / EXPIRED
- UNKNOWN（照合で解消するまで危険扱い）

遷移ルール（固定）：
- ACKが無いvenueでも照会/first seenで確定できる道を持つ
- UNKNOWNは最終状態ではない（reconciliationで収束）

---

## 4. Idempotency / Dedup（固定）
- client_order_id は intent_id から決定的生成（規約はadapterが制約に合わせる）
- retry前に照会（client_order_id検索）を優先し、二重発注を防ぐ
- retry は retryable=true の場合のみ（回数/間隔はPolicy）

---

## 5. Reconciliation（照合：固定）
照合対象（最低限）：
- open orders
- recent fills
- balances/positions
- local intent/order state

照合結果カテゴリ（固定）：
- MATCH
- MISSING_REMOTE
- MISSING_LOCAL（外部混入＝重大）
- DIVERGED
- STALE
- DANGEROUS（縮退必須）

自動復旧（固定）：
- MISSING_REMOTE：照会再実行→収束（REJECTED/EXPIRED等）
- MISSING_LOCAL：外部混入として隔離＋Safety連動（CLOSE_ONLY/FLATTEN/HALTのいずれか）
- DIVERGED：fill/照会で再構築し収束
- DANGEROUS：即時縮退＋監査必須

---

## 6. Safety Interlock強制（固定）
Safety State は全経路に優先する（`docs/specs/crosscut/safety_interlock_spec.md`）。

- SAFE_MODE：新規リスク増加行為禁止
- CLOSE_ONLY：建玉増加禁止
- FLATTEN：解消のみ許可
- HALT：発注停止（照会/監査/診断は可）

---

## 7. Observability（固定カテゴリ）
- intent_received_total
- orders_sent_total / orders_acked_total / rejects_total
- cancel_requested_total / cancel_failed_total
- fills_total / fill_latency_ms
- reconciliation_runs_total / reconciliation_diverged_total / external_orders_detected_total
- end_to_end_order_latency_ms（intent→ack/fill）
- error_total（category別）
- safety_mode_state

---

## 8. Audit / Replay（固定）
- Intentと主要遷移、照合差分、強制縮退は監査イベントとしてappend-onlyに残る
- intent_id/trace_idで「なぜ出たか」を追跡できる

---

## 9. Paper / Shadow / Live（固定）
- 同一ABI/同一経路で切替可能
- Shadowは送信しないが検証/照合が回る等、差は明示的に契約化

---

## 10. Versioning（SemVer）
- MAJOR：Intent必須概念/状態機械/照合カテゴリの破壊
- MINOR：後方互換拡張
- PATCH：表現修正
