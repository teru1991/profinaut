# Epic別 実装進捗監査レポート
**Task ID:** PROG-AUDIT-001
**Audit Date:** 2026-02-17
**Scope:** progress-audit-epics-v1
**Branch:** claude/audit-epic-progress-Eu5Ew
**Auditor:** Claude (claude-sonnet-4-5-20250929)

---

## 0. Preflight（Docs OS 整合チェック）

必読順に以下を確認した。各チェックの結果を記録する。

| # | ファイル | 結果 | 備考 |
|---|---|---|---|
| 1 | `docs/SSOT/README_AI.md` | ✅ PASS | SSOT エントリポイントとして正常 |
| 2 | `docs/status/status.json` | ✅ PASS | 必須フィールド確認済み（base_branch/active_task/open_prs/locks_held/next_actions/last_updated） |
| 3 | `docs/handoff/HANDOFF.json` | ✅ PASS | active_task: "CI-010-B"、status.json と整合 |
| 4 | `docs/status/HANDOFF.json` | ⚠️ STALE | task_id=null、timestamp=epoch（1970-01-01）。実質空ファイル |
| 5 | `docs/status/decisions.md` | ⚠️ EMPTY | テンプレートのみ、決定事項の記録ゼロ |
| 6 | `docs/status/trace-index.json` | ⚠️ SPARSE | 2タスク分エントリあり（DOC-FIX-001、CI-010-B）、PR URL/commit SHA は未記入 |
| 7 | `docs/status/trace-index.md` | ⚠️ EMPTY | テンプレートのみ |

### 差分・矛盾の整理

1. **`docs/status/HANDOFF.json` が stale**
   - 現状: `task_id=null, timestamp="1970-01-01T00:00:00Z"` で機能していない
   - `docs/handoff/HANDOFF.json` に real な内容（active_task: "CI-010-B"）が存在
   - README_AI は「両方を使う」と記載するが、status/HANDOFF.json は一切更新されていない
   - **修正方針**: 本タスクで `docs/status/HANDOFF.json` を `docs/handoff/HANDOFF.json` の内容に同期する（最小修正）

2. **`docs/status/decisions.md` が空**
   - 意思決定の記録がゼロ。既存の設計判断（SAFE_MODE採用、policy gate設計等）が未記録
   - **修正方針**: この監査レポートで観察事実として記録。decisions.md への記入は別タスク（LOCK:shared-docs）

3. **trace-index が sparse**
   - PR URL/commit SHA が全フィールドで空
   - **修正方針**: 本タスクで trace-index.json に Epic→証跡リンクを集約（本レポートのセクション6）

### active_task 整合確認

| ファイル | active_task 値 | 整合 |
|---|---|---|
| `docs/status/status.json` | `CI-010-B` | — |
| `docs/handoff/HANDOFF.json` | `CI-010-B` | ✅ 一致 |
| `docs/status/HANDOFF.json` | `null` | ⚠️ 不整合（stale） |

**判定**: 実質的な SSOT（status.json ↔ docs/handoff/HANDOFF.json）は整合している。docs/status/HANDOFF.json のみ stale であり、本タスクで同期する。

---

## 1. Epic A — 基盤・運用（Repo/CI/安全運用）

**現状ステータス:** `In Progress`
**進捗感:** ~60%

