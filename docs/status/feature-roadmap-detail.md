# Profinaut Ultimate Gold — 機能別実装進捗レポート

> 作成日: 2026-02-17
> 基準: リポジトリ実コード + `docs/status/ultimate-gold-progress-check.md` + `docs/roadmap.md` Step 0〜21完了

---

## 凡例

| 記号 | 意味 |
|------|------|
| **実装済** | コード＋テストが存在し、動作確認可能 |
| **部分実装** | 土台はあるが仕様の一部のみ、または運用保証が未整備 |
| **設計中** | 設計文書・スキーマのみ、実コードなし |
| **未着手** | コード・設計とも存在しない |

---

## 0) 全領域共通（非機能要件） — 進捗: **55%**

| ID | 機能 | 状態 | 根拠（ファイル/証跡） |
|-----|------|------|----------------------|
| UGF-0-001 | 共通ID (trace_id/request_id/run_id/bot_id/event_id/schema_version) | **部分実装** | `libs/observability/core.py` に `request_id_middleware()` (X-Request-ID伝搬)。`bot_id`/`command_id`/`run_id` は各サービスで使用。`schema_version` は JSON Schema にあるが全イベント統一は未完。 |
| UGF-0-002 | SAFE_MODE状態機械 (NORMAL/DEGRADED/SAFE_MODE/HALTED) | **部分実装** | `contracts/schemas/safe_mode.schema.json` でスキーマ定義済。`services/execution/app/config.py` で `execution_safe_mode` 実装。`services/execution/app/main.py:_resolve_safe_mode()` で判定。ただし全サービス横断の統一遷移ロジックは未完。 |
| UGF-0-003 | Unknown is dangerous (評価不能→ブロック) | **部分実装** | Execution の `policy_gate.py` で SAFE_MODE/HALTED 時ブロック。Market Data で `stale` 検知あり。全コンポーネント横断の統一ルールは未完。 |
| UGF-0-004 | 中央Policy/Risk Gate | **部分実装** | `services/execution/app/policy_gate.py` に `evaluate_policy_gate()` (ALLOW/BLOCK/THROTTLE/REDUCE_ONLY/CLOSE_ONLY/FLATTEN/HALT)。`contracts/schemas/policy_decision.schema.json` でスキーマ定義済。ただし判定ロジックは最小限（safe_mode/live gating/rate limit backoff のみ）。 |
| UGF-0-005 | Kill Switch 3段階 | **部分実装** | `dashboard_api/main.py` に `KillSwitch` モデル (enabled/message)。Dashboard API の Command で `KILL_SWITCH` タイプあり。3段階（CLOSE_ONLY/FLATTEN/HALT）の段階制御は未実装。 |
| UGF-0-006 | 強操作の理由入力＋二重確認＋期限 | **部分実装** | `services/dashboard-api/app/main.py` に confirmation token 方式の二重確認。`contracts/schemas/dangerous_op_meta.json`, `dangerous_confirmation_challenge.json` でスキーマ定義済。Command に `reason`/`expires_at` フィールドあり。全操作への適用は途上。 |
| UGF-0-007 | SoT優先順位固定 | **設計中** | 設計文書に記載あるが、venue/内部台帳/snapshotの優先順位を実装コードで統一するロジックは未実装。 |
| UGF-0-008 | Timeout/Retry/Backoff/CB | **部分実装** | Market Data WS connector にexponential backoff。Execution `live.py` にtimeout/rate limit対応。Circuit Breaker パターンの汎用実装は未着手。 |
| UGF-0-009 | 冪等性 (intent_id/command_id) | **実装済** | Execution `storage.py` に SQLite-backed idempotency mapping。Command に `command_id` + `expires_at`。Order Intent に `idempotency_key`。 |
| UGF-0-010 | Reconciliation失敗時安全遷移 | **部分実装** | `services/dashboard-api/app/main.py` に `/reconcile` エンドポイント + mismatch alert。自動的な停止→修復→canary復帰の自動化フローは未実装。 |
| UGF-0-011 | Bulkhead/Backpressure | **設計中** | WS connector に backpressure の概念はあるが、サービス横断の Bulkhead/部分停止は未実装。 |
| UGF-0-012 | 構造化ログ + 秘密マスク | **実装済** | `libs/observability/core.py` に `audit_event()` + `_redact()` (api_key/secret/token/password/passphrase マスク)。JSON ロガー (`python-json-logger`) 導入済。 |
| UGF-0-013 | 共通メトリクス・アラート | **部分実装** | `dashboard_api/main.py` に Prometheus Counter。Market Data `metrics.py` に IngestMetrics。Dashboard API に stale heartbeat/stuck run アラート。全サービス統一のメトリクス名・アラート基準は未統一。 |
| UGF-0-014 | `/healthz` + `/capabilities` | **実装済** | 全3サービス (dashboard-api, execution, marketdata) に `/healthz` エンドポイント。Market Data に `/capabilities` (build info + ingest stats)。Execution に capabilities 応答。 |
| UGF-0-015 | Run固定参照 (code_ref/dataset_ref) | **部分実装** | Backtest (`worker/backtest.py`) に `dataset_ref` ベースの決定論的データ生成。全コンポーネントへの展開は途上。 |
| UGF-0-016 | append-onlyイベント標準化 | **部分実装** | 監査ログ、Market Data の `md_events_json.schema.json`。完全な MarketData/Signal/Intent/Policy/Order/Fill/Portfolio のイベントチェーンは未完。 |
| UGF-0-017 | 互換性統治 (additive-only/CI) | **実装済** | CI で OpenAPI lint + JSON Schema validation。Gemini review で additive-only チェック。`contracts/` ディレクトリが SSOT。 |
| UGF-0-018 | 鍵管理 (CP無保持/Agent保持) | **実装済** | `.env.example` で Agent 側に交換所APIキー設定。Dashboard API は鍵を保持しない設計。`secrets.compare_digest()` でトークン比較。 |
| UGF-0-019 | Break-glass手順 | **未着手** | 期限付き昇格・監査の仕組みは未実装。 |
| UGF-0-020 | Feature Flag統治 | **部分実装** | 各サービスに環境変数ベースのフラグ (`execution_live_enabled`, `silver_enabled` 等)。動的なフラグ管理・段階昇格の統治フレームワークは未実装。 |
| UGF-0-021 | UTC統一/clock drift/sequence検証 | **部分実装** | Market Data orderbook に `check_gap(prev_seq, next_seq)` でsequence検証。UTC は各所で使用。clock drift 検知の仕組みは未実装。 |
| UGF-0-022 | テスト体系 | **部分実装** | 契約テスト (`tests/test_contracts.py`)、統合テスト (`tests/test_api.py`)、フェイルテスト (`tests/chaos/`)、Replay テスト (`tests/replay/`)。CI に chaos-injection workflow。カナリア検証は未実装。 |

