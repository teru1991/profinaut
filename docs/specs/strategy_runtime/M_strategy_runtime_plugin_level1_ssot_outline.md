# M: Strategy Runtime / Plugin（戦略実行基盤） — Level 1 SSOT Outline

**Source:** M.txt（Core Spec v1.1 / SSOT）  
**Domain Letter:** M  
**Status:** 最終版：実装目標機能詳細設計（Core Spec v1.1 / SSOT）

---

## 0. 目的と非交渉要件（Non-negotiable）

### 0.1 目的
- M は、戦略（Strategy）を **安全・堅牢・高速・再現可能** に実行するための「戦略実行OS」
- 戦略の増加・更新・失敗・入力の乱れを前提として、標準機能で保証する

### 0.2 必達（最低条件）
1. 戦略は **プラグイン**として追加できる（Rust / Python）
2. **サンドボックス**（資源制限・権限・外部I/O制御・出口ゲート）を備える
3. **State 永続化**と **再起動復元**を保証（破損耐性含む）
4. **Low-Latency（LL）**と Compute を分離し、相互干渉しない
5. M の唯一の出口は **Intent（意図）**。実注文確定は Execution（I）が行う
6. **二重起動・二重発注**を仕組みで封じる（lease / idempotency）
7. 事故解析・回帰検証のため、**決定論の前提（seed/time/契約）**を壊さない

---

## 1. 責務境界（Mがやる / やらない）

### 1.1 Mの責務（提供するもの）
- Strategy Host（起動/停止/再起動/更新/影武者）
- Plugin ABI（イベント入力 → Intent出力）
- Runtime Scheduler（LL/Compute、QoS、backpressure）
- Sandbox / Policy Enforcer（資源・権限・外部I/O・出力上限）
- State Manager（checkpoint/restore/migration/破損耐性）
- Circuit Breaker（戦略単位停止/縮退/復帰）
- Admission Control（起動前審査：契約・権限・資源・互換性）

### 1.2 Mが直接やらない（他ドメインの責務）
- 実発注・再送・約定照合・取引所方言の執行：**I（Execution）**
- 市場データ収集・正規化・一次品質判定：**H（Market Data）**
- 最終リスク判断（最大建玉/停止判定）：**J（Risk Gate）**
- 全体緊急停止・最終縮退統制：**E（Safety Controller）**
- 長期監査・決定論リプレイの保管/署名台帳：**O（Audit/Replay）**
- backtest/forward/paper 検証基盤：**N（Backtest/Forward）**
- 供給網防御（署名運用/ビルド証跡）：**U（Supply Chain）**
- 時刻規律・時刻意味の固定：**F（Clock/Time Semantics）**
- 診断バンドル/運用可観測性統合：**Y（Diagnostics）**

### 1.3 境界原則（要約）
- M は「戦略を安全に動かす箱」。出口は Intent のみ。実注文/最終停止は I/J/E が握る

---

## 2. Canonical Data Model / Contracts（固定）

### 2.1 Strategy Descriptor（戦略メタ定義）
必須フィールド：
- strategy_id, strategy_version（SemVer）
- runtime_kind: rust_wasm / rust_native / python
- instance_model: single / sharded
- capabilities（購読範囲、注文種別、対象 venue/asset、外部呼出し可否 等）
- resources（CPU/メモリ/スレッド/timeout/キュー上限/出力上限）
- state_schema_version
- config_schema（型・範囲・単位・必須・デフォルト）
- io_contracts（入力イベント / Intent 契約バージョン）
- binary_hash（署名/provenance は U 制度だが、M は検証できる形で保持）

### 2.2 Strategy Instance（インスタンスモデル）
- instance_id（例：venue/market/symbol shard）
- インスタンス単位で config/state/checkpoint/health を分離
- health_state: healthy / degraded / halted + reason_code

### 2.3 Plugin ABI（固定コールバック）
コールバック：
- on_init(ctx, config, restored_state)
- on_event(ctx, event)
- on_timer(ctx, timer_id, ts)
- on_shutdown(ctx, reason)
- on_healthcheck(ctx)（推奨）

返り値（分離扱い）：
- intents[]
- state_updates（checkpoint 要求含む）
- telemetry_hints（短いタグ/スコア等、**自由文は禁止**）

### 2.4 Intent（唯一の出口：固定形）
- OrderIntent / CancelIntent / AmendIntent / PositionIntent
- AlertIntent / NoOp
- RiskHint（最終判断は J/E）

