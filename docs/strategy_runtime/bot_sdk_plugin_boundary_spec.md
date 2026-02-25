# Strategy Runtime / Bot SDK / Plugin Boundary Core Spec v1.0（固定仕様）
Strategy plugin interface / Capability / Safe execution boundary

- Document ID: STRAT-SDK-PLUGIN-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): I（Strategy Runtime / Bot SDK）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Execution Safety: `docs/specs/execution/runtime_execution_safety_spec.md`
  - Market Data Collector: `docs/specs/market_data/collector_framework_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Research/Testing Repro: `docs/specs/research_testing/backtest_forwardtest_repro_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
- Policy separation（固定しない）:
  - 1秒あたりの意思決定上限、メモリ/CPU上限、risk limit、タイムアウト → `docs/policy/**`
  - 運用手順（配布/有効化/失効/審査） → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
戦略（bot/agent）は本質的に危険（暴走・誤発注・過負荷・不正確）である。
本仕様は、戦略を “安全な箱” に閉じ込め、以下を固定保証する：

1) **Strategy is sandboxed**：戦略は境界の外に副作用を持たない（勝手に発注できない）
2) **Contracts over freedom**：入力/出力は契約で固定し、暗黙動作を禁止
3) **Capability-driven**：戦略が何を必要とし、何を保証するか宣言させる
4) **Deterministic-by-input (at least evidence)**：同じ入力で同じ意思決定、または差分説明が残る
5) **Failure is safe**：タイムアウト/例外/不整合は “何もしない” へ縮退
6) **Auditability**：意思決定の要点と根拠参照が監査可能（秘密ゼロ）
7) **Performance isolation**：戦略の重さでシステムを壊さない（上限・隔離）

---

## 1. Strategy Runtime の責務境界
### 1.1 In（対象）
- Strategy Plugin Interface（入力/出力/ライフサイクル）
- Capability 宣言（必要データ/保証/制約）
- Decision loop（tick/event-driven）
- Risk intent の生成（注文意図の生成）
- Safe boundary（実発注は Execution Safety に委譲し、戦略は “intent提案” まで）
- Audit summary（決定要点・根拠参照）
- Replayability（入力参照・設定ハッシュ）

### 1.2 Out（対象外）
- 実際の注文送信（Execution Safetyが正本）
- Market dataの収集（Collectorが正本）
- ストレージ最適化（Storageが正本）

---

## 2. Plugin Interface（固定：概念契約）
### 2.1 Strategy Plugin のライフサイクル（固定）
- `init(ctx)`：初期化（capabilities宣言・設定検証）
- `on_start()`：開始
- `on_event(evt)`：イベント駆動（market/candle/fill等）
- `on_tick(now)`：時間駆動（一定周期）
- `on_stop(reason)`：停止（graceful）
- `health()`：自己診断（degraded/quarantined判定に利用）

固定ルール：
- plugin は外部へ直接副作用を起こさない（HTTP/WS/DB直叩き禁止）
- すべての外部アクセスは Runtime が提供する限定API経由（read-only中心）

### 2.2 Context（ctx）の固定要件
Runtime が plugin に与える ctx は最低限以下を含む：
- identity（actor/service）
- environment（dev/stage/prod、paper/shadow/live）
- safety_mode（NORMAL/SAFE/EMERGENCY_STOP）
- execution_killswitch（ALLOW/CLOSE_ONLY/FLATTEN/BLOCK）
- dataset/session refs（replay用）
- logger（redaction済み）
- time source（UTC、monotonic補助）

---

## 3. Capabilities（固定）
### 3.1 宣言（固定）
plugin は init 時に capabilities を宣言する：
- `plugin_id`（一意）
- `plugin_version`（SemVer）
- `required_features[]`（例：market.trades, market.orderbook.l2）
- `optional_features[]`
- `constraints`（例：max_symbols, max_orders_per_min, requires_candles）
- `determinism_level`（NONE / EVIDENCE / STRONG）
- `risk_profile`（requires_position, uses_leverage, maker_only等）

