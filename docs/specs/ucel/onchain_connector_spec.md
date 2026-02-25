# UCEL On-chain Connector Spec v1.0（固定仕様）
Unified On-chain / DEX Connector（Q）

- Document ID: UCEL-ONCHAIN-CONNECTOR-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): Q（On-chain Trading / Arbitrage）
- Depends-on: UCEL-SDK-CORE-SPEC, `docs/specs/crosscut/safety_interlock_spec.md`（実行する場合）
- Contracts SSOT:
  - 監査：`docs/contracts/audit_event.schema.json`
  - リプレイ参照：`docs/contracts/replay_pointers.schema.json`
- Goal:
  - マルチチェーン/RPC/DEX差分を吸収し
  - **finality/reorg前提**の安全な観測と（必要なら）取引実行を統一契約で提供する
- Non-goals:
  - 裁定判断や戦略ロジック（M/J）
  - 数値（gas上限、許容reorg深さ等）→ Policy

---

## 0. 不変原則（Non-negotiable）
1. **Finality-aware**：確定前提。reorgを"常識"として扱う。  
2. **Key isolation**：署名鍵は用途分離し、強操作は統制（B/E/J連携）。  
3. **MEV-aware**：フロントラン等を検知・抑制できる設計。  
4. **Deterministic identifiers**：tx_hash/log_index等でevent_uidを決定的にする。  
5. **Safety first**：危険時は縮退（HALT/SAFE_MODE）し資金を守る。

---

## 1. On-chain観測イベント（固定：必須概念）
共通ヘッダ（UCEL SDK準拠）：
- event_uid/trace_id/run_id/adapter_id/schema_version
- event_time（block time）/ recv_time / emit_time
- kind：block/tx/log/swap/pool_state/oracle_price 等

必須メタ（固定）：
- chain_id
- block_number
- block_hash
- tx_hash（該当時）
- log_index（該当時）
- finality_status：pending/confirmed/finalized/reorged
- supersedes（reorg時に置換される参照）

---

## 2. Finality / Reorg 契約（穴を塞ぐ）
- reorg は必ず起き得るため、イベントは finality_status を持つ
- reorged 時：
  - supersedes を保持し履歴を消さない
  - 監査イベントを発火できる
- "確定扱い"は finalized のみ（confirmedは暫定）

---

## 3. RPC冗長化と整合（固定）
- RPCは冗長化可能な設計（具体はPolicy）
- 不一致（同block_numberでhashが異なる等）は重大として隔離/監査
- 観測欠損はC（obs）へ出す

---

## 4. DEXルーティング/MEV対策の責務境界（固定）
adapter責務：
- 観測と実行の統一I/Fを提供（差分吸収）
- MEV兆候（異常price impact等）をイベント化可能な設計

上位（Q/M/J）責務：
- ルーティング最適化、MEV対策戦略、裁定判断

---

## 5. 実行（オンチェーン取引：固定の安全要件）
実弾を行う場合のみ（観測-onlyでも矛盾しない）。

- 署名鍵の用途分離（read/trade/withdraw相当）
- intent-first（実行も意思のSSOTを持てる設計）
- gas/slippage/route上限はPolicyで制御
- 失敗/遅延/置換（replace-by-fee）を前提に状態を管理
- 危険時（reorg多発/RPC不一致/MEV兆候）は Safety 連動（HALT等）

---

## 6. Observability（固定カテゴリ）
- rpc_calls_total / rpc_fail_total（category別）
- blocks_observed_total / reorg_detected_total
- finality_lag_seconds
- tx_submitted_total / tx_confirmed_total / tx_failed_total
- mev_alerts_total（兆候カテゴリ別）
- end_to_end_latency（block→event化）

---

## 7. Audit / Replay（固定）
- reorg置換関係を含めappend-onlyで追える
- 同一入力→同一event_uid（tx_hash/log_index基準）が成立する

---

## 8. Versioning（SemVer）
- MAJOR：finality/reorg契約の破壊、必須メタ変更
- MINOR：kind追加、イベント拡張
- PATCH：表現修正
