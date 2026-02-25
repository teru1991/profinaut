# Profinaut Ultimate Gold Spec v1.0 実装予定機能一覧

本書は、Ultimate Gold Spec v1.0 の実装対象を **機能カタログ** として整理したものです。
進捗管理は `docs/status/ultimate-gold-progress-check.md` を参照してください。

## 運用ルール（MRUカタログ化）

本ファイルは機能カタログに加えて、**1PR=1scope で実装可能な MRU（Minimum Releasable Unit）** を管理する。

### MRUレコード必須項目
- MRU-ID（例: `UGF-D-003-MRU-001`）
- Scope（PR scope名）
- Priority（P0/P1/P2）
- Depends-on（MRU-ID / PR URL / commit SHA）
- Contracts?（`Yes` の場合は additive-only の単独PR）
- Flags/Capabilities（OFF時挙動、degraded時挙動）
- Allowed paths / Forbidden paths
- DoD（実装・テスト・観測・縮退を含む完了条件）
- Notes/Links（設計・Runbook・PR）

### MRUテンプレート
| Field | Value |
|---|---|
| MRU-ID | `UGF-?-???-MRU-???` |
| Scope | `<scope-name>` |
| Priority | `P0/P1/P2` |
| Depends-on | `<none or link>` |
| Contracts? | `No / Yes(additive-only single PR)` |
| Flags/Capabilities | `<flag> (default OFF), capability:<name>, OFF時:<behavior>, degraded時:<behavior>` |
| Allowed paths | `<path glob list>` |
| Forbidden paths | `<path glob list>` |
| DoD | `<tests + metrics + alert + runbook update>` |
| Notes/Links | `<docs/decision/pr>` |

## P0/P1 優先MRUバックログ（抜け漏れ補完）

### UGF-0-013A SLO/SLI 継続監視
| Field | Value |
|---|---|
| MRU-ID | `UGF-0-013A-MRU-001` |
| Scope | `slo-sli-baseline-and-alerting` |
| Priority | `P0` |
| Depends-on | `UGF-0-012`, `UGF-0-013` |
| Contracts? | `No` |
| Flags/Capabilities | `obs.slo_enforced (default OFF), capability:slo_monitoring, OFF時:記録のみ, degraded時:新規実行抑止` |
| Allowed paths | `docs/**`, `services/**/metrics/**`, `services/**/alerts/**`, `.github/workflows/**` |
| Forbidden paths | `contracts/**`, `migrations/**`, `infra/**` |
| DoD | `SLI定義（latency/error/stale/mttr）+ SLO閾値 + 連続違反アラート + ダッシュボード + Runbook更新` |
| Notes/Links | `0-3 SLO/SLI 必須要件` |

### UGF-0-023 変更管理SSOT（DecisionLog/Assumptions/ChangeLog運用）
| Field | Value |
|---|---|
| MRU-ID | `UGF-0-023-MRU-001` |
| Scope | `decisionlog-assumptions-ssot-governance` |
| Priority | `P0` |
| Depends-on | `UGF-A-004`, `UGF-A-009` |
| Contracts? | `No` |
| Flags/Capabilities | `governance.change_control_enforced (default ON), OFF時:不可, degraded時:変更凍結` |
| Allowed paths | `docs/status/**`, `docs/workplan/**`, `docs/decisions/**`, `docs/assumptions/**`, `.github/**` |
| Forbidden paths | `services/**`, `contracts/**` |
| DoD | `DecisionLog/Assumptionsの更新基準明文化 + PRテンプレ必須化 + CI lint で未記載を失敗化` |
| Notes/Links | `0-7/0-9 変更管理SSOT` |

### UGF-0-024 監査ログ改ざん耐性（段階導入）
| Field | Value |
|---|---|
| MRU-ID | `UGF-0-024-MRU-001` |
| Scope | `audit-log-hash-chain-foundation` |
| Priority | `P1` |
| Depends-on | `UGF-0-016`, `UGF-J-002` |
| Contracts? | `Yes(additive-only single PR)` |
| Flags/Capabilities | `audit.tamper_proof (default OFF), capability:audit_hash_chain, OFF時:通常監査保存, degraded時:live操作禁止` |
| Allowed paths | `contracts/**`, `services/**/audit/**`, `docs/**` |
| Forbidden paths | `migrations/**`, `infra/**` |
| DoD | `イベントハッシュ連結 + 検証CLI + 破損検知アラート + 保全Runbook` |
| Notes/Links | `0-4 改ざん耐性（hash/署名/WORM）` |

