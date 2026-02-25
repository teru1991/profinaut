# Execution Failures Playbook v1.0
（拒否/429/取消失敗/遅延/UNKNOWN）

## 目的
Execution（I）で発生する失敗を安全に収束し、二重発注や資金毀損を避ける。

## 参照（SSOT）
- 固定仕様: `docs/specs/ucel/execution_connector_spec.md`
- 固定仕様: `docs/specs/crosscut/safety_interlock_spec.md`

---

## 0. 原則
- 不明ならHALT（特にUNKNOWNや外部混入）
- retry前に必ず照会（client_order_id検索）
- 取消失敗は常識（再照会で状態を確定して収束）

---

## 1. 入口：失敗のタイプ
- [ ] REJECT（署名/権限/入力不正）
- [ ] 429 / RateLimit（BAN兆候含む）
- [ ] Timeout/Network（送れたか不明：UNKNOWN）
- [ ] Cancel failed（取消失敗/遅延）
- [ ] Partial fill（部分約定）
- [ ] Reconciliation divergence（照合乖離）

---

## 2. REJECT（拒否）
チェック：
- [ ] エラーカテゴリ（Auth/Schema/State/RateLimit）
- [ ] 入力（qty/price/tif/post_only等）整合
対応：
- [ ] 修正して再送（ただし同intentならidempotency維持）
- [ ] 連発するなら SAFE_MODE（新規を止める）

監査：
- `MANUAL_OPERATION`（修正）
- 重大なら `SIGNATURE_VERIFICATION_FAILED` 等

---

## 3. 429 / RateLimit
対応：
- [ ] Policyの上限を超えない（burst抑制）
- [ ] backoff+jitterで待機
- [ ] 連続なら SAFE_MODE / HALT（状況により）

監査：
- `MANUAL_OPERATION`（抑制）
- 重大なら監査イベント化（将来type拡張）

---

## 4. Timeout/Network（UNKNOWN）
最重要手順（固定）：
1) client_order_id検索
2) 存在する → 状態をinternalへ収束
3) 存在しない → MISSING_REMOTEとして扱い、必要なら再送（照会→再送）

Safety：
- 影響範囲が大きいなら SAFE_MODE

---

## 5. Cancel失敗
対応：
- [ ] 取消要求→再照会（open orders）
- [ ] 取引所側の状態にinternalを収束
- [ ] 取消できない場合は CLOSE_ONLY/FLATTEN でリスクを止める

---

## 6. 部分約定
対応：
- [ ] fills取得でinternal fillsを再構築
- [ ] 残注文の扱い（取消/継続）を決定
- [ ] 必要なら CLOSE_ONLY/FLATTEN

---

## 7. 収束判定
- [ ] UNKNOWNが残っていない
- [ ] ReconciliationがMATCHに収束
- [ ] Safety解除は強操作（理由/TTL/監査）

---

## 8. 証明
- [ ] audit_tail（重要イベント）
- [ ] support_bundle（必要なら）
