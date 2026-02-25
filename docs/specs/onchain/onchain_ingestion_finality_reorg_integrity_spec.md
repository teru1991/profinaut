# On-chain Ingestion / Finality / Reorg / RPC Integrity Core Spec v1.0（固定仕様）
Block/Tx/Log ingestion / Finality model / Reorg handling / Multi-RPC integrity / Replayability

- Document ID: ONCHAIN-INGEST-INTEGRITY-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): M（On-chain）
- Depends-on（Fixed）:
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/safety_state.schema.json`
- Policy separation（固定しない）:
  - confirmations（finality閾値）、RPC数、バックオフ、保持期間、監視閾値 → `docs/policy/**`
  - 障害対応/復旧/切替手順 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
オンチェーンは “確定したと思った事実が覆る（reorg）” ことがある。
本仕様は、ブロック/Tx/Log を **欠損・遅延・RPC不一致・reorg** を前提に取り込み、
それでも **真実を隠さず、再現・証明可能**にし、安全縮退を確実にする。

必達要件（固定）：
1) **Finality-aware ingestion**：finality（確定度）を明示し、未確定を確定として扱わない
2) **Reorg correctness**：reorg を検知し、影響範囲を明示し、訂正（restate）できる
3) **Multi-RPC integrity**：複数RPC/プロバイダで整合性検証し、不一致は UNKNOWN として表面化
4) **No silent loss**：欠損/不明区間は integrity_report で必ず表面化
5) **Replayable evidence**：replay_pointers により入力範囲（block range）を追跡できる
6) **Safety honesty**：RPC不一致/観測欠損は SAFE 側へ（crosscut準拠）
7) **Performance**：高負荷・長期同期でもバックプレッシャで破綻しない（縮退は記録される）
8) **No secrets**：APIキー等は secret_ref のみ（identity_access_spec準拠）

---

## 1. 範囲（in / out）
### 1.1 In
- ブロックヘッダ/ブロック本体/Tx/Receipt/Logs の取り込み
- Finality モデル（confirmed/unconfirmed の扱い）
- Reorg 検知・影響範囲・訂正
- Multi-RPC クロスチェック（不一致検出）
- Backfill（過去同期）と Realtime follow
- Quarantine（チェーン/ネットワーク/アドレス単位の隔離）
- Integrity signals / replay pointers / audit events

### 1.2 Out
- 特定チェーンの細部（ただし差分はPolicy/adapterで吸収）
- DeFi解析ロジック（将来の解析ドメイン）
- UI/可視化

---

## 2. Core Concepts（固定）
### 2.1 Chain Identity（固定）
オンチェーン収集対象は少なくとも以下で識別できる：
- `chain`（例：ethereum, arbitrum）
- `network`（mainnet/testnet）
- `chain_id`
- `rpc_set_id`（利用しているRPC集合の識別）
- `address_scope`（監視アドレス集合/フィルタの識別）
- `stream_id`（上記を安定に連結した識別子）

### 2.2 Data Classes（固定）
- `BlockHeader`
- `BlockBody`（Tx一覧）
- `Tx`
- `Receipt`
- `Log`（event）
- `Trace`（optional: internal calls。取得できない場合は capabilityで宣言）
- `ControlEvent`（backfill/reorg/retry/quarantine等）

### 2.3 Finality State（固定語彙）
finality は “確定度” として最低限以下を持つ：
- `UNCONFIRMED`：確定していない（reorgにより覆りうる）
- `CONFIRMED`：所定確認数を満たした（Policy値）
- `FINAL`：実質確定（Policy値。チェーンにより段階は調整可能）

固定ルール：
- `FINAL` を `UNCONFIRMED` として扱うことはあっても、その逆（未確定を確定扱い）はしない

### 2.4 Reorg（固定）
reorg は「同一 height で block_hash が変わる」等として観測される。
固定要件：
- reorg を検知したら影響範囲（from_height..to_height）を推定し明示する
- 影響範囲のデータは “訂正（restate）” される（上書きしない）
- 影響範囲は integrity_report と audit_event に必ず残す

---

## 3. Ingestion Modes（固定）
### 3.1 Backfill（過去同期）
- 指定した block range を埋める
- missing を検知し、再試行する
- 完了/未完了を明示（unknownを残さない）

### 3.2 Follow（リアルタイム追従）
- head を追従し、UNCONFIRMED を取り込む
- confirmed/final は “確認数が揃った時点で昇格” させる（訂正イベントとして）

### 3.3 Hybrid（固定）
Backfill と Follow は同時に動きうるが、競合しても dedupe で吸収できる設計が必須。

---

## 4. Multi-RPC Integrity（固定）
### 4.1 原則（固定）
単一RPCは嘘をつく（停止/遅延/欠落/誤返答の可能性）。
よって、少なくとも以下を固定要求とする：
- 複数RPC（または複数データソース）で “同一heightの block_hash” を比較できる
- 不一致は “UNKNOWN/DEGRADED” として表面化（隠さない）

### 4.2 Cross-check strategy（固定枠組み）
以下のいずれか（または併用）を実現できる：
- Header hash cross-check（最小）
- Receipts root / tx count cross-check（可能なら）
- sample log cross-check（部分検証）

固定ルール：
- 不一致が一定以上発生したら対象 stream は QUARANTINED にできる
- RPC欠損（timeouts）も “unknown” として integrity に反映

---

## 5. Dedupe / Idempotency（固定）
### 5.1 原則（固定）
- 同じ事実（同じ block_hash/tx_hash/log_index）は二重計上しない
- 再試行/再取得/別RPC取得は正常系であり吸収する

### 5.2 Dedupe keys（固定）
- Block: `(chain_id, height, block_hash)`
- Tx: `(chain_id, tx_hash)`
- Log: `(chain_id, tx_hash, log_index)`（または `(block_hash, tx_index, log_index)`）

reorgで block_hash が変わる場合：
- “古いblock_hash系の事実” は restated として扱う（上書きしない）

---

## 6. Reorg Handling（固定：訂正モデル）
### 6.1 訂正（restate）原則（固定）
- 過去データの上書きは禁止（append-only）
- 訂正は “reorg event” として追加し、影響範囲と理由を記録
- 下流（派生/集計）は「どのcanonical chain」を採用したかを明示できる

### 6.2 Chain selection（固定概念）
- canonical chain は “最終的に採用される系列” として定義される
- selection の根拠（RPC多数決/信頼度/最終性）を audit に残せること

---

## 7. Quarantine（固定）
### 7.1 対象
- chain/network 単位
- rpc_set 単位
- address_scope 単位（異常なフィルタや過負荷）

### 7.2 状態（固定概念）
- `ACTIVE`
- `DEGRADED`
- `QUARANTINED`
- `RECOVERING`

### 7.3 Safety coupling（固定）
- P0相当のオンチェーン入力が QUARANTINED/UNKNOWN が閾値超 → Safety Mode を SAFE へ（最低限）
- QUARANTINED の解除は原則手動（dangerous op）にできる

---

## 8. Integrity / Gate inputs（固定）
オンチェーン収集は最低限、以下を integrity signals として出せる：

- head lag（latest headとの差）
- missing block intervals（欠損）
- rpc disagreement rate（不一致率）
- reorg count / max reorg depth
- confirmations progression（UNCONFIRMED→CONFIRMED→FINAL の遷移遅延）
- persist backlog（storage遅延）
- observability missing intervals（監視欠損）

これらは integrity_report と gate_results の根拠。

固定ルール：
- 監視欠損は UNKNOWN として扱う（健康扱い禁止）
- UNKNOWN は crosscut safety により SAFE 側へ

---

## 9. Replay pointers（固定）
replay_pointers は少なくとも以下を参照できる：
- chain/network/rpc_set_id
- block range（start_height..end_height）
- address_scope（対象フィルタ）
- raw保存範囲（object keys/partitions）
- restatement（reorg）イベント参照
- outputs（integrity_report等）

---

## 10. Audit（固定）
最低限、以下を audit_event として残す（秘密値なし）：
- `onchain.run.start/end`（startup_report参照）
- `onchain.backfill.start/end`（range）
- `onchain.follow.head.advance`（height/hash）
- `onchain.rpc.disagreement`（rate + scope）
- `onchain.reorg.detected`（range + depth + evidence refs）
- `onchain.reorg.restate.applied`（影響範囲）
- `onchain.quarantine.enter/exit`
- `integrity.record`（integrity_report_ref）
- `gate.record`（gate_results_ref）

---

## 11. Performance / Backpressure（固定）
- 高負荷時は backpressure をかける（silent drop禁止）
- 重要度（P0/P1/P2）はPolicyだが、縮退した事実は必ず integrity_report に残す
- RPCのリトライ storm を抑制（backoff + jitter：値はPolicy）

---

## 12. テスト/検証観点（DoD）
最低限これが検証できること：

1) finality（UNCONFIRMED/CONFIRMED/FINAL）が明示され、未確定を確定扱いしない
2) reorg を検知し、影響範囲を明示し、訂正イベントとして残す（上書きしない）
3) multi-RPC 不一致が UNKNOWN/DEGRADED として表面化し、隠蔽しない
4) 欠損が integrity_report に必ず出る（silent loss無し）
5) replay_pointers が block range を指し、再現が可能
6) 監視欠損で gate UNKNOWN → safety SAFE 側へ寄る（crosscut）

---

## 13. Policy/Runbookへ逃がす点
- confirmations閾値、RPC数/重み、reorg深度の危険判定、保持期間
- 障害時切替、復旧手順、再同期手順
→ Policy/Runbookへ（意味は変えない）

---
End of document