---

## A) 基盤・運用 — 進捗: **60%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-A-001 | 1PR=1scope ガード | **部分実装** | Gemini review workflow で 1PR=1scope チェック。CI レベルの自動ブロックは未実装。 |
| UGF-A-002 | 危険領域ロック | **部分実装** | `docs/status/ultimate-gold-progress-check.md` に LOCK 管理表あり。CI での自動ロック検知は Gemini review の dangerous region 検出で部分対応。 |
| UGF-A-003 | CODEOWNERS | **未着手** | CODEOWNERS ファイルが存在しない。 |
| UGF-A-004 | PRテンプレ | **未着手** | `.github/pull_request_template.md` が存在しない。 |
| UGF-A-005 | ブランチ保護 | **部分実装** | GitHub 側設定（リポジトリ外）。CI 必須は workflows で対応済。up-to-date 必須は要確認。 |
| UGF-A-006 | Observability集約 | **部分実装** | Prometheus metrics, JSON logging, Discord alerting 実装済。集約ダッシュボード（Grafana等）は未構築。 |
| UGF-A-007 | Secrets運用 | **部分実装** | Gitleaks secret scan (`.github/workflows/secret-scan.yml`)。ローテーション/失効対応は手動。 |
| UGF-A-008 | Dead Man's Switch | **実装済** | `bots/simple_mm/main.py` に `DeadmanSwitch` クラス (timeout + recovery 回数制御)。`sdk/python/profinaut_agent/deadman.py` に SDK 版。`.env.example` に `DEADMAN_TIMEOUT_SECONDS=90`, `DEADMAN_ACTION=SAFE_MODE`。 |
| UGF-A-009 | Runbook/Incident運用 | **部分実装** | `docs/runbooks/` ディレクトリ存在。包括的な Runbook セットは途上。 |
| UGF-A-010 | 環境/設定/リリース管理 | **部分実装** | Docker Compose で環境分離。Alembic migrations で DB バージョン管理。層状 config (Pydantic Settings + env)。ロールバック UI/自動化は未実装。 |

