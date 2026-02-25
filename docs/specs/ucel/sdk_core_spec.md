# UCEL SDK Core Spec v1.0（固定仕様）
Universal Connector & Exchange Layer（統合SDK/ライブラリ）

- Document ID: UCEL-SDK-CORE-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): A（Platform Foundation）＋ G（Data Contracts）
- Contracts SSOT:
  - `docs/contracts/`（JSON Schema が唯一の正）
  - 監査：`docs/contracts/audit_event.schema.json`
- Goal:
  - 国内CEX/海外CEX/DEX/株IR/指数/ニュース等、外部サービスの差異を吸収し
  - システム内部へ **統一I/F（Canonical Contract）**を提供する再利用SDKを定義する
- Non-goals:
  - 常時稼働SLO・二重化・保持等の運用品質（Collector/Execution/IR/On-chain側Spec）
  - UI/DB/ETL/戦略ロジック
  - 閾値（ms/％/日数/接続数）→ Policy

---

## 0. 不変原則（Non-negotiable）
1. **One SDK, Many Domains**：UCELは"収集アプリ"ではなく全ドメインが使う共通SDK。  
2. **Adapter差分吸収**：外部差分は adapter に閉じ、内部は canonical contract に統一。  
3. **Contracts-first**：入出力は schema_version を持つ契約で固定し、互換性をGateで担保。  
4. **Safety-by-design**：秘密非漏洩、危険挙動の封じ込め（隔離/Quarantine）を設計で保証。  
5. **Observability built-in**：利用側が必ず観測できるよう計測フックをSDKに内蔵。  
6. **Deterministic-friendly**：再現性（event_uid/trace_id/clock）をSDKの責務として提供。

---

## 1. UCELの責務境界（穴を塞ぐ）
### 1.1 UCELが必ず提供する（SDK責務）
- Transport抽象（WS/REST/Socket.IO等）
- Auth/Signing抽象（APIキー、JWT、HMAC、OAuth等：秘密値は保持しない）
- Rate-limit / Retry / Backoff / Jitter / Storm guard
- Standard Error Model（標準エラーモデル）
- Time discipline（event/recv/emit/persist の定義統一、clock skew補助）
- Canonical event（内部イベントモデル：schema_version付き）
- Redaction（禁止キー検知＋赤塗り）
- Observability hooks（構造化ログ/メトリクス/トレース相関）
- Capability宣言（できる/できない、保証の宣言）
- Gate用メタ（schema_version、adapter_version、binary_hash等の識別）

### 1.2 UCELが提供しない（上位ドメイン責務）
- SLO達成（常時稼働、欠損検知、二重化、ディスク運用等）
- Raw/DB/レイク/ETL（保存実装）
- 戦略判断、資金配分、リスク制御（ただしリスク/安全の"強制"には従う）
- UI/ダッシュボード

---

## 2. 固定アーキテクチャ層（実装形態は自由、責務は固定）
- **ucel-core**：共通型、ID、標準エラー、時刻、観測フック
- **ucel-transport**：接続/再接続/heartbeat/IO制御
- **ucel-auth**：署名/認証抽象（秘密はsecret_ref参照のみ）
- **ucel-adapter-kit**：分類・正規化・capabilities・Quarantineフック
- **ucel-contracts**：Canonical Schema + schema_version + 互換性ルール
- **ucel-gates**：契約テスト、禁止キー検知、互換性チェックの枠

---

## 3. 共通ID（traceability）
UCELが生成/伝播できるべき最小セット。

- `trace_id`：外部入力〜内部処理〜出力まで相関
- `run_id`：起動単位識別
- `adapter_id`：`provider + venue + surface`（例：CEX:bitbank:marketdata）
- `schema_version`：契約の版
- `event_uid`：同一イベント判定（重複排除・再現の核）
- `stream_id`：ストリーム識別（市場データ等で使用）

---

