# Market Data: stale / gap / thin Runbook v1.0

## 目的
Market Data Collector（H）で発生する典型障害（stale / gap / thin）を、
「落とさない／戻す／証明する」を壊さずに最短で復旧する。

## 参照（SSOT）
- 固定仕様: `docs/specs/ucel/marketdata_collector_spec.md`
- 契約:
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/audit_event.schema.json`
- 運用値: `docs/policy/ucel_marketdata_thresholds.toml`

---

## 0. 原則
- 迷ったらまず **観測の健全性**（Prom/Loki）を確認する  
  → 観測が壊れていると "正常に見える" 事故が起きる
- gap は優先度が高い（orderbook整合破綻の可能性）
- Unknown急増（schema drift）は推測実装しない → Quarantine

---

## 1. 初動（共通）
1) ダッシュボードで対象を絞る  
   - venue / op(kind) / symbol（または stream_id）
2) 観測欠損がないか確認（Prom targets / Loki ingest）
3) Support Bundleを生成（必要なら）  
   - 直近の audit_tail / logs_tail / safety_state / gate_results
4) Safetyの判断  
   - Market Data単独なら通常 HALT は不要  
   - ただし "観測欠損が長い" "ディスク逼迫/IO stall" などは全体影響 → SAFE_MODE 以上を検討

---

## 2. stale（一定時間メッセージが来ない）
### 2.1 症状
- `last_message_age` が閾値超過
- subscribed数は維持されている場合もある（"つながっているが死んでいる"）

### 2.2 典型原因
- 取引所側の一時停止/メンテ
- 回線/NAT/ルータのセッション劣化
- 購読は成功していない（subscribed判定が甘い）
- backpressure（詰まり）で局所停止している

### 2.3 対応手順
1) **queue/backpressure** を確認（詰まりが先ならそちらが原因）  
2) **subscribeの成立**を確認（expected vs subscribed）  
3) **局所再接続**（conn単位）  
4) 収束しない場合、venue単位のワーカー再起動  
5) それでもだめなら "観測欠損 or 取引所メンテ" を疑い、通知レベルを調整（Policy）

### 2.4 監査（必須）
- staleが閾値を超えたら監査イベント（例：OBSERVABILITY…ではなく、原因に応じたカテゴリ）
- 手動で再起動/切替した場合は `MANUAL_OPERATION` を残す

---

## 3. gap（整合破綻：orderbook等）
### 3.1 症状
- `gap_detected` 増加
- `resnapshot_total` 増加
- book健全性監査（cross/negative）に違反

### 3.2 原則
- gapは **自動resnapshot** が基本
- 自動resnapshotが連敗する場合は **Quarantine**（推測実装禁止）

### 3.3 対応手順
1) gap発生 stream を特定（stream_id / venue / symbol / op）  
2) resnapshotが成功しているか（連敗/成功率）  
3) 回線/遅延/詰まり（WAL/queue）で resnapshotが間に合っていない可能性を確認  
4) 連敗する場合：
   - Quarantineへ（仕様不明/破綻を封印）
   - 取引所仕様変更（schema drift）を疑う（Unknown急増も併発しがち）

### 3.4 監査（必須）
- `GAP_DETECTED`
- `RESNAPSHOT_STARTED/SUCCEEDED/FAILED`
- `QUARANTINE_ENTERED`（必要なら）

---

## 4. thin（生きてるが薄い）
### 4.1 症状
- `rate_deviation` が閾値超過（expectedより明確に少ない）

### 4.2 典型原因
- 薄商い（市場要因）
- 取引所の配信仕様変更（チャンネル間引き）
- 接続/購読はあるが一部が届いていない（NAT劣化）
- 監視/集計側の欠損（観測欠損）

### 4.3 対応手順
1) まず観測欠損を除外（Prom/Loki）  
2) 同一symbolの別チャンネルと比較（tickerは来るのに tradesが薄い等）  
3) conn rotation を実施（ws_rulesに従う）  
4) 改善しない場合、Policyで「薄商い」許容を調整（ただしP0は保守的に）

### 4.4 監査（推奨）
- `MANUAL_OPERATION`（調整・再接続など）
- 重大薄化が継続するなら監査イベント化

---

## 5. 収束判定（recovered）
- expected=subscribed が回復
- stale解消
- gap収束（resnapshot成功、健全性安定）
- thinが許容範囲に戻る（Policy）

---

## 6. 最後に（Integrity）
- 日次Integrity Reportで該当期間が反映されていること
- 観測欠損があった場合は "Unknown/Degraded期間" として必ず記録されていること