---

## B) Control Plane（Dashboard API + UI） — 進捗: **70%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-B-001 | 認証/RBAC/テナント分離 | **部分実装** | `X-Admin-Token` ベースの認証 (`services/dashboard-api/app/auth.py`)。RBAC（ロール定義/権限マトリクス）は未実装。テナント分離は未着手。 |
| UGF-B-002 | Registry (Connector/Strategy/Bot/Dataset/Model) | **部分実装** | Bot Registry (`dashboard_api/bot_registry.py`) + Module Registry (`services/dashboard-api/app/main.py`)。Connector/Dataset/Model の台帳は未実装。 |
| UGF-B-003 | Bot状態・heartbeat・稼働管理 | **実装済** | `/ingest/heartbeat` エンドポイント。Bot/Instance テーブル。`safe_mode`/`degraded` 理由表示。起動/停止コマンド。Web UI の Bots ページ。 |
| UGF-B-004 | Run作成/Cancel/Analytics | **実装済** | Module Runs CRUD + cancel + stats summary。Equity drawdown, performance analytics, failure-rate, throughput, active-age の各分析エンドポイント (Step 12〜18)。 |
| UGF-B-005 | コマンド配布・Ack・監査 | **実装済** | Command dispatch + Ack tracking (`command_id`/`expires_at`/`reason`/`issued_by`)。Web UI の Commands ページ。監査ログ保存。 |
| UGF-B-006 | Portfolio可視化 | **部分実装** | Positions/Metrics ingest + exposure + net-pnl 基盤 (Step 7〜9)。Web UI `/portfolio` ページ。DD 停止連動は未実装。 |
| UGF-B-007 | Markets可視化 | **部分実装** | `/markets` ページ + ticker API。`stale` 閾値表示あり。欠損/遅延の包括的表示は途上。 |
| UGF-B-008 | Datasets/Research UI | **設計中** | Web UI に `/datasets` ルートあり。実データ管理 UI は未実装。 |
| UGF-B-009 | capabilities駆動UI + 安全UX | **部分実装** | `DangerousActionDialog` コンポーネント。Confirmation token 方式。capabilities エンドポイント存在。全 UI での一貫適用は途上。 |

---

