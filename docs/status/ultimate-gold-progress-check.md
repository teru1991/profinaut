# Profinaut Ultimate Gold Spec v1.0 進捗チェック管理

## 1. 目的
本ドキュメントは **Profinaut Ultimate Gold Spec v1.0** の実装進捗を、コミット/PRの実績ベースで管理するための台帳です。
「理想仕様との差分」を明確化し、次の優先実装を判断可能にします。

## 2. ステータス定義（共通）
- `Not Started`: 未着手
- `Scoping`: 要件分解・設計中
- `In Progress`: 実装進行中（部分達成）
- `Blocked`: 依存待ち/意思決定待ち
- `Review`: PRレビュー中
- `Verified`: テスト・運用観点で確認済み
- `Released`: 本番反映済み
- `Deferred`: 意図的に後ろ倒し

## 3. 優先度定義
- `P0`: 事故防止・運用継続に必須（安全性/整合性/鍵管理/監査）
- `P1`: 実運用に必要（可観測性/品質/自動復旧/段階導入）
- `P2`: 拡張性・効率改善（高度分析、拡張プラグイン、SaaS化）

## 4. 進捗スナップショット（コミット/PR実績ベース）

> 判定基準: `docs/roadmap.md` の Step 0〜21 完了、`docs/changelog.md` の実装履歴、直近マージPR（#58〜#68）

| Epic ID | 領域 | Priority | 現在状態 | 進捗感 | 実装済み要素（抜粋） | 主な未実装/不足 |
|---|---|---|---|---:|---|---|
| UG-00 | 全体NFR | P0 | In Progress | 55% | Contracts SSOT、idempotency、dead-man、監査ログ、health/capabilities基盤 | SAFE_MODE統一運用、SoT優先順位の明文化、replay決定性の全体保証 |
| UG-A | 基盤・運用 | P0 | In Progress | 60% | CI、契約検証、changelog/roadmap運用、段階的ステップ開発 | 危険領域ロックの制度化、branch保護厳格化の文書固定 |
| UG-B | Control Plane | P0 | In Progress | 70% | bots/modules/module-runs/commands/alerts/analytics UI/API | 本格RBAC、テナント分離、強操作2段確認の全機能適用 |
| UG-C | Market Data | P0 | In Progress | 50% | ticker取得表示、marketページ、latest系API | 品質スコア、lineage、WS sequence再同期の包括実装 |
| UG-D | Trading/Execution | P0 | In Progress | 55% | order-intent系経路、cancel、live hardening、orders/fills履歴 | 中央Policy Gateの完全化、flatten/halt標準化、照合修復runbook |
| UG-E | Strategy/Research | P1 | Scoping | 25% | simple_mm bot、shadow/paper系の土台 | Backtest-First実行器、実験管理、model_ref固定運用 |
| UG-F | Portfolio/Treasury | P1 | In Progress | 45% | positions/metrics ingest、exposure、net-pnl基盤 | DD停止連動、treasury強操作フロー、税務レポート |
| UG-G | Human-in-the-loop | P2 | Not Started | 5% | なし（監査ログ土台のみ） | 承認フロー、XAI要約、自然言語クエリ |
| UG-H | インフラ/ランタイム | P1 | Scoping | 30% | compose/基本環境、migrations、サービス分割 | DR演習、段階リリース統治、設定差分ロールバックUI |
| UG-I | 品質保証/安全検証 | P0 | In Progress | 50% | API/SDK/contracts test、stepごとの回帰追加 | 障害注入自動化、Replay E2Eの継続検証 |
| UG-J | データガバナンス | P1 | Scoping | 20% | 監査ログ保存の基盤 | lineage監査、保持ポリシー厳格化、コンプライアンス運用 |
| UG-K | 拡張性 | P2 | Scoping | 35% | module registry/run枠組み | connector/strategy/ui plugin SDK制度 |
| UG-L | SaaS化（任意） | P2 | Deferred | 0% | 対象外 | 課金/請求/組織分離 |

## 5. 実装済み洗い出し（証跡付き）

### 5.1 主要達成（Roadmap Step 0〜21 完了）
- Step 0〜21 が完了状態。
- Contracts、Dashboard API、Web UI骨格、SDK、Command/Audit、Alerts、Portfolio、Reconcile、Analytics拡張、Resource telemetry まで実装済み。

### 5.2 直近PR/コミット由来の進捗
- PR #68: live mode hardening、idempotency永続化、orders/fills履歴
- PR #67: markets page の型安定化
- PR #66/#64/#63/#62/#61: commands UI/API、bot command polling/ack、運用系の改善
- PR #60: セキュリティ関連のエンドポイント修正
- PR #58: marketdataのバリデーション強化

### 5.3 Ultimate Gold要件へのマッピング（現時点）
- **達成済み（または土台あり）**
  - Contracts SSOT
  - command_idベース冪等
  - 監査ログ保存
  - /healthz・/capabilities
  - 基本的な可観測性（alerts/analytics）
- **部分達成（追加設計・実装が必要）**
  - SAFE_MODE統一仕様
  - 中央Policy/Risk Gateの判定体系統合
  - Reconciliation失敗時の自動復帰手順（canary必須化）
  - Backtest-First（Replay決定性）