### UGF-0-025 サプライチェーン対策（SBOM/脆弱性/生成物コミット防止）
| Field | Value |
|---|---|
| MRU-ID | `UGF-0-025-MRU-001` |
| Scope | `supply-chain-guardrails-ci` |
| Priority | `P1` |
| Depends-on | `UGF-A-002`, `UGF-A-005` |
| Contracts? | `No` |
| Flags/Capabilities | `security.supply_chain_guard (default ON), OFF時:不可, degraded時:release禁止` |
| Allowed paths | `.github/workflows/**`, `scripts/**`, `docs/**` |
| Forbidden paths | `services/**`, `contracts/**` |
| DoD | `SBOM生成 + 脆弱性スキャン + 生成物コミット検知 + lockfile変更ガード` |
| Notes/Links | `0-6 サプライチェーン対策` |

### UGF-I-004A 時刻/順序/決定性の機械検証
| Field | Value |
|---|---|
| MRU-ID | `UGF-I-004A-MRU-001` |
| Scope | `replay-determinism-clock-sequence-tests` |
| Priority | `P0` |
| Depends-on | `UGF-0-021`, `UGF-I-004`, `UGF-E-003` |
| Contracts? | `No` |
| Flags/Capabilities | `qa.replay_determinism_gate (default OFF), capability:replay_e2e, OFF時:警告のみ, degraded時:canary昇格禁止` |
| Allowed paths | `tests/**`, `services/**/replay/**`, `.github/workflows/**`, `docs/**` |
| Forbidden paths | `contracts/**`, `infra/**` |
| DoD | `同一dataset_refでPolicy/OMS/Portfolio一致をCI検証 + clock drift/sequence異常注入テスト + 失敗時SAFE遷移検証` |
| Notes/Links | `0-10/0-11 の機械検証要件` |

## 0. 全領域共通（Non-Functional Requirements）
- **UGF-0-001** 共通ID: trace_id / request_id / run_id / bot_id / agent_id / event_id / schema_version
- **UGF-0-002** SAFE_MODE状態機械（NORMAL/DEGRADED/SAFE_MODE/HALTED）
- **UGF-0-003** Unknown is dangerous（評価不能時は新規行動禁止）
- **UGF-0-004** 中央Policy/Risk Gate最終判定（ALLOW/BLOCK/THROTTLE/REDUCE_ONLY/CLOSE_ONLY/FLATTEN/HALT）
- **UGF-0-005** Kill Switch 3段階（CLOSE_ONLY/FLATTEN/HALT）
- **UGF-0-006** 強操作の理由入力＋二重確認＋期限
- **UGF-0-007** SoT優先順位固定（venue/内部台帳/market snapshot）
- **UGF-0-008** Timeout/Retry/Backoff/Jitter/Circuit Breaker標準
- **UGF-0-009** 冪等性（intent_id/command_idベース）
- **UGF-0-010** Reconciliation失敗時安全遷移（停止→修復→canary復帰）
- **UGF-0-011** Bulkhead/Backpressure/部分停止定義
- **UGF-0-012** 構造化ログ共通スキーマ＋秘密マスク
- **UGF-0-013** 共通メトリクス・共通アラート最小セット
- **UGF-0-014** `/healthz`・`/capabilities`標準化
- **UGF-0-015** Run固定参照（code_ref/config_ref/dataset_ref/...）
- **UGF-0-016** append-onlyイベント群標準化（MarketData/Signal/Intent/Policy/Order/Fill/Portfolio）
- **UGF-0-017** 互換性統治（additive-only / schema_version / CI破壊検知）
- **UGF-0-018** 鍵管理（Control Plane無保持、Agent側保持）
- **UGF-0-019** Break-glass手順（期限付き昇格・監査）
- **UGF-0-020** Feature Flag統治（既定OFF・段階昇格）
- **UGF-0-021** UTC統一・clock drift検知・sequence検証
- **UGF-0-022** テスト体系（契約/統合/フェイル/Replay E2E/カナリア検証）

## A. 基盤・運用
- **UGF-A-001** 1PR=1scopeガード
- **UGF-A-002** 危険領域ロック（contracts/migrations/lockfile/CI/infra）
- **UGF-A-003** CODEOWNERS強制レビュー
- **UGF-A-004** PRテンプレ必須項目（TaskID/Scope/Depends-on/Flags/LOCK/リスク）
- **UGF-A-005** ブランチ保護（PR+CI+up-to-date+直push禁止）
- **UGF-A-006** Observability集約・アラート運用
- **UGF-A-007** Secrets運用（ローテ/失効/漏洩対応）
- **UGF-A-008** Dead Man’s Switch標準運用
- **UGF-A-009** Runbook/Incident/Postmortem運用
- **UGF-A-010** 環境/設定/リリース管理（層状config/ロールバック/移行戦略）