## C) Market Data Platform — 進捗: **50%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-C-001 | HTTP/WSコネクタ基盤 | **実装済** | `services/marketdata/app/main.py` (REST poller) + `gmo_ws_connector.py` (WebSocket)。再接続 (exponential backoff)、session recording。 |
| UGF-C-002 | CEX HTTP取得 (ticker/ohlcv/trades) | **実装済** | ticker/OHLCV/trades の REST 取得 + stale 検知。GMO 対応。`/ticker/latest`, `/ohlcv/latest` エンドポイント。 |
| UGF-C-003 | CEX WS取得 (orderbook/trades/sequence再同期) | **部分実装** | `gmo_ws_connector.py` で ticker/trades/orderbook WS 購読。`silver/orderbook.py` に `OrderbookEngine` (snapshot/delta/BBO/gap検知)。完全な sequence 再同期ロジックは途上。 |
| UGF-C-004 | DEX/オンチェーン | **未着手** | コード/設計なし。 |
| UGF-C-005 | 株式/指数/為替/金利 | **未着手** | コード/設計なし（理想拡張）。 |
| UGF-C-006 | ニュース/IR非構造化 | **未着手** | コード/設計なし（理想拡張）。 |
| UGF-C-007 | canonical schema正規化 | **実装済** | `services/marketdata/app/silver/normalizer.py` (381行) に trade/ohlcv/bba/orderbook の正規化。`contracts/schemas/marketdata/` に 7つの JSON Schema。 |
| UGF-C-008 | 保存設計 (TS/snapshot/event/hot-warm-cold) | **部分実装** | SQLite persistence (`db/repository.py`)。MinIO object store (`object_store.py`)。Bronze/Silver 層分離。hot/warm/cold の段階管理は未実装。 |
| UGF-C-009 | 配信 (REST/Stream/Cache/認可) | **部分実装** | REST API 配信済 (`/ticker/latest`, `/ohlcv/latest`, `/orderbook/bbo/latest`)。WS ストリーム配信、キャッシュ層、認可は未実装。 |
| UGF-C-010 | 品質ゲート＋リネージ | **部分実装** | `metrics.py` の IngestMetrics (ingest/fail/dup カウント)。stale 検知。品質スコア算出、リネージ追跡は未実装。 |

---

## D) Trading / Execution Platform — 進捗: **55%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-D-001 | 取引所コネクタ (place/cancel/fill) | **部分実装** | `services/execution/app/live.py` に `GmoLiveExecutor` (place_order/cancel_order)。GMO のみ。replace/fill購読/open-orders は未実装。Binance は設定のみ。 |
| UGF-D-002 | OMS状態機械 (Intent→Order) | **実装済** | `services/execution/app/storage.py` に ACCEPTED→FILLED/CANCELED/REJECTED 状態遷移。`schemas.py` に OrderIntent/Order/Fill モデル。idempotency key による重複防止。 |
| UGF-D-003 | Policy/Risk中央Gate | **部分実装** | `services/execution/app/policy_gate.py` に評価ロジック (ALLOW〜HALT)。`contracts/schemas/policy_decision.schema.json`。ただし判定条件は safe_mode/live gating/rate limit のみで、ポジション制限・exposure 制限等の高度ルールは未実装。 |
| UGF-D-004 | 実行時リスク制御 (close-only/flatten/halt) | **部分実装** | Policy Gate で CLOSE_ONLY/FLATTEN/HALT の判定を定義済。実際の flatten 執行ロジック（全ポジション決済の自動実行）は未実装。 |
| UGF-D-005 | Reconciliation (注文/約定/残高) | **部分実装** | Dashboard API `/reconcile` エンドポイント + mismatch alert。DB migration (`0005_reconcile_results.py`)。自動修復フローは未実装。 |
| UGF-D-006 | 執行アルゴ (TWAP/VWAP等) | **未着手** | コード/設計なし（理想拡張）。 |
| UGF-D-007 | paper/shadow/forward/canary統一経路 | **部分実装** | Execution service に paper mode (fill エンドポイント)。`worker/backtest.py` に backtest。shadow/forward/canary の統一経路は未実装。 |
| UGF-D-008 | 注文根拠監査 | **部分実装** | OrderIntent に reason フィールド。Command に reason/issued_by。Bot ログに decision 記録。統一的な evidence chain は途上。 |
| UGF-D-009 | SOR | **未着手** | コード/設計なし（将来）。 |

---

