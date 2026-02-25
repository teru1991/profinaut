# Safety Operations Runbook v1.0

## 目的
- SafetyState（NORMAL/SAFE_MODE/CLOSE_ONLY/FLATTEN/HALT）を正しく運用する

---

## 1. 発動（強操作）
- `HALT`：最優先。異常時は迷わずHALT
- `FLATTEN`：解消のみ許可
- `CLOSE_ONLY`：増やす禁止
- `SAFE_MODE`：縮退運転

発動時は必ず：
- 理由（短文）
- 期限（TTL）
- 監査イベント（MANUAL_OPERATION等）

---

## 2. 解除（強操作）
解除は危険。必ず以下を満たす：
- 原因が特定/収束している
- 監視が生きている（観測欠損がない）
- Executionなら照合がMATCHに収束している
- Collectorなら gap/Unknownが落ち着いている

解除時も必ず：
- 理由
- TTL
- 監査イベント

---

## 3. 推奨運用
- "不安なときはSAFE_MODE"
- "実弾系の不明はHALT"
- "外部混入はFLATTENまたはHALT"