共通必須メタ（欠けたら破棄）：
- bot_id, strategy_id, strategy_version, instance_id
- run_id, trace_id
- idempotency_key
- risk_context（最大許容スリッページ、許容遅延、想定保有時間、予算 等）

---

## 3. 実行レーン（LL / Compute）とスケジューリング（固定）

### 3.1 Lanes
- LL lane：短 deadline（ミリ秒オーダ）、市場イベントは最新優先（間引き/集約可）
- Compute lane：特徴量生成、最適化、推論、外部モデル問い合わせ。結果は遅延し得る→stale 判定で捨てる

### 3.2 ルール（非交渉）
- LL は Compute 滞留で止まらない（完全分離）
- LLM/外部推論は LL 禁止（Compute のみ）
- Compute 詰まり時の標準縮退：NoOp / 簡易ロジック / 取引停止

### 3.3 QoS / SLO（運用レベル）
- 戦略/インスタンスごとに SLO（p95 遅延、許容ドロップ率、最大 CPU 秒）
- SLO 違反時：優先度調整 → degrade → halt（段階制御）

---

## 4. 入力イベントと Backpressure（固定）

### 4.1 入力イベント種別（最低限）
- MarketEvent（H）
- ExecutionEvent（I）
- Risk/SafetyEvent（J/E）
- TimerEvent（M）
- ManualSignal（運用操作）

### 4.2 Backpressure ポリシー（種別ごと固定）
- Market：最新優先（古いものを捨てる/集約）
- Execution：落とさない（順序保証）
- Risk/Safety：最優先（割り込み）
- キュー上限超過 → degraded（理由コード付き）＋縮退連動

---

## 5. サンドボックス（暴走・漏洩・過負荷対策：固定）

### 5.1 隔離レベル（推奨）
- L0：スレッド＋watchdog
- L1：別プロセス＋OS制限
- L2：Wasm（WASI）（Rust 推奨標準）
- L3：コンテナ隔離（Python 推奨標準）

### 5.2 必須制限（二重化推奨）
- 1イベント deadline（timeout）
- CPU/メモリ/スレッド上限
- 外部 I/O 原則禁止（許可リスト）
- ログ量上限（秒/実行）
- Intent 量上限（秒/イベント/インスタンス）
- 例外/パニックは戦略単位で隔離（ホスト巻き添え禁止）

### 5.3 違反時の段階対応（固定）
- warn → degrade → halt
- すべて reason_code を発行し、運用側で即行動可能にする

---

## 6. State（永続化・復元・破損耐性：固定）

### 6.1 保存要件
- state_schema_version 必須
- checksum 必須
- last_event_cursor 必須（どこまで処理したか）
- checkpoint 方針：定期 / 重要イベント / 停止前強制 flush

### 6.2 破損耐性（非交渉）
- atomic swap（二重書き＋検証）
- 破損検知時：直前 checkpoint へ rollback または 安全停止＋証跡保存
- migration hook：互換不能なら起動拒否（admission で止める）

---

## 7. 二重起動防止（Split-Brain：固定）
- 同一 bot_id + strategy_id + instance_id の二重起動禁止
- lease ロック＋heartbeat
- フェイルオーバー時の奪取ルールを固定（安全に移譲）

---

## 8. 出口（Intent Gate）と二重発注防止（固定）

### 8.1 Intent Gate（Mの“最後の柵”）
- Intent 正規化（単位/丸め/精度/最小数量）
- Intent 上限（秒/イベント/インスタンス）
- 危険 Intent の自動縮退（reduce-only化、指値のみ化、サイズ縮小、NoTrade化）
- 必須メタ欠落は破棄（run/trace/idempotency/risk_context）

### 8.2 重複耐性（Exactly-once-ish）
- idempotency_key を必須化し、M→I 境界で保持
- emit 後クラッシュ→再起動でも二重発注しない前提を維持

### 8.3 段階確定（Intent Staging）
- staged → approved → committed をサポート（最終確定は I/J/E 側の協約に従う）

---

## 9. 戦略単位の Circuit Breaker（固定）

### 9.1 トリガ例
- timeout 連発、Intent 過剰、遅延悪化、入力品質劣化、拒否率/429 兆候 等

### 9.2 動作
- 縮退モード：NoTrade / ReduceOnly / CancelAll / HedgeToFlat
- 復帰：cooldown / 手動 / 自動（条件充足）
- すべて reason_code を発行