## E) Strategy / Research Platform — 進捗: **25%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-E-001 | Strategy共通I/F | **部分実装** | `bots/simple_mm/main.py` が market data → decision → OrderIntent の流れを実装。ただし汎用 Strategy I/F の抽象化は未実施。 |
| UGF-E-002 | ルールベース戦略管理 | **部分実装** | simple_mm が momentum ルールを実装。設定の版管理、reason_code の体系化は未実装。 |
| UGF-E-003 | Backtest実行基盤 | **部分実装** | `worker/backtest.py` (133行) に dataset_ref からの決定論的データ生成 + forward-only replay + summary 出力。`tests/replay/test_backtest_runner.py` で決定性・look-ahead 防止を検証。本格的なイベント駆動 backtest エンジン（手数料/スリッページ/遅延モデル）は未実装。 |
| UGF-E-004 | Forward/Shadow/Canary比較 | **設計中** | 概念は設計文書に記載。比較ロジック・逸脱検知の実装は未着手。 |
| UGF-E-005 | ML戦略ライフサイクル | **未着手** | コード/設計なし（理想拡張）。 |
| UGF-E-006 | Feature Store | **未着手** | コード/設計なし（理想拡張）。 |
| UGF-E-007 | Experiment Tracking | **未着手** | コード/設計なし。 |

---

## F) Portfolio / Treasury — 進捗: **45%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-F-001 | Accounts/Balancesスナップショット | **部分実装** | Reconciliation で exchange balance 取得・比較。スナップショット保存の体系化は途上。 |
| UGF-F-002 | Positions管理 | **部分実装** | `/ingest/positions` エンドポイント (symbol/qty/avg_entry/unrealized_pnl)。照合は Reconciliation で部分対応。帰属（Strategy別）は未実装。 |
| UGF-F-003 | Exposure集計 | **実装済** | Portfolio exposure API + Web UI `/portfolio` ページ。通貨/銘柄別の exposure 表示。 |
| UGF-F-004 | PnL/DD計算 | **部分実装** | Net PnL 基盤 (Step 9: cost ingest + formula)。Equity drawdown summary (Step 14)。再現可能な計算、DD 停止連動は未実装。 |
| UGF-F-005 | リスク指標 (VaR/ES) | **未着手** | コード/設計なし（段階導入）。 |
| UGF-F-006 | Treasury | **未着手** | 資金移動ポリシー/引当/監査の実装なし。 |
| UGF-F-007 | 税務/監査レポート | **未着手** | コード/設計なし（理想拡張）。 |

---

## G) Human-in-the-loop / Copilot — 進捗: **5%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-G-001 | 承認フロー | **未着手** | 監査ログ土台のみ。approve→execute フローは未実装。 |
| UGF-G-002 | XAI要約 | **未着手** | 未実装。 |
| UGF-G-003 | 自然言語クエリ | **未着手** | 未実装。 |
| UGF-G-004 | センチメント/Smart Money | **未着手** | 未実装（理想拡張）。 |

---

## H) インフラ/ランタイム — 進捗: **30%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-H-001 | dev/stg/prod分離 + 層状設定 | **部分実装** | Docker Compose で dev 環境。Pydantic Settings で環境変数ベースの層状 config。stg/prod 分離は未実装。 |
| UGF-H-002 | ストレージ役割分離 | **部分実装** | PostgreSQL (OLTP) + SQLite (execution state) + MinIO (object store)。明確な TS/OLAP 分離は未実装。 |
| UGF-H-003 | DR/バックアップ/復旧 | **未着手** | バックアップ/復旧手順・演習の実装なし。 |
| UGF-H-004 | カナリア・段階リリース | **設計中** | 設計文書に記載あり。CI/CD パイプラインでのカナリアデプロイは未実装。 |

---

## I) 品質保証/安全検証 — 進捗: **50%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-I-001 | 契約テスト自動化 | **実装済** | `tests/test_contracts.py` + CI workflow (`ci.yml`) で OpenAPI lint + JSON Schema validation を自動実行。 |
| UGF-I-002 | 擬似取引所/擬似データ統合テスト | **部分実装** | Paper mode (Execution service) で擬似取引。Backtest で擬似データ生成。専用の擬似取引所サーバーは未実装。 |
| UGF-I-003 | 障害注入フェイルテスト | **部分実装** | `tests/chaos/test_fault_injection.py` + `ChaosFaultHandler` (429/503/timeout)。CI に `chaos-injection.yml` workflow。SAFE 遷移検証の自動化は途上。 |
| UGF-I-004 | Replay E2E決定性検証 | **部分実装** | `tests/replay/test_backtest_runner.py` で backtest 決定性を検証。Intent→Gate→OMS→Portfolio の全経路 Replay は未実装。 |

