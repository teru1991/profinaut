# Collector Recovery Playbook v1.0
（Market Data Collector：隔離/復旧/証明）

## 目的
Market Data Collector（H）を、最小の影響範囲で復旧し、欠損・観測欠損・改変を隠さず証明する。

## 参照（SSOT）
- 固定仕様: `docs/specs/ucel/marketdata_collector_spec.md`
- 契約:
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/audit_event.schema.json`
- Policy: `docs/policy/ucel_marketdata_thresholds.toml`

---

## 0. 原則
- **最小隔離→局所復旧→段階拡大**（topic→conn→venue→process）
- 観測欠損を除外できない限り"正常"とは言わない
- gap/Unknown急増は推測で直さずQuarantine

---

## 1. 入口：障害のタイプを決める（チェック）
- [ ] stale（last_message_age超過）
- [ ] gap（seq欠落/整合破綻、resnapshot失敗）
- [ ] thin（rate expectation乖離）
- [ ] backpressure（queue_len / wal_latency / disk/IO）
- [ ] schema drift（Unknown急増）
- [ ] observability outage（targets down / ingest停止）

---

## 2. 最優先：観測の健全性（チェック）
- [ ] Prom targets はUPか
- [ ] Loki ingest は動いているか
- [ ] 欠損期間があるなら Unknown/Degraded として扱う（復旧の前提）

監査（必須）：
- `OBSERVABILITY_TARGETS_DOWN` / `OBSERVABILITY_INGEST_STOPPED`

---

## 3. 復旧順序（固定）
### Step 1: topic復旧
- [ ] 該当stream_idの再同期（bookならresnapshot）
- [ ] 収束しなければQuarantine

監査：
- `GAP_DETECTED`
- `RESNAPSHOT_*`
- `QUARANTINE_ENTERED`

### Step 2: conn復旧
- [ ] conn再接続（指数+jitter+storm guard）
- [ ] subscribe再実行（レート制御遵守）
- [ ] rotation（計画再接続）も検討

監査：
- `MANUAL_OPERATION`（手動介入の場合）

### Step 3: venue復旧
- [ ] venueワーカー再起動
- [ ] 期待購読（expected）再計算と乖離確認

### Step 4: process復旧
- [ ] プロセス再起動（最後）
- [ ] WAL復旧チェック
- [ ] 起動レポート生成（StartupReport）

監査：
- `WAL_RECOVERY_PERFORMED`（復旧実施時）

---

## 4. backpressure/durability（チェック）
- [ ] wal_queue_len が危険域か
- [ ] wal_append_latency が危険域か
- [ ] disk_free が危険域か
- [ ] IO stall 兆候があるか

対応（段階停止の考え方）：
- P2→P1→P0の順に落とし、P0維持
- ディスク逼迫は最優先で解決（退避/圧縮/拡張）

監査：
- `DISK_PRESSURE_*`
- `IO_STALL_*`

---

## 5. schema drift（Unknown急増）
- [ ] Unknownが急増しているstreamを特定
- [ ] Quarantineへ（推測実装禁止）
- [ ] 取引所仕様変更の可能性として調査（契約更新/adapter更新）

監査：
- `QUARANTINE_ENTERED`
- `MANUAL_OPERATION`（調査/適用）

---

## 6. recovered判定（チェック）
- [ ] expected == subscribed（許容乖離内）
- [ ] stale解消
- [ ] gap収束（resnapshot成功、健全性安定）
- [ ] thin収束（許容内）
- [ ] Quarantineが解除（必要な場合）
- [ ] 観測欠損が解消

---

## 7. 証明（必須）
- [ ] Daily Gate結果（PASS/WARN/FAIL）
- [ ] Integrity Report（日次、該当期間）
- [ ] 重大イベントのaudit_tail確認
- [ ] Support Bundle生成（必要なら）