- **未着手に近い**
  - 承認フロー（Human-in-the-loop）
  - Data lineage/高度ガバナンス
  - SaaS化要件

## 6. 近接マイルストーン（更新版）

| Milestone | 対象 | Exit Criteria（完了条件） | 状態 |
|---|---|---|---|
| M1 Safety Foundation | UG-00, UG-A, UG-D | SAFE_MODE定義、中央Gate判定一元化、close-only/flatten/halt運用固定 | In Progress |
| M2 Operability | UG-00, UG-A, UG-B, UG-C | 共通ログ項目統一、degraded理由のUI/API一貫表示 | In Progress |
| M3 Reproducible Core | UG-D, UG-E, UG-I | Intent→Gate→OMS→Portfolioをdataset_refでReplay一致 | Scoping |
| M4 Portfolio Integrity | UG-F, UG-D, UG-I | PnL/DD再現計算 + 停止条件連動 + 監査証跡 | Scoping |
| M5 Extensibility | UG-K, UG-C, UG-E, UG-B | Connector/Strategy/UI拡張ガイドと互換性ゲート | Scoping |

## 7. 次アクション（P0優先）
1. **UG-00/UG-D**: SAFE_MODE遷移と許可操作（ALLOW/BLOCK/THROTTLE/REDUCE_ONLY/CLOSE_ONLY/FLATTEN/HALT）を契約と実装で統一。
2. **UG-B**: 重要操作の理由入力 + 二重確認 + 期限を UI/API/監査で強制。
3. **UG-I**: 障害注入テスト（429/5xx/timeout/WS断）をCIに追加し、SAFE遷移検証を自動化。
4. **UG-E**: Backtest-First最小実装（dataset_ref固定、look-ahead防止、artifact保存）。

## 8. 更新ルール
- 更新トリガ: `main` へのマージ、または milestone関連PR の取り込み時。
- 更新責務: 変更を入れたPRが本ドキュメントの該当Epic行を同時更新する。
- `In Progress` 判定は、**コード＋テスト＋運用導線** の3要素のうち2つ以上を満たすこと。

## 9. 変更履歴
| Date | Change | Author |
|---|---|---|
| 2026-02-15 | 初版作成 | Codex |
| 2026-02-15 | コミット/PR実績に基づく進捗更新版へ改訂 | Codex |

## 10. 評価サマリー（Ultimate Gold Spec 清書版 v1.0 照合）

### 10.1 総評
- **網羅性評価: 高い（約90%）**
  - `docs/workplan/ultimate-gold-implementation-feature-list.md` は、提示された清書版 v1.0 の章構成（0, A〜L）と機能群を概ね同一粒度で保持している。
- **実装進捗評価: 中程度（約45〜55%）**
  - `docs/status/ultimate-gold-progress-check.md` の進捗表では、P0中核（UG-00/A/B/C/D/I）が `In Progress` 中心で、運用安全性の“制度化・自動化”が未完。
- **運用投入準備度: 条件付き**
  - 基盤は揃いつつあるが、SAFE_MODE統一、中央Gate完全化、障害注入自動検証、Replay決定性が揃わない限り「最上システム」としては未達。

### 10.2 強み（現状で評価できる点）
1. **仕様のカタログ化が明確**
   - UGF-IDで管理され、非機能要件から拡張性/SaaS化まで一本化されている。
2. **P0の課題認識が妥当**
   - SAFE_MODE、Policy/Risk Gate、Reconciliation、Fail test、Backtest-First など事故直結領域を未実装として正しく認識できている。
3. **マイルストーン設計が実務的**
   - Safety/Operability/Reproducibility/Portfolio Integrity/Extensibility の順で段階化されている。

### 10.3 不足/改善推奨（優先順）
1. **P0項目の Exit Criteria を定量化する**
   - 例: stale秒閾値、SAFE遷移率、reconcile不一致許容時間、canary復旧成功率など。
2. **「機能あり」と「運用保証あり」を分離管理する**
   - 実装済みでも、runbook・監査証跡・自動テストが未整備なら `Verified` に上げない運用を徹底。
3. **UGF-ID単位の証跡リンクを追加する**
   - 各IDに `code/test/doc/PR` リンクを持たせ、進捗表の主観度を下げる。
4. **依存関係（blocking chain）を明文化する**
   - 例: UGF-D-003（中央Gate）未完了なら UGF-D-004/005 を `Verified` 不可、など。

### 10.4 最短で「最上システム」に近づく実行順（提案）
1. **M1 Safety Foundation 完了を最優先**
   - UGF-0-002/0-004/0-005/0-010 と UGF-D-003/004/005 を同時に締める。
2. **M2 Operability の可観測性統一**
   - UGF-0-012/0-013/0-014 と UGF-B-009/UGF-C-010 の整合を取り、degraded理由をUI/APIで同一表現にする。
3. **M3 Reproducible Core で再現性を担保**
   - UGF-0-015/0-016/0-021/0-022 + UGF-E-003 + UGF-I-004 を一連で実装・検証。

### 10.5 判定
- **結論**: 現在の「最終搭載予定機能一覧」は、理想仕様の骨格として十分に妥当。
- **ただし**: Ultimate Gold の到達判定には、P0領域の「実装完了」ではなく「安全運用での再現可能性・監査可能性・自動検証完了」までを必須条件にするべき。