---

## 10. パラメータ更新（Config Hot-Reload：固定）
- 型付きスキーマ検証（範囲/単位/必須/デフォルト）
- 適用タイミング制御：次 tick / 次 bar / 次 checkpoint
- 変更監査メタ（誰が/いつ/何を）はイベントとして扱える形にする（永続ログは Y/O 側）

---

## 11. 入力品質（Data Quality Awareness：固定）
- stale / gap / dup / out_of_order の品質フラグを受け取れる
- 品質低下時の標準行動：取引停止、サイズ縮小、指値のみ、再同期待ち
- “時間の意味”は F で固定され、M は品質フラグとして処理

---

## 12. Admission Control（起動前審査：固定）
起動前に必ず：
- descriptor 整合（SemVer、capabilities、resources、契約バージョン）
- state_schema 互換性（migration 可否）
- 依存/ハッシュ一致（binary_hash、必要なら署名検証）
- 動的スモーク（疑似イベントで契約違反がないこと）

不合格は起動拒否（“動くけど危険”を排除）

---

## 13. 外部依存（モデル/LLM/シグナル：固定ポリシー）
- 原則：戦略から外部ネットワーク禁止
- 必要なら別サービス（Model/Signal）経由で問い合わせ（Compute のみ）
- LLM ポリシー：
  - LL 禁止（Compute のみ）
  - 自由文出力禁止（構造化のみ）
  - LLM 出力で直接 Intent 生成禁止（提案止まり＋ゲート通過必須）

---

## 14. 観測・運用（Mが提供する最小：固定）
M が少なくとも出せる（統合は Y 側）：
- strategy_event_latency_ms{strategy,instance,lane}
- strategy_queue_depth{strategy,instance,lane}
- strategy_timeout_total{strategy,instance}
- strategy_fault_total{strategy,instance,kind}
- intent_emitted_total{strategy,instance,type}
- state_checkpoint_bytes{strategy,instance}
- state_restore_time_ms{strategy,instance}
- health_state{strategy,instance} と reason_code

加えて、安全なライブ内省（サンプリング）を可能にする（秘密は絶対に出さない）

---

## 15. Mの外だが必須の境界仕様（前提として固定する連携）
M は以下の協約が存在する前提で設計され、M 側は破らない：
- A（共通基盤）：run_id/trace_id/schema_version 伝播、idempotency 生成規約
- F（時刻規律）：event/recv/processed の意味固定、ドリフトの品質表現
- H（Market Data）：品質フラグ共通表現、cursor の扱い
- I（Execution）：Intent→注文ライフサイクル分離、idempotency 最終適用、staging 確定点
- J/E（Risk/Safety）：縮退/停止の優先順位、RiskContext 必須項目
- N/O（検証/監査）：同一 ABI で backtest/forward/paper/live 貫通、決定論前提（seed/time）
- U（Supply Chain）：署名/provenance 制度、M は起動拒否できる形で検証可能
- Y（Diagnostics）：support bundle/可視化統合、M は reason_code と統計を提供

---

## 16. 完全実装（DoD：完成条件）
1. Rust（Wasm）と Python で戦略追加でき、同一 ABI で動く
2. timeout/資源/外部I/O/ログ/Intent 上限が機能し、違反で確実に止まる
3. state 復元が動き、破損時に rollback or 安全停止できる
4. LL/Compute 分離が成立し、Compute 滞留で LL が止まらない
5. lease/heartbeat で二重起動を防げる
6. idempotency で二重発注を防げる前提が M→I 境界まで維持される
7. Circuit Breaker で戦略単位の縮退/停止/復帰ができる
8. Hot-reload（パラメータ更新）が型検証つきで安全に適用できる
9. 入力品質低下（stale/gap 等）で標準縮退が動く
10. health_state / reason_code / 主要メトリクスが観測できる
11. admission control で危険な戦略を起動前に拒否できる

---

## 17. Capability Index（ID保持）
> 入力テキスト内に **A-xxx / Sxx / T-xx / F-xx / Y-xx** 等の個別ID記載は見当たりませんでした。  
> **TODO:** 既存の管理ID体系がある場合、各機能（例：Intent Gate、State、Admission Control 等）に対応する ID を割当・追記してください。

### 17.1 ドメイン参照（境界協約）
- A / F / H / I / J / E / N / O / U / Y（M が前提とする連携先）

---
