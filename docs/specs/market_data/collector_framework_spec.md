# Market Data Collector Framework Core Spec v1.0（固定仕様）
WS/REST Ingestion / Normalization Boundary / Quarantine / Replayability

- Document ID: MD-COLLECTOR-FW-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): D（Market Data Ingestion / Collector）
- Depends-on（Fixed）:
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/startup_report.schema.json`
  - `docs/contracts/safety_state.schema.json`
- UCEL references（正本）:
  - `docs/specs/ucel/*`（symbols / ws ingest / normalization / coverage 等）
- Policy separation（固定しない）:
  - 再接続間隔、バックオフ、subscribe上限、優先度(P0/P1/P2)、欠損許容、保持期間 → `docs/policy/**`
  - 復旧手順/隔離解除/手動介入 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、あらゆる取引所/市場の Market Data を、**欠損・遅延・重複・順序乱れ・再送・部分停止**を前提に、
それでも **安全・堅牢・高速・安定**に取り込み、再現可能な証拠を残す “Collector Framework” の不変条件を固定する。

必達要件（固定）：
1) **Raw-first**：外部から受け取った事実は、可能な限り未加工で保存できる（再現の起点）
2) **No silent loss**：欠損/未知/監視不能は “無かったことにしない”（Integrityで必ず表面化）
3) **Dedup + Ordering tolerant**：重複・順序乱れは正常系として吸収する
4) **Quarantine**：異常は隔離し、全体停止に波及させない（ただしP0超はSafetyに連動）
5) **Replayable evidence**：replay_pointers で再現可能な入力範囲を指せる
6) **Coverage discipline**：購読対象はSSOTで定義され、契約と整合し続ける
7) **Performance**：高頻度データでもバックプレッシャで破綻せず、劣化は制御される
8) **Safety honesty**：観測不能/整合性不明は SAFE 側へ（crosscut準拠）

---

## 1. Collector Framework の責務境界
### 1.1 In（本仕様の対象）
- WS/REST 受信（接続、再接続、購読、再購読）
- Raw-first ingest（受信フレームの証拠化）
- 正規化境界（raw→canonical の責務分離）
- 重複排除、順序乱れ許容、スナップショット同期
- Backpressure / load shedding（ただし silent drop しない）
- Quarantine（隔離・復旧・証跡）
- Integrity / Gate inputs の生成
- Replay pointers の生成

### 1.2 Out（本仕様の対象外）
- 具体的な保存形式（parquet/avro/jsonl等）
- 具体的なDB/オブジェクトストア選定
- UI/ダッシュボード表示
（ただし “どの証拠を残すべきか” は固定）

---

## 2. 基本モデル（固定）
### 2.1 Stream Identity（固定）
Collectorは “何を収集しているか” を一意に識別できる必要がある。

必須フィールド（固定）：
- `venue`（取引所）
- `market_type`（spot/linear等：UCELに準拠）
- `symbol`（正規化済み識別子：UCEL symbol masterに準拠）
- `channel`（trades/orderbook/ticker等）
- `connection_id`（同一venue内の接続実体識別）
- `stream_id`（上記を安定に連結した識別子：監査/隔離/整合のキー）

### 2.2 Message Classes（固定）
外部入力は最低限以下に分類できる：
- `RawFrame`：受信したままのWSフレーム/RESTレスポンス
- `CanonicalEvent`：正規化されたイベント（型はUCEL）
- `ControlEvent`：購読/再購読/再接続/スナップ取得/シーケンス再同期

### 2.3 Raw-first Boundary（固定）
- RawFrame は “受信事実の証拠” である
- CanonicalEvent は “利用しやすさ” のための派生である
固定ルール：
- RawFrame の欠落は **再現性の欠落**として扱い、Integrityに影響する
- CanonicalEvent の欠落は **利用欠落**として扱い、別途Integrityに影響する
- どちらも silent に隠蔽しない

---

## 3. Subscription SSOT（固定）
購読対象は SSOT で定義されること。

### 3.1 SSOTの要件（固定）
- “何を収集するか” は一覧で固定できる（symbols/channels/markets）
- 追加/削除は計画（Plan）と手順（Runbook）を伴う
- 実行時に SSOT と実際の購読が一致しているか検証できる（Gate）

### 3.2 Coverage Gate（固定）
- 期待購読（expected）と実購読（observed）を比較し、差分を gate_results として出せる
- 観測不能（metrics down 等）なら “UNKNOWN” として SAFE 側へ連動（crosscut）

---

## 4. 接続・再接続（固定）
### 4.1 Reconnect discipline（固定）
- 再接続は必ず backoff + jitter（値はPolicy）
- 失敗が連続する場合は “波及停止” ではなく “局所隔離（quarantine）” を基本
- reconnect storm は抑制される（全接続同時再接続を避ける）

### 4.2 Resubscribe discipline（固定）
- 再購読は idempotent に設計される
- 二重購読が起きても、重複排除で吸収できる
- スナップショット同期が必要なチャネルは、再購読時に必ず同期フェーズを踏む

---

## 5. Ordering / Sequence / Snapshot（固定）
### 5.1 順序乱れの扱い（固定）
- “順序が乱れる” のは正常系として扱う（ネットワーク/再送のため）
- CanonicalEvent は必要なら sequence で整列できるが、できない場合は “不確実” として表現し隠さない

### 5.2 Snapshot + Delta（固定）
Orderbook系などは snapshot+delta の整合を保証する必要がある。
固定要件：
- snapshot の取得/適用と delta の適用順は明確に
- 破綻（gap/sequence skip）時は再同期へ
- 再同期中は “品質低下状態” を明示（Quarantine or Degraded）

---

## 6. Dedupe（固定）
### 6.1 原則
- 重複は正常
- dedupe は “落とす” のではなく “二重計上しない”
- dedupe key は determinism-friendly に（Platform Foundationの event_uid 原則に従う）

### 6.2 例（固定の考え方）
- trade は exchange trade_id を最優先
- 無い場合は (ts, price, size, side, payload_hash) のような安定キー

---

## 7. Backpressure / Load Shedding（固定）
### 7.1 原則（固定）
- “silent drop” 禁止
- バックプレッシャは明示され、Integrityに記録される
- 高負荷時の縮退は段階的に（Policyで閾値）

### 7.2 縮退の優先順位（固定概念）
- P0（最重要）を最優先維持
- P2 は縮退しやすい（ただし縮退は記録される）
- 取りこぼしは Integrity Report に反映される

---

## 8. Quarantine（隔離：固定）
### 8.1 目的
異常な stream を隔離し、全体へ波及させない。

### 8.2 Quarantine の状態（固定概念）
- `ACTIVE`：正常収集
- `DEGRADED`：収集継続だが品質低下（遅延/欠損増）
- `QUARANTINED`：隔離（自動再試行はするが “正常扱いしない”）
- `RECOVERING`：回復試行中（再同期/再購読）

### 8.3 Quarantine と crosscut Safety（固定連携）
- P0 stream が policy threshold 以上 QUARANTINED → Safety Mode を SAFE へ（最小）
- Quarantine の出入りは必ず監査イベント（audit_event）に残す

---

## 9. Integrity / Gate inputs（固定）
Collector は Integrity と Gate の入力を生成する責務がある。

### 9.1 最低限の integrity signals（固定）
- expected vs observed subscriptions
- message rate / lag
- gaps（sequence gap / time gap）
- quarantine counts and durations
- persist backlog（Raw-firstの遅延）
- observability missing intervals（監視欠損）

### 9.2 監視欠損の扱い（固定）
監視欠損は “健康” ではなく “不明”。
- gate_results は UNKNOWN を取り得る
- crosscutにより SAFE へ倒す

---

## 10. Replay pointers（固定）
Collector は replay のために “どの範囲の入力が使われたか” を指し示せること。

最低限：
- RawFrame の保存範囲（キー/時刻/パーティション）
- config snapshot（SSOT/plan hash）
- 生成物（integrity_report 等）への参照

replay_pointers.schema.json に適合し、audit_event から参照される。

---

## 11. Audit（固定）
最低限、以下を audit_event として残す：
- run.start / run.end（startup_report参照）
- subscription.expected_loaded（SSOT hash）
- subscription.observed_changed（差分）
- reconnect.attempt / success / failure（原因）
- resync.start / success / failure
- quarantine.enter / exit（stream_id, reason）
- integrity.record（integrity_report_ref）
- gate.record（gate_results_ref）

秘密値は絶対に含めない。

---

## 12. テスト/検証観点（DoD）
最低限これが検証できること：

1) reconnect storm を抑制し、局所隔離できる
2) snapshot+delta の破綻で確実に再同期へ移行する
3) silent drop が起きず、欠損は integrity に必ず出る
4) quarantine の入退が audit_event に残る
5) replay_pointers が生成され、再現の入力範囲が追跡できる
6) 監視欠損で gate が UNKNOWN になり SAFE に寄る（crosscut）

---

## 13. Policy/Runbookへ逃がす点（明確な分離）
- backoff/jitter、subscribe上限、P0/P1/P2分類、欠損許容、保持期間
- quarantine解除条件、再同期手順、障害時運用
→ Policy/Runbookに置く（意味は変えない）

---
End of document
