# DeFi / DEX Analytics & Arbitrage Readiness Core Spec v1.0（固定仕様）
Pool state truth / Quote quality / Route modeling / MEV-aware readiness / Evidence replay

- Document ID: DEFI-DEX-ARBI-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): N（DeFi / DEX / Arbitrage）
- Depends-on（Fixed）:
  - On-chain Ingestion: `docs/specs/onchain/onchain_ingestion_finality_reorg_integrity_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Risk/Portfolio: `docs/specs/risk/portfolio_risk_management_spec.md`
  - Execution Safety（オンチェーン実行に接続する場合の境界）: `docs/specs/execution/runtime_execution_safety_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/safety_state.schema.json`
- Policy separation（固定しない）:
  - スリッページ上限、ガス上限、MEV対策の運用値、ルート探索深さ、優先DEX、保持期間 → `docs/policy/**`
  - 事故対応/停止/復旧/アラート手順 → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
DEX/DeFiは「状態（pool/reserve/tick/liquidity）」が価格を決め、かつオンチェーン確定性（finality/reorg）とMEVに支配される。
本仕様は、DEX分析・アービ準備を **真実を隠さず、再現・証明可能**にし、誤判断で損失を出さない土台を固定する。

必達要件（固定）：
1) **Pool state is truth**：価格ではなく “状態” を正本として扱う（replay可能）
2) **Finality-aware**：未確定状態を確定扱いしない（onchain spec準拠）
3) **Quote has quality**：見積（quote）は quality（OK/DEGRADED/UNKNOWN）を持ち、UNKNOWNを利益として扱わない
4) **Route modeling**：ルート探索は “入力→出力→前提” を固定的に記録し再現できる
5) **MEV-aware readiness**：MEV/サンドイッチ/リオルグ等のリスクを固定モデルで見積もり、運用値で調整する
6) **No silent loss**：欠損/不一致/監視欠損は integrity_report に出す
7) **Safety coupling**：UNKNOWN/FAILは SAFE 側へ（crosscut）
8) **No secrets**：秘密は secret_ref のみ（署名鍵などは絶対に露出しない）

---

## 1. 範囲（in / out）
### 1.1 In
- DEXプールの状態追跡（v2/v3系の差異を吸収）
- ルーティング/見積（quote）生成
- アービ機会の候補抽出（機会そのものは“候補”であり実行は別境界）
- ガス/手数料/スリッページ/価格影響の見積
- MEVリスクの枠組み（運用値はPolicy）
- 最終性/リオルグの扱い（訂正・restate）
- replay pointers と監査イベント

### 1.2 Out
- 実際のオンチェーン発注（送信）は execution safety / control plane を経由
- 特定プロトコルの細部最適化（別ドキュメントで拡張可）
- UI/可視化

---

## 2. Core Concepts（固定）
### 2.1 DEX Identity（固定）
- `chain_id`
- `dex`（uniswap_v2, uniswap_v3, sushi, curve 等）
- `pool_id`（pairアドレス/プールアドレス）
- `token0` / `token1`（アドレス）
- `fee_tier`（該当する場合）
- `stream_id`（上記を安定に連結）

### 2.2 Pool State（固定：正本）
Pool state は最低限以下を含む（dex型により一部欠けても良いが欠けはqualityに反映）：
- `block_ref`（height/hash/finality）
- v2系：`reserve0`, `reserve1`, `k`（導出可）
- v3系：`sqrtPriceX96`, `tick`, `liquidity`, （可能なら）tick bitmap / tick data range
- `timestamp_utc`
- `quality`（OK/DEGRADED/UNKNOWN）
- `source_refs`（ログ/ストレージ参照：replay可能）

固定ルール：
- pool state は **価格より上位**の真実
- 価格は pool state から導出される派生値で、qualityを継承する

### 2.3 Quote（固定：見積）
Quote は “入力→出力の見積” で、最低限：
- `quote_id`（一意）
- `route`（後述）
- `amount_in` / `token_in`
- `amount_out_est` / `token_out`
- `assumptions`（ガス、fee、slippage、finality等）
- `quality`（OK/DEGRADED/UNKNOWN）
- `evidence_refs`（pool states / block refs / replay pointers）

---

## 3. Finality / Reorg（固定）
### 3.1 Finality coupling（固定）
- pool state は onchain spec の finality state（UNCONFIRMED/CONFIRMED/FINAL）を継承する
- 未確定（UNCONFIRMED）の quote は、原則として “実行候補” から除外、もしくは quality=DEGRADED/UNKNOWN とする（運用値はPolicyだが意味は固定）

### 3.2 Reorg handling（固定）
- reorg により block_hash が変わる場合：
  - affected pool states は restate される（上書きしない）
  - quote の根拠がreorg影響下なら “invalidated” として扱える（quality低下）
- reorg の影響範囲と再計算が audit_event と integrity に残る

---

## 4. Pool State Ingestion（固定）
### 4.1 Sources（固定）
pool state の更新は少なくとも以下から生成できる：
- onchain logs（Swap/Mint/Burn/Sync等）
- onchain calls（slot0/reserves 等、ただしRPC不一致に注意）
- indexer（使用する場合は capability と integrity を明示）

### 4.2 Multi-source integrity（固定）
- 同一blockで log と call の結果が矛盾したら UNKNOWN として扱い、隠さない
- RPC不一致がある場合は onchain spec の UNKNOWN と連動

---

## 5. Routing Model（固定）
### 5.1 Route（固定）
ルートは “hopの列” として表現できる：
- hop: `(dex, pool_id, token_in, token_out, fee_tier?)`
- `max_hops`（Policy）
- `constraints`（許可DEX、許可token、禁止path等：Policy）

### 5.2 Route evaluation（固定）
route の評価は必ず根拠を持つ：
- 使用した pool states（pool_id + block_ref）
- 使用したガス/feeモデル（policy snapshot ref）
- 推定slippage / price impact
- 失敗可能性（call revert 等）

---

## 6. Quote Quality（固定：重要）
Quote は必ず quality を持ち、意思決定で使う前提を固定する。

### 6.1 Quality levels（固定）
- `OK`：必要なpool stateが揃い、finality条件も満たす
- `DEGRADED`：一部不確実（未確定/データ欠け/遅延）だが参考として保持
- `UNKNOWN`：根拠が不明/矛盾/監視欠損/大規模reorg中 等

固定ルール：
- UNKNOWN を “利益” として扱わない（アービ候補として採用しない）
- DEGRADE/UNKNOWN は表示・監査・integrityに反映し、隠さない

---

## 7. MEV-aware readiness（固定枠組み）
MEV対策の詳細値はPolicyだが、枠組みは固定する。

### 7.1 MEV risk factors（固定カテゴリ）
- sandwich risk（価格影響が大きいルート）
- backrun risk（競合に奪われる）
- private tx / relay dependency（利用するなら）  
- latency risk（観測→署名→送信→包含までの遅延）
- reorg risk（未確定含む）

### 7.2 MEV risk output（固定）
Quote/Opportunity は少なくとも以下を出せる：
- `mev_risk_level`（LOW/MED/HIGH/UNKNOWN）
- `assumptions`（relay使用、max fee、deadline等）
- `quality` と整合（UNKNOWNなら riskもUNKNOWN寄り）

---

## 8. Arbitrage Opportunity（固定：候補モデル）
Opportunity は “候補” であり、実行は別境界。

最低限：
- `opp_id`
- `routes[]`（単一 or 複合）
- `profit_est`（評価通貨はPolicy）
- `cost_est`（gas/fee）
- `net_est`（profit-cost）
- `quality`
- `evidence_refs`（pool states/blocks/policy snapshot）
- `constraints`（最大サイズ、期限等）

固定ルール：
- quality=UNKNOWN は候補として採用しない
- net_est がプラスでも、finality/MEV/qualityが満たされないなら採用しない（運用値で調整できるが意味は固定）

---

## 9. Safety / Execution Boundary（固定）
### 9.1 実行への接続（固定）
Opportunity は Execution Safety（pre-trade gate）に “intent” として提案される。
- 実際のオンチェーン送信は control plane + dangerous op を経由し得る
- SAFE/EMERGENCY_STOP では実行へ流れない（crosscut）

### 9.2 Risk coupling（固定）
- portfolio/risk の上限を超える提案は禁止（pre-trade gateで拒否）
- quote/opportunityがUNKNOWNなら新規増は禁止（CLOSE_ONLY相当）

---

## 10. Integrity / Replay（固定）
### 10.1 Integrity signals（固定）
- pool state lag（headとの差）
- missing logs/blocks（欠損）
- rpc disagreement rate（onchain由来）
- reorg count/depth（影響範囲）
- quote quality distribution（OK/DEGRADED/UNKNOWN）
- opportunity invalidation（reorg/price moveで無効化された数）

これらを integrity_report の根拠にする。

### 10.2 replay pointers（固定）
最低限以下を参照できる：
- chain/network + block range
- pool_id set（対象集合）
- onchain raw logs range（object keys/partitions）
- route evaluation inputs（policy snapshot refs）
- outputs（quotes/opportunities）

---

## 11. Audit（固定）
最低限、以下を audit_event として残す（秘密値なし）：
- `defi.pool_state.updated`（pool_id + block_ref + quality）
- `defi.quote.generated`（quote_id + route summary + quality + refs）
- `defi.quote.invalidated`（reorg/price move/ttl）
- `defi.opportunity.detected`（opp_id + net_est + quality）
- `defi.opportunity.rejected`（reason_codes：quality/mev/finality/risk）
- `defi.quarantine.enter/exit`（pool/chain scope）
- `integrity.record` / `gate.record`

---

## 12. テスト/検証観点（DoD）
最低限これが検証できること：

1) pool state が正本として保存され、価格は派生である
2) UNCONFIRMED を確定扱いしない（quality/finalityで表現）
3) reorg で restate が発生し、上書きされない
4) quote/opportunity が必ず quality を持ち、UNKNOWNは採用されない
5) multi-source/RPC不一致がUNKNOWNとして表面化し隠蔽されない
6) replay_pointers が block range と pool集合を指し、再現可能
7) SAFE/EMERGENCY_STOP で実行へ流れない

---

## 13. Policy/Runbookへ逃がす点
- slippage/gas/fee上限、ルート探索深さ、優先DEX、MEV対策運用値
- 停止/復旧、quarantine解除、緊急時運用
→ Policy/Runbookへ（意味は変えない）

---
End of document