---

## J) データガバナンス/コンプライアンス — 進捗: **20%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-J-001 | データリネージ/品質監査 | **設計中** | Market Data で品質メトリクス収集の土台あり。リネージ追跡システムは未実装。 |
| UGF-J-002 | 監査ログ保全/検索/アクセス制御 | **部分実装** | `libs/observability/core.py` の `audit_event()`。Dashboard API の `/audit` エンドポイント。改ざん耐性（hash chain等）は未実装。 |
| UGF-J-003 | プライバシー/規制対応 | **未着手** | 未実装。 |

---

## K) 拡張性 — 進捗: **35%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-K-001 | Connector SDK | **部分実装** | `sdk/python/` に Agent SDK (agent/client/deadman/processor)。取引所コネクタ追加のための SDK フレームワークは未整備。 |
| UGF-K-002 | Strategy Plugin | **部分実装** | Module Registry がプラグイン的な戦略管理の土台。Strategy Plugin SDK は未実装。 |
| UGF-K-003 | UI Plugin | **未着手** | 未実装。 |
| UGF-K-004 | Schema進化制度 | **部分実装** | `contracts/` SSOT + CI validation + additive-only ルール。正式な versioning/移行ガイドは途上。 |

---

## L) SaaS化（任意） — 進捗: **0%**

| ID | 機能 | 状態 | 根拠 |
|-----|------|------|------|
| UGF-L-001 | 課金/請求 | **未着手** | Deferred。 |
| UGF-L-002 | 利用制限/SLO | **未着手** | Deferred。 |
| UGF-L-003 | テナント隔離/エクスポート/削除要求 | **未着手** | Deferred。 |

---

## サマリー

| カテゴリ | 進捗 | 状態 |
|----------|------:|------|
| 0) 全領域共通NFR | 55% | In Progress |
| A) 基盤・運用 | 60% | In Progress |
| B) Control Plane | 70% | In Progress |
| C) Market Data | 50% | In Progress |
| D) Trading/Execution | 55% | In Progress |
| E) Strategy/Research | 25% | Scoping |
| F) Portfolio/Treasury | 45% | In Progress |
| G) Human-in-the-loop | 5% | Not Started |
| H) インフラ/ランタイム | 30% | Scoping |
| I) 品質保証/安全検証 | 50% | In Progress |
| J) データガバナンス | 20% | Scoping |
| K) 拡張性 | 35% | Scoping |
| L) SaaS化 | 0% | Deferred |

### 全体進捗: 約 **45〜50%**

### 主な強み（実装が進んでいる領域）
1. **Contracts SSOT** — OpenAPI + 25 JSON Schema、CI 検証付き
2. **Bot管理ライフサイクル** — heartbeat/commands/modules/analytics 一通り動作
3. **冪等性・監査** — command_id/idempotency_key、audit_event、秘密マスク
4. **安全機構の土台** — SAFE_MODE状態機械、Policy Gate、Dead Man's Switch、KillSwitch
5. **Market Data Bronze/Silver** — GMO HTTP/WS 取得、正規化、orderbook エンジン

### 主な課題（P0で未完の領域）
1. **SAFE_MODE全サービス統一運用** — Execution 以外への適用が不足
2. **中央Policy Gate高度化** — exposure/position ベースの判定ルール未実装
3. **Reconciliation自動復帰** — 停止→修復→canary フローの自動化
4. **Backtest-First本格化** — 手数料/スリッページ/遅延モデル、全経路 Replay
5. **CODEOWNERS / PRテンプレ** — 基盤運用ガードレールの不足
