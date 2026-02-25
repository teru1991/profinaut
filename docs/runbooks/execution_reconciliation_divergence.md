# Execution: Reconciliation Divergence Runbook v1.0
（照合乖離 / 外部混入 / UNKNOWN 収束）

## 目的
Execution（I）で発生する「照合の不一致」を、資金毀損なしで最短収束させる。

## 参照（SSOT）
- 固定仕様: `docs/specs/ucel/execution_connector_spec.md`
- 固定仕様: `docs/specs/crosscut/safety_interlock_spec.md`
- 契約:
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/audit_event.schema.json`
- 運用値: Policy（損失/露出/照合頻度等）

---

## 0. 原則（最重要）
- 迷ったら **HALT**（最優先）  
- 外部混入（MISSING_LOCAL）は重大：**FLATTENまたはHALT**  
- UNKNOWNは放置しない：照合で必ず収束させる

---

## 1. 初動（共通）
1) Safetyを確認し、必要なら `HALT` を発動（理由/TTL/監査）  
2) Support Bundleを生成（audit_tail/logs_tail/gate_results/safety_state）  
3) 乖離の種類（カテゴリ）を確定  
   - MATCH / MISSING_REMOTE / MISSING_LOCAL / DIVERGED / STALE / DANGEROUS  
4) 対象範囲（account/venue/symbol/bot）を絞る

---

## 2. MISSING_LOCAL（外部混入：最重大）
### 症状
- 取引所に open order / fill があるが、内部に intent が存在しない

### 対応
1) 即 `HALT` か `FLATTEN`（状況により）  
2) "外部経路"を疑う（直叩き、別ツール、誤操作）  
3) 取引所側の open orders / fills / balances を取得し、事実を確定  
4) 収束方針：
   - open orderが残っているなら取消（許可される範囲で）
   - ポジが増えているなら FLATTEN（解消のみ）へ
5) 以後、必ず監査イベントに残す
   - `EXTERNAL_ORDERS_DETECTED`（推奨）
   - `MANUAL_OPERATION`（対応手順）

---

## 3. DIVERGED（数量/価格/状態が不一致）
### 症状
- internal state と取引所 state が一致しない（部分約定、取消遅延、WS/REST遅延など）

### 対応
1) open orders を照会し最新状態へ更新  
2) recent fills を取得し、internal fills を再構築  
3) balances/positions を取得し、内部台帳への反映を再確認  
4) それでも収束しない場合：
   - STALE（情報が古い）として待機＋再照会
   - 重大なら SAFE_MODE/CLOSE_ONLY へ縮退

---

## 4. MISSING_REMOTE（取引所に無い）
### 症状
- internalでは送った扱いだが、取引所に存在しない

### 対応
1) client_order_id 検索で存在確認（存在するなら二重送信禁止）  
2) 無いなら、REJECTED/EXPIREDへ収束（理由を記録）  
3) retry が必要なら policyに従い再送（必ず照会→再送の順）

---

## 5. UNKNOWN（不確実）
### 症状
- タイムアウト/通信断で送れたか不明
- ACKが無い

### 対応（固定原則）
1) まず照会（client_order_id検索）  
2) 存在すれば、その状態にinternalを収束  
3) 無ければ、MISSING_REMOTEとして扱い、必要なら再送

---

## 6. 解除（Safety解除の条件）
- 観測欠損がない（Prom/Loki）
- 照合がMATCHに収束
- 外部混入が解消（残注文ゼロ、ポジション整合）
- 解除は強操作（理由/TTL/監査）

---

## 7. 事後（必須）
- 原因分類：直叩き経路、レート制限、取引所遅延、実装不備
- 再発防止：禁止経路のGate化、Policy調整、契約テスト追加
