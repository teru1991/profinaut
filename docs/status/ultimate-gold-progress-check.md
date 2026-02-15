# Profinaut Ultimate Gold Spec v1.0 進捗チェック管理

## 1. 目的
本ドキュメントは **Profinaut Ultimate Gold Spec v1.0** の実装進捗を、
- Safe-by-Default
- Backtest-First
- Contracts SSOT
- Reproducible / Auditable
の原則に沿って、継続的に可視化・監査可能にするための管理台帳です。

## 2. ステータス定義（共通）
- `Not Started`: 未着手
- `Scoping`: 要件分解・設計中
- `In Progress`: 実装中
- `Blocked`: 依存待ち/意思決定待ち
- `Review`: PRレビュー中
- `Verified`: 検証完了（テスト/運用確認）
- `Released`: 本番反映済み
- `Deferred`: 意図的に後ろ倒し

## 3. 優先度定義
- `P0`: 事故防止・運用継続に必須（安全性/整合性/鍵管理/監査）
- `P1`: 実運用に必要（可観測性/品質/自動復旧/段階導入）
- `P2`: 拡張性・効率改善（高度分析、拡張プラグイン、SaaS化）

## 4. 進捗サマリー（エピック単位）

| Epic ID | 領域 | 説明 | Priority | 現在状態 | 備考 |
|---|---|---|---|---|---|
| UG-00 | 全体NFR | 0-0〜0-11（安全・信頼性・監査・再現・セキュリティ） | P0 | Not Started | 全サービス横断 |
| UG-A | 基盤・運用 | Repo/CI/運用ガードレール | P0 | Not Started | 1PR=1scope/危険領域ロック |
| UG-B | Control Plane | Dashboard API/UI/RBAC/Run管理/監査 | P0 | Not Started | 鍵非保持原則 |
| UG-C | Market Data | 収集・正規化・品質ゲート・配信 | P0 | Not Started | stale/欠損耐性 |
| UG-D | Trading/Execution | OMS/EMS/Policy Gate/Reconcile | P0 | Not Started | 発注出口の単一化 |
| UG-E | Strategy/Research | Strategy I/F、Backtest、Experiment | P1 | Not Started | Backtest-First |
| UG-F | Portfolio/Treasury | 残高/ポジション/PnL/DD/資金管理 | P1 | Not Started | リスク指標連動 |
| UG-G | Human-in-the-loop | 承認フロー/XAI/自然言語照会 | P2 | Not Started | 将来拡張 |
| UG-H | インフラ/ランタイム | 環境分離/DR/段階リリース | P1 | Not Started | 復旧演習含む |
| UG-I | 品質保証/安全検証 | 契約/統合/フェイル/Replay E2E | P0 | Not Started | 決定性検証 |
| UG-J | データガバナンス | リネージ/監査保全/規制対応 | P1 | Not Started | 監査耐性 |
| UG-K | 拡張性 | Connector/Strategy/UI Plugin | P2 | Not Started | 長期進化基盤 |
| UG-L | SaaS化（任意） | 課金/請求/マルチ組織統治 | P2 | Deferred | 到達後導入 |

## 5. マイルストーン管理

| Milestone | 対象 | Exit Criteria（完了条件） | Target |
|---|---|---|---|
| M1 Safety Foundation | UG-00, UG-A, UG-D | SAFE_MODE定義、中央Gate強制、Kill/Close/Flatten、command監査 | TBD |
| M2 Operability & Observability | UG-00, UG-A, UG-B, UG-C | 共通ログ/メトリクス/アラート、/healthz・/capabilities統一 | TBD |
| M3 Reproducible Trading Core | UG-D, UG-E, UG-I | Intent→Gate→OMS一貫、Reconcile自動、Replay E2E成立 | TBD |
| M4 Portfolio & Risk Integrity | UG-F, UG-D, UG-I | PnL/DD再現可能、Unknown時安全側、停止条件連動 | TBD |
| M5 Extensible Platform | UG-K, UG-C, UG-E, UG-B | SDK/Plugin拡張経路確立、互換性ゲート自動化 | TBD |

## 6. ワークリング（実行単位）

| Work ID | Epic | タスク名 | 状態 | Owner | 依存 | 期限 |
|---|---|---|---|---|---|---|
| UG-00-01 | UG-00 | 共通ID（trace/request/run/event/schema_version）の契約化 | Not Started | TBD | contracts | TBD |
| UG-00-02 | UG-00 | SAFE_MODE状態遷移と許可操作の仕様固定 | Not Started | TBD | UG-00-01 | TBD |
| UG-A-01 | UG-A | scope-guard + 危険領域ロック運用導入 | Not Started | TBD | repo policy | TBD |
| UG-B-01 | UG-B | RBACロール導入（operator/risk/platform/treasury） | Not Started | TBD | auth baseline | TBD |
| UG-C-01 | UG-C | MarketData品質指標（遅延/欠損/重複）実装 | Not Started | TBD | ingest | TBD |
| UG-D-01 | UG-D | OrderIntent→PolicyDecision→OrderEventイベント整備 | Not Started | TBD | UG-00-01 | TBD |
| UG-E-01 | UG-E | Backtestイベント駆動実行器（look-ahead防止） | Not Started | TBD | dataset registry | TBD |
| UG-F-01 | UG-F | Exposure/PnL/DDの再現計算パス固定 | Not Started | TBD | reconcile | TBD |
| UG-I-01 | UG-I | 契約互換性CI + リプレイE2E CI | Not Started | TBD | test infra | TBD |

## 7. 週次更新ルール
- 毎週、各Epicの `現在状態` を更新する。
- `Blocked` は必ず「ブロック要因」と「解除条件」を備考に記載する。
- `Verified` へ遷移する際は、対応PR/Runbook/テスト結果をリンクする。
- `Released` への遷移は、環境（dev/stg/prod）と日時を記録する。

## 8. 変更履歴
| Date | Change | Author |
|---|---|---|
| 2026-02-15 | 初版作成 | Codex |