### 実装済み要素
| 要素 | 証跡 |
|---|---|
| CI ワークフロー基盤（ci.yml: contracts/API/SDK/web/smoke） | [PR #109](https://github.com/teru1991/profinaut/pull/109) / commit `6606201` |
| CI ワークフロー rules/runbook 統合 | [PR #110](https://github.com/teru1991/profinaut/pull/110) / commit `4a1314f` |
| セキュリティワークフロー強化（dependency-review, supply-chain, Trivy分割） | [PR #112](https://github.com/teru1991/profinaut/pull/112) / commit `223cd38` / CI-010-B |
| CodeQL/secret-scan/gitleaks ワークフロー | `.github/workflows/codeql.yml`, `secret-scan.yml` |
| Gemini PR Review ボット | `.github/workflows/gemini-review.yml` |
| docs OS 整備（README_AI, rules, trace-index, SSOT） | [PR #108](https://github.com/teru1991/profinaut/pull/108) / commit `2c364fa` |
| SSOT ドキュメント bootstrap | [PR #113](https://github.com/teru1991/profinaut/pull/113) / commit `4a0fdaf` |
| PR preflight runbook | `docs/runbooks/pr-preflight.md` / commit `344c8c7` |
| Parallel development safety rules | `docs/rules/parallel-development-safety.md` |
| Task generation policy v3 | `docs/rules/task-generation-policy-v3-enforced.md` |

### 満たしている DoD
- [x] CI が全PR で実行される（contracts/API/SDK/web/smoke）
- [x] セキュリティスキャン（CodeQL, dependency-review, secret-scan, Trivy）が動作
- [x] PR preflight runbook が存在
- [x] docs OS（SSOT/LOCK/HANDOFF/trace）の骨格が存在
- [x] 並列開発安全ルールが文書化されている

### 未達ギャップ（次の最小作業）
- [ ] `CODEOWNERS` ファイルが存在しない（UGF-A-003 未達）— 要: `.github/CODEOWNERS` 作成
- [ ] ブランチ保護ルール（直push禁止、up-to-date必須）が文書化のみで CI enforcement なし（UGF-A-005）
- [ ] decisions.md に実際の決定事項が0件（変更管理 SSOT 未運用）
- [ ] 危険領域ロック制度が status.json で管理されているが LOCK:ci が actual PR なしで held 状態

### LOCK 競合リスク
- 主な競合: `LOCK:ci`（現在 CI-010-B が保持）、`LOCK:shared-docs`

---

## 2. Epic B — Control Plane（Dashboard API + UI）

**現状ステータス:** `In Progress`
**進捗感:** ~70%

### 実装済み要素
| 要素 | 証跡 |
|---|---|
| bots 一覧/状態/heartbeat API | `services/dashboard-api/` / Roadmap Step 0-21 |
| modules/module-runs/commands API | [PR #66](https://github.com/teru1991/profinaut/pull/66) / commit `ea4387b` |
| bot command polling/ack | [PR #64](https://github.com/teru1991/profinaut/pull/64) / commit `62861f5` |
| Commands UI (reason + confirm + expiry, capabilities-driven) | UG-P0-103 / UG-P0-104 / `docs/status/progress-updates/UG-P0-103.md` |
| 危険操作確認スペック（2段確認） | `docs/specs/dangerous-ops-confirmation.md`, `docs/specs/dangerous-ops-taxonomy.md` |
| Portfolio exposure 可視化 | `services/dashboard-api/app/` / Roadmap Step 21 |
| Markets 可視化（/market pages） | [PR #67](https://github.com/teru1991/profinaut/pull/67) / commit `5cc0593` |
| analytics / alerts UI | Roadmap Step 0-21 |
| capabilities endpoint 標準化 | `/healthz`, `/capabilities` 実装済み |

### 満たしている DoD
- [x] REST API 骨格（bot/module/command/analytics）
- [x] Bot状態・heartbeat 管理
- [x] コマンド配布（idempotent、reason/expires実装）
- [x] Portfolio 可視化基盤
- [x] capabilities 駆動 UI の基盤（reason/confirm capabilities-aware）

### 未達ギャップ（次の最小作業）
- [ ] RBAC/認証・認可が未実装（UGF-B-001）— 最優先
- [ ] テナント分離なし（UGF-B-001）
- [ ] Kill-switch UI（read-only panel）が未実装（T192 = planned, 未着手）
- [ ] bots UI の state/degraded/degraded_reason/last_seen 表示（T191 = planned, 未着手）
- [ ] 強操作の UI/API/監査での全機能適用（2段確認は実装済みだが全エンドポイント未適用）

### LOCK 競合リスク
- 主な競合: `apps/web`（T191/T192 が同時に狙う場合は並列不安全）、`contracts`（RBAC 追加時）

---

## 3. Epic C — Market Data Platform

**現状ステータス:** `In Progress`
**進捗感:** ~50%

### 実装済み要素
| 要素 | 証跡 |
|---|---|
| GMO ticker HTTP 取得 | `services/marketdata-rs/` |
| /ticker/latest API | `services/marketdata-rs/` / Roadmap Step 20 |
| markets ページ（Next.js）型安定化 | [PR #67](https://github.com/teru1991/profinaut/pull/67) / commit `5cc0593` |
| marketdata バリデーション強化 | [PR #58](https://github.com/teru1991/profinaut/pull/58) / commit `1a8c39e` |
| orderbook warm-start 永続化復帰 | [PR #115](https://github.com/teru1991/profinaut/pull/115) / commit `cbfdf6d` / [PR #117](https://github.com/teru1991/profinaut/pull/117) |
| GMO trade dedup stable source key | [PR #119](https://github.com/teru1991/profinaut/pull/119) / commit `06fde5b` / [PR #120](https://github.com/teru1991/profinaut/pull/120) / commit `e2f9454` |
| 決定的リプレイ CLI | [PR #114](https://github.com/teru1991/profinaut/pull/114) / commit `34018b5` |

### 満たしている DoD
- [x] HTTP ticker 取得（GMO）
- [x] markets UI ページ
- [x] orderbook warm-start 復帰
- [x] trade dedup の安定化
- [x] replay CLI 基盤

### 未達ギャップ（次の最小作業）
- [ ] WebSocket コネクタ・sequence 再同期（UGF-C-003）
- [ ] データ品質ゲート・lineage（UGF-C-010）
- [ ] canonical schema 正規化（UGF-C-007）— ticker 以外の asset 型が未対応
- [ ] hot-warm-cold ストレージ設計（UGF-C-008）
- [ ] 複数 CEX 対応（現状 GMO のみ）

### LOCK 競合リスク
- 主な競合: `services/marketdata-rs`（独立サービスなので比較的安全）、`contracts`（スキーマ追加時）

---

## 4. Epic D — Trading/Execution Platform

**現状ステータス:** `In Progress`
**進捗感:** ~55%

### 実装済み要素
| 要素 | 証跡 |
|---|---|
| order-intent 作成・cancel | `services/execution/app/main.py` / Roadmap Step 15+ |
| live mode hardening + idempotency 永続化 + orders/fills 履歴 | [PR #68](https://github.com/teru1991/profinaut/pull/68) / commit `47efa14` |
| SAFE_MODE gating（order-intents, cancel） | UG-P0-102 / `docs/status/progress-updates/UG-P0-102.md` |
| 中央 Policy Gate（ALLOW/BLOCK/THROTTLE/REDUCE_ONLY/CLOSE_ONLY/FLATTEN/HALT） | UG-P0-110 / `docs/status/progress-updates/UG-P0-110.md` |
| SAFE_MODE / HALTED 状態機械（contracts スキーマ） | UG-P0-101 / `docs/status/progress-updates/UG-P0-101.md` / `contracts/schemas/safe_mode.schema.json` |
| Reconcile mismatch repair runbook（canary-first） | UG-P0-111 / `docs/status/progress-updates/UG-P0-111.md` / `docs/runbooks/reconcile-mismatch-repair.md` |
| GMO live execution スペック | `docs/specs/execution-gmo.md` |
| 実行可観測性スペック | `docs/specs/execution.md` |

### 満たしている DoD
- [x] Intent→Order 経路
- [x] SAFE_MODE/HALTED による注文ブロック
- [x] 中央 Policy Gate の基盤（全 decision type 実装）
- [x] command_id ベース idempotency
- [x] Reconcile runbook（手動）

### 未達ギャップ（次の最小作業）
- [ ] OMS 状態機械の完全実装（Intent→Submitted→Filled/Rejected/Cancelled）
- [ ] close-only/flatten/halt の自動実行経路（UGF-D-004）
- [ ] Reconciliation 自動復帰（canary 昇格必須化）（UGF-D-005）
- [ ] 注文根拠監査の reason_code/explain 全エンドポイント適用（UGF-D-008）
- [ ] 実行観測強化ログ（degraded/recovery transition）— T060 = planned

### LOCK 競合リスク
- 主な競合: `services/execution`、`contracts`（OMS state machine 追加時）、`migrations`

---

## 5. Epic E — Strategy/Research Platform

**現状ステータス:** `In Progress`（Scoping→In Progress に格上げ）
**進捗感:** ~30%

### 実装済み要素
| 要素 | 証跡 |
|---|---|
| simple_mm bot（basic MM 戦略） | `bots/simple_mm/` |
| Python SDK（heartbeat/command/dead-man） | `sdk/python/` |
| SDK Dead Man's Switch → SAFE_MODE | UG-P0-106 / `docs/status/progress-updates/UG-P0-106.md` |
| Backtest-first 最小実装（dataset_ref 必須・look-ahead 防止・artifact 保存） | UG-P0-112 / `docs/status/progress-updates/UG-P0-112.md` |
| 決定的リプレイ CLI | [PR #114](https://github.com/teru1991/profinaut/pull/114) / commit `34018b5` |

### 満たしている DoD
- [x] シンプル MM 戦略の動作
- [x] SDK dead-man switch → SAFE_MODE 遷移
- [x] Backtest-First の最小基盤（look-ahead 防止、artifact 保存、determinism テスト）
- [x] リプレイ CLI の骨格

### 未達ギャップ（次の最小作業）
- [ ] Backtest-First 完全実装（dataset_ref 全経路、code_ref/config_ref 固定）
- [ ] Forward/Shadow/Canary 比較フレームワーク（UGF-E-004）
- [ ] 実験管理・model_ref 固定運用（UGF-E-007）
- [ ] Strategy 共通 I/F（Signal→Intent の共通インターフェース）（UGF-E-001）

### LOCK 競合リスク
- 主な競合: `bots/`、`sdk/`（比較的独立）、`services/execution`（strategy→execution 接続時）

---

## 6. Epic F — Portfolio/Treasury

**現状ステータス:** `In Progress`
**進捗感:** ~45%

### 実装済み要素
| 要素 | 証跡 |
|---|---|
| /portfolio/exposure API | `services/dashboard-api/` / Roadmap Step 21 |
| positions/metrics ingest | `services/dashboard-api/` |
| net-PnL 基盤 | Roadmap Step 21 |
| Portfolio ページ（Next.js） | `apps/web/app/portfolio/page.tsx` / [PR #107](https://github.com/teru1991/profinaut/pull/107) commit `7c2b0e4` |
| Reconcile runbook | `docs/runbooks/reconcile-mismatch-repair.md` |

### 満たしている DoD
- [x] Exposure 集計（basic）
- [x] Portfolio UI ページ
- [x] Reconcile 手順文書

### 未達ギャップ（次の最小作業）
- [ ] DD 停止連動（drawdown threshold → SAFE_MODE 遷移）（UGF-F-004）
- [ ] PnL/DD の再現可能計算（dataset_ref 固定）（UGF-F-004）
- [ ] Treasury 強操作フロー（資金移動ポリシー）（UGF-F-006）
- [ ] Accounts/Balances スナップショット自動取得（UGF-F-001）

### LOCK 競合リスク
- 主な競合: `services/dashboard-api`（exposure/portfolio）、`contracts`（新スキーマ時）

---

## 7. Epic H — インフラ/ランタイム

**現状ステータス:** `In Progress`（Scoping→In Progress に格上げ）
**進捗感:** ~35%

### 実装済み要素
| 要素 | 証跡 |
|---|---|
| docker-compose.yml によるサービス分割 | `docker-compose.yml` |
| alembic migrations 基盤 | `alembic/` |
| サービス分割（dashboard-api/execution/marketdata-rs） | `services/` |
| 環境設定分離（dev/prod）基盤 | 各サービス設定ファイル |
| supply chain security CI | [PR #112](https://github.com/teru1991/profinaut/pull/112) |

### 満たしている DoD
- [x] compose 環境でのサービス起動
- [x] migrations 基盤
- [x] 基本的なサービス分割

### 未達ギャップ（次の最小作業）
- [ ] dev/stg/prod 明示的分離（UGF-H-001）— 現状 compose のみ
- [ ] DR/バックアップ/復旧演習の手順文書（UGF-H-003）
- [ ] カナリア・段階リリース統治（UGF-H-004）
- [ ] ストレージ役割分離（TS/OLTP/OLAP/Object）（UGF-H-002）

### LOCK 競合リスク
- 主な競合: `infra/`（現在空）、`docker-compose.yml`（Forbidden paths のため本タスクでは変更不可）、`migrations`

---

## 8. Epic I — 品質保証/安全検証

**現状ステータス:** `In Progress`
**進捗感:** ~50%

### 実装済み要素
| 要素 | 証跡 |
|---|---|
| contracts test（OpenAPI/JSON Schema 検証） | `ci.yml` / contracts lint ジョブ |
| dashboard API テスト | `ci.yml` / dashboard-api-tests ジョブ |
| SDK Python テスト | `ci.yml` / sdk-python-tests ジョブ |
| chaos injection テスト（429/5xx/timeout/WS断） | UG-P0-105 / `docs/status/progress-updates/UG-P0-105.md` / `.github/workflows/chaos-injection.yml` |
| backtest 決定性テスト | UG-P0-112 / `docs/status/progress-updates/UG-P0-112.md` / `tests/replay/test_backtest_runner.py` |
| 決定的リプレイ CLI + 検証フック | [PR #114](https://github.com/teru1991/profinaut/pull/114) / commit `34018b5` |
| paper E2E smoke runbook | `docs/runbooks/paper_e2e.md` |
| marketdata smoke 結果 | `docs/verification/marketdata-data-platform-smoke-results.md` |

### 満たしている DoD
- [x] 契約テスト自動化
- [x] 障害注入テスト（基本4種）
- [x] Replay E2E 決定性検証（基本）
- [x] CI での継続実行

### 未達ギャップ（次の最小作業）
- [ ] 障害注入テストの CI 常時実行化（現在は手動/PR trigger のみ）
- [ ] Intent→Gate→OMS→Portfolio の dataset_ref Replay E2E CI ゲート（UGF-I-004A）
- [ ] clock drift / sequence 異常注入テスト
- [ ] SAFE 遷移の自動検証（degraded→SAFE_MODE の機械証明）

### LOCK 競合リスク
- 主な競合: `tests/`、`.github/workflows`（chaos-injection/CI 変更時は LOCK:ci）

---

## 9. status.json ↔ HANDOFF 整合チェック

| チェック項目 | 結果 |
|---|---|
| status.json に `base_branch` フィールド存在 | ✅ "main" |
| status.json に `active_task` フィールド存在 | ✅ "CI-010-B" |
| status.json に `open_prs` フィールド存在 | ✅ (pr_number: null / status: unknown) |
| status.json に `locks_held` フィールド存在 | ✅ ["LOCK:ci"] |
| status.json に `next_actions` フィールド存在 | ✅ (3件) |
| status.json に `last_updated` フィールド存在 | ✅ "2026-02-17T00:00:00Z" |
| docs/handoff/HANDOFF.json の active_task | ✅ "CI-010-B"（status.json と一致） |
| docs/status/HANDOFF.json の active_task | ⚠️ null（stale、本タスクで修正） |

**不整合の状態明文化**: `docs/status/HANDOFF.json` は epoch timestamp のまま放置されており、実質的な SSOT（status.json ↔ docs/handoff/HANDOFF.json）とは別の stale ファイルとなっている。本タスクで同期修正を実施する。

---

## 10. 次に実装すべきタスク候補（Epic 別、最大5件ずつ）

> ※ 本タスクでは実装は行わない。監査に集中。Required Locks と競合しやすいパスを付記。

### Epic A — 基盤・運用

| # | タスク候補 | Required Locks | 競合しやすいパス |
|---|---|---|---|
| A-1 | CODEOWNERS ファイル作成（UGF-A-003） | LOCK:ci | `.github/CODEOWNERS` |
| A-2 | decisions.md 運用記録開始（初期エントリ記入） | LOCK:shared-docs | `docs/status/decisions.md` |
| A-3 | branch 保護ルール CI enforcement 文書化 | LOCK:shared-docs | `docs/rules/` |
| A-4 | CURRENT_STATUS.md の自動生成スクリプト整備 | LOCK:shared-docs | `docs/status/`, `scripts/` |

### Epic B — Control Plane

| # | タスク候補 | Required Locks | 競合しやすいパス |
|---|---|---|---|
| B-1 | bots UI フィールド整合（T191: state/degraded/degraded_reason/last_seen） | LOCK:apps-web | `apps/web/app/bots/` |
| B-2 | kill-switch read-only パネル（T192） | LOCK:apps-web | `apps/web/app/dashboard/` |
| B-3 | RBAC 基盤スケルトン（JWT 検証 middleware） | LOCK:contracts, LOCK:shared-services | `services/dashboard-api/`, `contracts/` |
| B-4 | 危険操作 2 段確認の残エンドポイント適用 | LOCK:shared-services | `services/dashboard-api/` |
| B-5 | capabilities 駆動 UI 全ページ統一 | LOCK:apps-web | `apps/web/` |

### Epic C — Market Data Platform

| # | タスク候補 | Required Locks | 競合しやすいパス |
|---|---|---|---|
| C-1 | WebSocket connector + sequence 再同期（UGF-C-003） | LOCK:shared-services | `services/marketdata-rs/` |
| C-2 | データ品質ゲート基盤（stale 検出・遅延アラート）（UGF-C-010） | LOCK:shared-services | `services/marketdata-rs/`, `services/dashboard-api/` |
| C-3 | canonical schema 正規化（ticker/OHLCV/trades 統一）（UGF-C-007） | LOCK:contracts | `contracts/schemas/` |
| C-4 | 複数 CEX コネクタ骨格（Binance 等、HTTP）（UGF-C-002） | LOCK:shared-services | `services/marketdata-rs/` |
| C-5 | marketdata リプレイ検証の CI ゲート化（UGF-I-004A） | LOCK:ci | `.github/workflows/`, `tests/replay/` |

### Epic D — Trading/Execution Platform

| # | タスク候補 | Required Locks | 競合しやすいパス |
|---|---|---|---|
| D-1 | 実行観測強化ログ（T060: degraded/recovery transition） | LOCK:shared-services | `services/execution/` |
| D-2 | OMS 状態機械完全実装（Intent→Submitted→Filled/Rejected/Cancelled） | LOCK:contracts, LOCK:shared-services | `contracts/`, `services/execution/` |
| D-3 | close-only/flatten 自動実行経路（UGF-D-004） | LOCK:shared-services | `services/execution/` |
| D-4 | Reconciliation 自動復帰（canary 昇格）（UGF-D-005） | LOCK:shared-services, LOCK:migrations | `services/dashboard-api/`, `migrations/` |
| D-5 | 注文根拠監査の reason_code 全エンドポイント適用（UGF-D-008） | LOCK:contracts, LOCK:shared-services | `contracts/`, `services/execution/`, `services/dashboard-api/` |

### Epic E — Strategy/Research

| # | タスク候補 | Required Locks | 競合しやすいパス |
|---|---|---|---|
| E-1 | Strategy 共通 I/F（Signal→Intent）実装（UGF-E-001） | LOCK:shared-services | `bots/`, `sdk/python/` |
| E-2 | Backtest-First 完全実装（code_ref/config_ref 固定）（UGF-E-003） | LOCK:shared-services | `worker/`, `tests/replay/` |
| E-3 | Forward/Shadow 比較フレームワーク骨格（UGF-E-004） | LOCK:shared-services | `services/execution/`, `worker/` |
| E-4 | 実験管理 SSOT（model_ref/dataset_ref tracking）（UGF-E-007） | LOCK:shared-docs | `docs/`, `worker/` |

### Epic F — Portfolio/Treasury

| # | タスク候補 | Required Locks | 競合しやすいパス |
|---|---|---|---|
| F-1 | DD 停止連動（drawdown → SAFE_MODE 遷移）（UGF-F-004） | LOCK:shared-services | `services/dashboard-api/`, `services/execution/` |
| F-2 | PnL/DD 再現可能計算（dataset_ref 固定）（UGF-F-004） | LOCK:shared-services | `services/dashboard-api/` |
| F-3 | Accounts/Balances スナップショット自動取得（UGF-F-001） | LOCK:shared-services | `services/dashboard-api/` |
| F-4 | Treasury 強操作フロー（資金移動ポリシー）（UGF-F-006） | LOCK:contracts, LOCK:shared-services | `contracts/`, `services/dashboard-api/` |

### Epic H — インフラ/ランタイム

| # | タスク候補 | Required Locks | 競合しやすいパス |
|---|---|---|---|
| H-1 | dev/stg/prod 明示分離（compose profiles / env 設定）（UGF-H-001） | LOCK:infra | `infra/`, `docker-compose.yml`（要Forbidden path例外申請） |
| H-2 | DR/バックアップ手順 runbook 作成（UGF-H-003） | LOCK:shared-docs | `docs/runbooks/` |
| H-3 | カナリア段階リリース設計文書（UGF-H-004） | LOCK:shared-docs | `docs/` |
| H-4 | ストレージ役割分離アーキテクチャ文書（UGF-H-002） | LOCK:shared-docs | `docs/` |

### Epic I — 品質保証/安全検証

| # | タスク候補 | Required Locks | 競合しやすいパス |
|---|---|---|---|
| I-1 | Intent→Gate→OMS→Portfolio Replay E2E CI ゲート（UGF-I-004A） | LOCK:ci | `.github/workflows/`, `tests/replay/` |
| I-2 | clock drift / sequence 異常注入テスト | LOCK:ci | `tests/chaos/`, `.github/workflows/` |
| I-3 | SAFE 遷移自動検証（degraded→SAFE_MODE 機械証明） | LOCK:ci | `tests/`, `.github/workflows/` |
| I-4 | chaos injection テスト の CI 常時実行化（PR trigger 以外） | LOCK:ci | `.github/workflows/chaos-injection.yml` |
| I-5 | 擬似取引所 mock による統合テスト基盤（UGF-I-002） | LOCK:shared-services | `tests/integration/`, `services/execution/` |

---

## 付録: 監査使用ファイル一覧

| ファイル | 役割 |
|---|---|
| `docs/SSOT/README_AI.md` | AI 必読エントリポイント |
| `docs/status/status.json` | SSOT（機械可読） |
| `docs/handoff/HANDOFF.json` | 引継ぎ状態（CI-010-B） |
| `docs/status/HANDOFF.json` | 同期先（stale、本タスクで修正） |
| `docs/status/decisions.md` | 意思決定台帳（空） |
| `docs/status/trace-index.json` | トレースリンク SSOT |
| `docs/status/trace-index.md` | 同上（markdown） |
| `docs/status/ultimate-gold-progress-check.md` | Epic 別進捗台帳 |
| `docs/workplan/ultimate-gold-implementation-feature-list.md` | Epic 構造定義 |
| `docs/audits/repo-progress-audit-2026-02-14.md` | 前回監査レポート |
| `docs/audits/ci-workflows-audit.md` | CI 監査（CI-010） |
| `docs/status/progress-updates/UG-P0-10*.md` | 各タスク進捗証跡 |
| `docs/status/progress-updates/UG-P0-11*.md` | 各タスク進捗証跡 |
