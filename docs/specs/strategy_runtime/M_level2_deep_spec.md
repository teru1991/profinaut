# M — Level 2 Deep Spec（整理のみ / 追加仕様なし）

> 本入力は **Non-negotiable（0）**、**Canonical Model/Contract（2, 15）**、**Behavior/Tests相当（16, 各“固定ルール”）** が揃っているため、Level 2 を併記します（新規仕様は作らず、構造化のみ）。

## 1. Invariants（非交渉の不変条件）
- 出口は Intent のみ（実注文確定は I）
- LL と Compute は完全分離（Compute 滞留で LL は止まらない）
- State 永続化と復元（破損耐性含む）
- 二重起動（lease/heartbeat）と二重発注（idempotency）を封じる
- 決定論前提（seed/time/契約）を壊さない

## 2. Contracts（固定契約のチェック対象）

### 2.1 Descriptor Contract
- 必須フィールドの欠落 → **起動拒否**（Admission）
- binary_hash は **保持**し、検証可能な形で扱う（署名制度は U）
- TODO: descriptor の表現形式（JSON/YAML/CBOR等）／署名検証手順の受け口（M側のI/F）

### 2.2 Plugin ABI Contract
- callbacks と返り値（intents/state_updates/telemetry_hints）を固定
- telemetry_hints は自由文禁止（短いタグ/スコア等）
- TODO: 例外時（panic/例外）に ABI 上どのエラー表現を返すか（reason_code への対応）

### 2.3 Intent Contract
- 共通必須メタ欠落は **破棄**（Intent Gate）
- idempotency_key 必須で M→I 境界で保持
- TODO: Intent 正規化の具体（単位/丸め/最小数量）の参照元（どこが正）

## 3. Runtime Lanes & Scheduling（挙動整理）

### 3.1 LL lane
- 短 deadline（ミリ秒）
- MarketEvent は最新優先（間引き/集約可）

### 3.2 Compute lane
- 特徴量生成・最適化・推論・外部モデル問い合わせ
- 結果は stale 判定で捨てる

### 3.3 Degradation Ladder
- SLO 違反 → 優先度調整 → degrade → halt
- Compute 詰まり標準縮退：NoOp / 簡易ロジック / 取引停止

## 4. Backpressure（種別別の必達挙動）
- Market：最新優先で捨て/集約
- Execution：落とさない・順序保証
- Risk/Safety：最優先割り込み
- キュー上限超過：degraded＋理由コード＋縮退連動

## 5. Sandbox / Policy Enforcer（必須制限と違反時挙動）

### 5.1 Isolation Levels
- L0/L1/L2(WASI)/L3(container) を推奨として整理

### 5.2 Limits（必須）
- timeout / CPU / memory / threads / external I/O allowlist / log rate / intent rate

### 5.3 Enforcement
- warn → degrade → halt（reason_code を必ず発行）
- TODO: reason_code のコード体系一覧（表）

## 6. State Manager（復元・破損耐性・移行）
- 保存必須：state_schema_version / checksum / last_event_cursor
- checkpoint：定期/重要イベント/停止前強制 flush
- 破損耐性：atomic swap（二重書き＋検証）
- 破損時：rollback or 安全停止＋証跡保存
- migration：互換不能は起動拒否（Admission）
- TODO: State storage の配置先（ローカル/分散/KV等）と整合性境界

## 7. Split-Brain & Idempotency（Exactly-once-ishの整理）
- 二重起動禁止：bot_id+strategy_id+instance_id、lease+heartbeat
- Intent 二重発注防止：idempotency_key 必須、M→I 境界で保持
- TODO: フェイルオーバー奪取ルールの具体（タイムアウト閾値等は未記載）

## 8. Intent Gate（最後の柵）
- 正規化（単位/丸め/精度/最小数量）
- 量上限（秒/イベント/インスタンス）
- 危険 Intent 縮退（reduce-only、指値のみ、サイズ縮小、NoTrade）
- 必須メタ欠落は破棄
- Intent Staging（staged→approved→committed）

## 9. Circuit Breaker（戦略単位の停止/縮退/復帰）
- トリガ例と縮退モード、復帰方式（cooldown/手動/自動）
- TODO: 各トリガの閾値（timeout 回数、p95、429率等）

## 10. Config Hot-Reload（型検証と適用タイミング）
- スキーマ検証＋適用タイミング（次tick/次bar/次checkpoint）
- 変更監査メタはイベント化（永続ログは Y/O）

## 11. Data Quality Awareness（品質フラグと標準行動）
- stale/gap/dup/out_of_order を受け取る
- 品質低下時：取引停止/サイズ縮小/指値のみ/再同期待ち
- 時刻意味は F が固定、M は品質として処理

## 12. Admission Control（起動前の拒否ゲート）
- descriptor/state_schema/binary_hash/動的スモーク を必ず実施
- 不合格は起動拒否（危険排除）
- TODO: 動的スモークの“疑似イベント”セット定義（最小ケース群）

## 13. External Dependency Policy（特に LLM）
- 外部ネットワーク原則禁止、必要なら別サービス経由（Compute のみ）
- LLM：LL禁止、自由文禁止、LLM出力の直接Intent生成禁止

## 14. Observability（Mが出すもの）
- 主要メトリクス、health_state/reason_code、安全なライブ内省
- TODO: “秘密は絶対に出さない”ための具体的マスキング/禁止項目

## 15. Dependencies（Mが前提とする協約）
- A/F/H/I/J/E/N/O/U/Y との境界協約

## 16. Acceptance Criteria / DoD（完成判定）
- DoD 11項目を Acceptance Criteria として採用