## B. Control Plane（Dashboard API + UI）
- **UGF-B-001** 認証/認可/RBAC/テナント分離
- **UGF-B-002** Registry（Connector/Strategy/Bot/Dataset/Model）
- **UGF-B-003** Bot状態・heartbeat・稼働管理
- **UGF-B-004** Run作成/Cancel/Analytics
- **UGF-B-005** コマンド配布・Ack・監査（idempotent + expires）
- **UGF-B-006** Portfolio可視化（exposure/PnL/DD）
- **UGF-B-007** Markets可視化（stale/欠損/遅延表示）
- **UGF-B-008** Datasets/Research UI
- **UGF-B-009** capabilities駆動UI（安全UX、重要操作2段確認）

## C. Market Data Platform
- **UGF-C-001** HTTP/WSコネクタ基盤
- **UGF-C-002** CEX HTTP取得（ticker/ohlcv/trades）
- **UGF-C-003** CEX WS取得（orderbook/trades、sequence再同期）
- **UGF-C-004** DEX/オンチェーン収集（段階導入）
- **UGF-C-005** 株式/指数/為替/金利拡張（理想）
- **UGF-C-006** ニュース/IR非構造化データ収集
- **UGF-C-007** canonical schema正規化
- **UGF-C-008** 保存設計（TS/スナップショット/イベント/hot-warm-cold）
- **UGF-C-009** 配信（REST/Stream/Cache/認可）
- **UGF-C-010** 品質ゲート＆データリネージ

## D. Trading / Execution Platform
- **UGF-D-001** 取引所コネクタ（place/replace/cancel/fill/open-orders）
- **UGF-D-002** OMS状態機械（Intent→Order）
- **UGF-D-003** Policy/Risk中央Gate
- **UGF-D-004** 実行時リスク制御（close-only/flatten/halt）
- **UGF-D-005** Reconciliation（注文/約定/残高）
- **UGF-D-006** 執行アルゴ（TWAP/VWAP/IOC/FOK等）
- **UGF-D-007** paper/shadow/forward/canary統一経路
- **UGF-D-008** 注文根拠監査（reason_code/explain/evidence）
- **UGF-D-009** SOR（将来）

## E. Strategy / Research Platform
- **UGF-E-001** Strategy共通I/F（入力:市場/資産/状態、出力:Intent）
- **UGF-E-002** ルールベース戦略管理
- **UGF-E-003** Backtest実行基盤（Backtest-First）
- **UGF-E-004** Forward/Shadow/Canary比較
- **UGF-E-005** ML戦略ライフサイクル（理想）
- **UGF-E-006** Feature Store（理想）
- **UGF-E-007** Experiment Tracking

## F. Portfolio / Treasury
- **UGF-F-001** Accounts/Balancesスナップショット
- **UGF-F-002** Positions管理（照合・帰属）
- **UGF-F-003** Exposure集計
- **UGF-F-004** PnL/DD計算（再現可能）
- **UGF-F-005** リスク指標（VaR/ES等、段階導入）
- **UGF-F-006** Treasury（資金移動ポリシー/引当/監査）
- **UGF-F-007** 税務/監査レポート（理想）

## G. Human-in-the-loop / Copilot
- **UGF-G-001** 承認フロー（signal→approve→execute）
- **UGF-G-002** XAI要約（Run/注文/停止理由）
- **UGF-G-003** 自然言語クエリ（RBAC監査付き）
- **UGF-G-004** センチメント/Smart Money統合（理想）

## H. インフラ/ランタイム
- **UGF-H-001** dev/stg/prod分離 + 層状設定
- **UGF-H-002** ストレージ役割分離（TS/OLTP/OLAP/Object）
- **UGF-H-003** DR/バックアップ/復旧演習
- **UGF-H-004** カナリア・段階リリース・ロールバック

## I. 品質保証/安全検証
- **UGF-I-001** 契約テスト自動化
- **UGF-I-002** 擬似取引所/擬似データ統合テスト
- **UGF-I-003** 障害注入フェイルテスト
- **UGF-I-004** Replay E2E決定性検証

## J. データガバナンス/コンプライアンス
- **UGF-J-001** データリネージ/品質監査
- **UGF-J-002** 監査ログ保全/検索/アクセス制御
- **UGF-J-003** プライバシー/規制対応（必要時）

## K. 拡張性
- **UGF-K-001** Connector SDK
- **UGF-K-002** Strategy Plugin
- **UGF-K-003** UI Plugin
- **UGF-K-004** Schema進化制度（versioning/移行/互換性ゲート）

## L. SaaS化（任意）
- **UGF-L-001** 課金/請求
- **UGF-L-002** 利用制限/SLO
- **UGF-L-003** テナント隔離/エクスポート/削除要求

## 実装順推奨（推奨フェーズ）
1. **Phase 1（安全基盤）**: 0, A, D, I の P0 機能
2. **Phase 2（運用可能化）**: B, C, F の P0/P1 機能
3. **Phase 3（研究・最適化）**: E, H, J
4. **Phase 4（高度拡張）**: G, K, L