## 4. Canonical Event（内部統一I/F）
UCELは外部事実を canonical event として内部へ供給する。

### 4.1 共通ヘッダ（必須概念）
- `event_uid`
- `trace_id`, `run_id`
- `adapter_id`, `schema_version`
- `event_time`（外部時刻、無ければnull）
- `recv_time`（受信時刻：UTC）
- `emit_time`（UCEL発行時刻：UTC）
- `surface`（marketdata/execution/reference/disclosure/onchain 等）
- `kind`（trades/orderbook/ticker/order/fill/balance/announcement 等）
- `symbol_or_asset`（該当する場合）
- `raw_ref`（Raw保存参照：任意）
- `parse_status`（OK/Unknown/Err）
- `payload`（正規化本体：契約に従う）

### 4.2 event_uid（不変原則）
- 外部が提供する一意ID（trade_id/seq/order_id/tx_hash等）を最優先
- 無い場合は決定的生成（payload hash主、時刻依存は最小化）
- "同じ入力→同じevent_uid" を可能な限り満たす

---

## 5. Standard Error Model（標準エラーモデル）
### 5.1 最小分類（必須）
- Network（DNS/TCP/TLS/Timeout）
- Protocol（WS close code、Socket.IO、HTTP status）
- RateLimit（429/ban兆候）
- Auth（署名不正/期限切れ/権限不足）
- Schema（形状変化/必須欠落/Unknown急増）
- State（順序違反/整合破綻）
- Dependency（外部依存：clock/secret provider等）
- Internal（バグ/資源枯渇）

### 5.2 不変フィールド
- `error_code`（説明可能な一意ID）
- `severity`（info/warn/error/critical）
- `retryable`（true/false）
- `category`
- `context`（adapter_id、stream_id、op等）
- `redacted_message`（秘密なし）

---

## 6. Transport Contract（WS/REST/Socket.IO）
- 再接続：指数バックオフ + jitter + storm guard
- 失敗原因分類を必ず残す（Network/Protocol/RateLimit/Auth）
- Backpressure：bounded queue、遅延/詰まりの観測フック
- Input Hardening：payload size、JSON深さ等の上限（形式に応じて）

---

## 7. Auth/Secrets Contract（秘密非漏洩）
- UCELは秘密値を保持しない。`secret_ref` を受け取り解決は外部Provider
- Redactionを共通層で強制（禁止キー検知）
- 検知時は fail-closed 可能（emit抑止＋監査＋必要ならSafety連動）

---

## 8. Capability Model（機能宣言）
各adapterは「できる/できない」を宣言し、上位は安全に縮退できる。

例：
- surface：marketdata/execution/disclosure/onchain
- delivery：ws_only/rest_only/ws_rest
- coverage_mode：full_matrix/priority_matrix/sample/manual
- guarantees：seq_available/checksum_available/ack_available 等

---

## 9. Observability Hooks（SDK内蔵）
必須カテゴリ（SLO/閾値は上位SpecとPolicy）：
- 接続：connected/reconnect/connect_fail
- 取得：request/response、subscribe
- 受信：inbound msgs/bytes
- 遅延：last_message_age、end_to_end_latency（可能なら）
- backpressure：queue_len、pause/resume
- 例外：error_total（category別）
- clock：clock_skew_estimate（注入可能なら）

---

## 10. Contract Governance（schema_version/互換性）
- canonical contracts は schema_version を必須
- 互換性ルールは固定（必須削除=破壊、追加=後方互換）
- CI Gateで契約テストを必須化できる設計

---

## 11. SDK Gate（最低限の固定チェック）
- 禁止キー検知（ログ/例外/メトリクスラベル）
- schema_version 整合
- canonical eventの必須概念欠落禁止
- event_uid 決定性のテスト可能性

---

## 12. Versioning（SemVer）
- MAJOR：canonical contract/エラーモデル等の破壊的変更
- MINOR：後方互換拡張
- PATCH：誤記修正/表現整形
