# UCEL Market Data Collector Spec v1.0（固定仕様）
Golden Standard Market Data Collection（H）

- Document ID: UCEL-MD-COLLECTOR-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): H（Market Data Platform）
- Depends-on: UCEL-SDK-CORE-SPEC
- Contracts SSOT:
  - 監査：`docs/contracts/audit_event.schema.json`
  - 起動：`docs/contracts/startup_report.schema.json`
  - Gate：`docs/contracts/gate_results.schema.json`
  - Integrity：`docs/contracts/integrity_report.schema.json`
  - Support：`docs/contracts/support_bundle_manifest.schema.json`
- Goal:
  - Public market data の常時収集を最高レベルで成立（落とさない/戻す/証明する）
  - SSOT三点セット、WAL境界、Runtime/Daily Gate、二重起動を固定仕様として定義
- Non-goals:
  - 発注/残高（I）
  - 下流DB/分析（別ドメイン）
  - 閾値・保持・接続数（Policy）

---

## 0. 不変原則（Non-negotiable）
1. **Collector First**：全停止しない。局所障害で落ち、自動復旧。  
2. **SSOT完全性**：定義があるのに購読していない状態を許さない。  
3. **Raw-first**：受信事実を最優先に保持し、後段は再生成可能。  
4. **Observability is honest**：監視欠損を重大扱いし、見えてない期間を隠さない。  
5. **WAL境界**：「落とさない」の最小保証はWAL永続化。  
6. **Two-node readiness**：Active/Shadowで欠損最小化。  
7. **Daily proof**：Integrity Reportで成立を説明できる。  
8. **No guessing**：壊れ/仕様不明はQuarantineで封印。

---

## 1. スコープ（固定）
- Public market data（trades/orderbook/ticker等）
- stale/gap/thin検知→復旧
- 保存：Raw-first + WAL（durabilityの境界）
- 観測：メトリクス/ログ/アラート契約（具体閾値はPolicy）
- Gate：CI/Runtime/Daily（合否機械判定）
- 二重起動：Shadow/Active（切替条件カテゴリ固定）

---

## 2. SSOT三点セット（起動の前提）
CollectorはSSOT三点セットが揃って初めて起動対象。

1) coverage_v2：全チャンネル定義（schema_version必須）  
2) ws_rules：運用ルール（上限/heartbeat/再接続/rotation/freshness/rate expectation/Quarantine条件、schema_version）  
3) symbols：ペア母集団（REST優先、fallback/last-known、増減監査、schema_version）

**DoD（固定）**
- coverage：網羅
- ws_rules：Full（Partial禁止）
- symbols：非空・復旧可能
- schema_version不整合：起動拒否（またはDegraded停止）

---

## 3. 期待購読計画（plan_hash）と起動レポート
- 起動時に expected_topics_total を機械生成し plan_hash を算出
- 起動レポート（StartupReport）を秘密なしで生成（SSOTに準拠）

---

## 4. 状態機械（subscribed/stale/gap/thin/recovered）
### subscribed
- 期待topic生成 + ack or first message seen により成立

### stale/gap/thin
- stale：last_message_age
- gap：整合破綻（seq欠落/適用不能）
- thin：rate expectation 乖離

### recovered（不変条件）
- 期待購読数に戻る
- stale解消
- book再同期完了（対象の場合）
- 健全性が一定期間安定
- Quarantineではない

---

## 5. Orderbook整合（核）
- snapshot+delta整合
- gap検知→自動resnapshot→戻らなければQuarantine
- 健全性監査カテゴリ（固定）：crossed spread / negative size / levels逸脱 / checksum mismatch（提供される場合）

---

## 6. 隔離と自動復旧（固定）
隔離階層：topic → conn → venue → process → Quarantine

復旧手段カテゴリ（固定）：
- 再接続（指数+jitter+storm guard）
- rotation（max_connection_age）
- backpressure（bounded queue、strict）

---

## 7. Raw-first保存（受信事実）
### Raw共通ヘッダ（必須概念）
- collector_instance_id / conn_id / stream_id
- ts_recv_utc / ts_exchange
- payload（原文）
- parse_status
- plan_hash / ssot_hash / binary_hash

### event_uid（統合）
- 外部ID優先、無ければ決定的生成
- 二重起動時は「重複だけ除去し欠損は残す」が成立すること

---

## 8. WAL境界（durability）
- "落とさない"最小保証は WAL永続化
- WAL遅延/キューは観測し、危険域は段階停止（P0維持）

起動時復旧（固定）：
- WAL整合チェック、救出不可区間は隔離＋監査イベント化

---

## 9. 時刻規律
- ts_recv_utcはUTC固定
- clock drift/逆行は監査イベント
- drift中はrecovered判定を保守化

---

## 10. 観測性（固定カテゴリ）
- 接続：connected/reconnect/connect_fail
- 購読：expected_topics/subscribed_topics/subscribe_fail/subscribe_rate
- 受信：inbound msgs/bytes
- 欠損：last_message_age/gap_detected/resnapshot/rate_deviation
- 詰まり：queue_len/backpressure_events
- 永続：wal_append_latency/wal_queue_len
- 時刻：clock_skew_estimate
- ディスク：disk_free
- Quarantine：enter/exit/streams_total

### 観測の観測（固定）
- Prom targets down / Loki ingest停止は重大
- 欠損期間はUnknown/Degradedとして表示し、Integrityに記録

### 受信→永続化整合監査（固定）
- inbound_msgs と persisted_count の差分を監査

---

## 11. Runtime Coverage Gate（固定）
- expected vs subscribed の乖離を常時計測
- 乖離継続はDegraded＋復旧ループ（再購読/conn再生成）

---

## 12. Schema Drift検知（固定）
- parse_status=Unknown急増検知
- Unknown継続streamはQuarantine（推測実装禁止）

---

## 13. 二重起動（Shadow/Active）
- Active：主運用
- Shadow：収集・保存中心（比較・バックアップ）

切替条件カテゴリ（固定）：
- P0 freshness健全
- gap増加なし
- WAL健全
- plan_hash一致
- clock健全
- P0 Quarantineなし

リハーサル頻度はPlan/Runbookへ委譲。

---

## 14. Integrity Report（固定）
日次（または任意期間）で以下を秘密なしで出力できること（契約はSSOT）。
- coverage達成率
- freshness達成率（P0）
- gap/resnapshot/quarantine一覧
- 受信vs永続化差分
- WAL/disk/IO/clock危険域滞在
- 観測欠損の有無と時間
- binary_hash/ssot_hash/plan_hash紐付け

---

## 15. Gate（CI/Runtime/Daily：固定）
- CI：SSOT整合、expected生成、禁止キー検知、依存固定（Uと連携）
- Runtime：起動レポート、coverage、stale/gap/thin、WAL/clock、観測欠損
- Daily：Integrity Report生成＋Policy閾値でPASS/WARN/FAIL

---

## 16. Versioning（SemVer）
- MAJOR：状態機械/WAL境界/必須概念の破壊
- MINOR：後方互換拡張
- PATCH：表現修正