固定ルール：
- capabilities が欠損/不正なら起動拒否（SAFE側）
- runtime は capabilities に基づき “提供できない入力” を与えない（代わりに起動拒否/縮退）

---

## 4. Inputs（固定：入力契約）
### 4.1 入力の種類（固定）
- market events（trades, book, ticker, candles）
- execution feedback（fills, order status）
- portfolio snapshot（position/exposure）
- system signals（safety/gate/quarantine）

### 4.2 入力の品質表明（固定）
入力には品質メタを添付できること：
- `quality`（OK/DEGRADED/UNKNOWN）
- `gaps`（time gap / seq gap）
- `quarantine`（対象streamの隔離状態）

固定ルール：
- quality が UNKNOWN の入力で “攻めた判断” をしてはいけない（pluginは縮退する）
- runtime は quality UNKNOWN が増えた場合、plugin を QUARANTINED にできる

---

## 5. Outputs（固定：出力契約）
plugin の出力は “intent” のみ（副作用は持たない）。

### 5.1 Intent types（固定）
- `NOOP`（何もしない）
- `ORDER_INTENT`（place/amend/cancel/close/flatten の提案）
- `RISK_ADJUST_INTENT`（kill-switch tightenの提案など）
- `ANNOTATION`（説明/メタ：監査・可視化用）

### 5.2 Intent safety（固定）
- plugin は “live発注を確定” できない。あくまで提案。
- 実行可否は Execution Safety の pre-trade gate が決める。

固定ルール：
- safety_mode SAFE/EMERGENCY_STOP のとき plugin は NOOP を基本とし、例外的に “risk reducing only” を提案できる
- killswitch CLOSE_ONLY/FLATTEN/BLOCK を侵害する intent は runtime が破棄する

---

## 6. Determinism / Replay（固定）
### 6.1 Deterministic-by-input（最低保証）
同じ入力イベント列（または replay pointersで固定した入力範囲）と同じ config で、
同じ決定を返すか、差分説明を残す。

- `determinism_level=EVIDENCE` を最低推奨
- STRONG は将来拡張

### 6.2 入力参照（固定）
runtime は decision ごとに：
- input refs（window, streams）
- config_hash
- plugin_version
を記録し、audit_event.details と replay_pointers に結びつける。

---

## 7. Timeouts / Resource isolation（固定）
- plugin の意思決定にはタイムアウトがある（値はPolicy）
- タイムアウト時は：
  - decision は NOOP とする
  - audit_event に timeout を記録
- 過負荷（CPU/メモリ）が検出された場合：
  - plugin を PAUSED/QUARANTINED にし、全体へ波及させない

---

## 8. Quarantine（固定）
runtime は plugin を隔離できる：
- 連続例外
- 長時間タイムアウト
- 不整合（禁止intent多発）
- quality UNKNOWN の増加時に危険判断を続ける

隔離は audit_event に残し、手動解除は dangerous op（control plane）扱い。

---

## 9. Audit（固定）
最低限、以下を audit_event に残す：
- `strategy.plugin.loaded`（plugin_id/version/capabilities）
- `strategy.decision.made`（要約：intent type、根拠refs）
- `strategy.decision.noop`（理由）
- `strategy.intent.rejected_by_runtime`（killswitch/safety違反）
- `strategy.quarantine.enter/exit`
- `strategy.timeout`
秘密値は含めない。

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) plugin が外部副作用を起こせない（intent提案のみ）
2) SAFE/EMERGENCY_STOP で攻めた intent が通らない（runtimeが破棄/NOOP）
3) killswitch を侵害する intent が実行に流れない
4) タイムアウトで NOOP に縮退し、監査が残る
5) 同じ入力で同じ決定、または差分説明が残る
6) quarantine が機能し、手動解除が dangerous op 扱い

---

## 11. Policy/Runbookへ逃がす点
- タイムアウト、CPU/メモリ上限、意思決定頻度上限
- risk limit、許容intent範囲
- 配布/審査/失効フロー
→ Policy/Runbookへ（意味は変えない）

---
End of document
