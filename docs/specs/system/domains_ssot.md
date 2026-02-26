# System Domains SSOT（固定）v1.0

- Document ID: SYS-DOMAINS-SSOT
- Status: Canonical / Fixed
- Purpose: 本書は「総合トレーディングシステム」に存在する **大枠ドメイン（機能領域）**のSSOTである。
- Rule: 追加/削除/統合は SemVer で管理する（追加=MINOR、削除/統合=MAJOR）。

---

## 方針（不変）
- 各ドメインは **Core Spec（固定仕様） / Policy（運用） / Plan（計画）** に分離して文書化できる粒度で統一する。
- 本一覧は **"文書分割単位"ではなく"責務領域の地図"**である。実装効率のため、横断仕様（crosscut）は別途固定仕様で定義する。

---

## A. Platform Foundation（基盤・共通）
- 共通ID/相関（trace_id/run_id/schema_version）、標準エラーモデル、冪等性
- 設定階層（base/env/tenant/bot/secret）、配布、差分、ロールバック
- Feature Flags / Capabilities（機能宣言と安全縮退）
- 環境隔離（dev/stage/prod、paper/shadow/live誤爆防止の基盤）

## B. Secrets / Identity / Access（鍵・認証・権限）
- Secrets保管・ローテ・失効・漏洩対応
- 認証（OIDC/Access等）・RBAC・強操作（承認/理由/期限）
- APIキー/ウォレット鍵の用途分離（read-only / trade / withdraw）

## C. Observability / SRE（観測・監視・SLO）
- 構造化ログ・メトリクス・トレース・アラート（標準化）
- /healthz /metrics /capabilities、SLO/SLI、障害レポート
- Observability of Observability（監視欠損を重大扱い）

## D. Incident Response / Runbooks（障害対応・運用手順）
- 典型障害Runbook、復旧手順、事後検証テンプレ
- degraded運転、代替系への自動切替、監査と証跡

## E. Safety Controller（最終安全装置）
- Kill Switch（UI/CLI/緊急キー二系統）
- SAFE_MODE / CLOSE_ONLY / FLATTEN / HALT（自動発動含む）
- 暴走検知（異常発注/異常キャンセル/PnL急落/遅延悪化）
- ※固定仕様は `docs/specs/crosscut/safety_interlock_spec.md`

## F. Time / Clock Discipline（時刻の規律）
- NTP監視、ドリフト検知、取引所時刻補正
- event_time / recv_time / persist_time の定義統一
- 詳細仕様: `docs/specs/system/F_time_clock_discipline_v1_4.md`

## G. Data Contracts / Schema Governance（スキーマ統治）
- 互換性ルール、バージョニング、契約テストとGate
- 依存グラフ（どのbot/戦略がどのスキーマに依存するか）
- ※契約SSOTは `docs/contracts/`
- ※実装目標機能詳細設計（Level 1 SSOT Outline）は `docs/specs/system/level1_ssot_outline_g_schema_governance_v1_6.md`

## H. Market Data Platform（収集・品質・保存・配信）
- CEX/DEX/オンチェーン/株/ニュース/IRの取り込み（冗長化・再接続）
- 正規化（Canonical Schema）＋品質評価（欠損/重複/遅延/信頼度）
- 保存（hot/warm/cold）と再生成可能なデータレイク設計
- 配信API/ストリーム（bot/ダッシュボード/外部）
- ※Golden Standard Collectorは `docs/specs/ucel/marketdata_collector_spec.md`

## I. Execution Platform（OMS/EMS：唯一の発注出口）
- OrderIntent→注文→約定→残高のライフサイクル
- レート制限、エラー分類、再送設計、部分約定/取消失敗耐性
- Reconciliation（注文/約定/残高照合）と復旧
- paper/shadow/live を同一経路で切替
- ※固定仕様は `docs/specs/ucel/execution_connector_spec.md`

## J. Risk / Policy Gate（中央リスク制御）
- pre-trade / in-run / post-trade ゲート
- 日次損失、最大DD、露出上限、相関上昇時の縮小、ボラ急変対応
- 例外ルール管理（明文化・期限・監査）
- ※Safetyと統合して横断仕様で管理する

## K. Portfolio / Treasury（統一台帳・資金管理）
- Single Book of Record（残高/ポジ/注文の統合）
- PnL（実現/含み、手数料等の分離）、DD、exposure
- 出金/移動統制（ホワイトリスト、timelock、承認）

## L. Bot Control Plane（ボット管理・運用）
- Bot Registry（定義・バージョン・依存・権限）
- ライフサイクル管理（deploy/start/pause/stop/retire）
- Safe Rollout（shadow/canary/段階反映）
- bot別の資金割当・停止条件（Risk Gate連動）
- 監査（命令→判断→発注→結果の完全追跡）

## M. Strategy Runtime / Plugin（戦略実行基盤）
- Rust/Python戦略プラグイン、サンドボックス（暴走防止）
- 状態（state）永続化、再起動復元
- 低遅延経路と高計算経路（AI等）の分離

## N. Experiment / Backtest / Forward（検証・実験管理）
- 同一ABIで backtest/paper/forward/live を切替
- 実運用級シミュレータ（slippage/手数料/板薄/遅延/欠損/順序乱れ）
- リアルタイムフォワード（ライブ同等の障害も再現）＋自動比較
- 実験台帳（dataset_ref/code_ref/params_ref/run）

## O. Deterministic Replay / Audit Event Log（再現性・監査）
- 決定論的リプレイ（同じ入力→同じ出力）
- Append-onlyイベントログ、リプレイ検証、証跡保全（段階導入可）
- ※固定仕様は `docs/specs/crosscut/audit_replay_spec.md`

## P. AI/ML Platform（深層強化学習・最適化・LLM活用）
- 学習ジョブ管理（参照ID、seed固定、再現可能）
- HPO、Walk-forward、リーク/過学習検知
- RLは安全制約付き（Risk Gateで行動空間制限）
- LLMは提案・解析用途中心（実行はゲート・承認・検証を必須化）

## Q. On-chain Trading / Arbitrage（オンチェーン取引・裁定）
- マルチチェーン接続、RPC冗長化、reorg/finality管理
- DEXルーティング、MEV対策、ガス最適化、フロントラン検知
- ブリッジ/跨ぎの遅延・失敗・資金拘束リスク管理
- 署名鍵運用（分離・出金統制・承認）
- ※固定仕様は `docs/specs/ucel/onchain_connector_spec.md`

## R. Equity/IR Analytics（株：決算/IR・格付け・検索）
- 開示/IR/決算情報の収集・構造化
- 指標計算と独自スコア（格付け）
- 任意条件スクリーナー、イベント/テキスト検索
- アラート（決算、修正、増資等）
- ※固定仕様は `docs/specs/ucel/ir_connector_spec.md`

## S. Dashboard Product（運用UI・自由レイアウト）
- 自由レイアウト（Grid/Dock）＋永続化（ユーザー別/端末別）
- ウィジェット基盤（追加/削除/権限/依存）
- Explain UI（なぜ停止/なにが危険/どう復旧）
- 通知設計（重要度階層、通知疲れ防止）
- 外出先監視（モバイルUX、帯域制御）

## T. Testing / Simulation / Chaos（品質保証・最悪条件検証）
- 取引所/チェーンエミュレータ（429、遅延、部分約定、reorg等）
- property-based / fuzz / 障害注入
- ゴールデンE2E（paper→shadow→live安全保証）

## U. Release / Supply Chain / Hardening（リリースと安全強化）
- SBOM、依存スキャン、署名付きリリース
- 最小権限、秘密情報の赤塗り、監査ログ保全

## V. FinOps / Storage Lifecycle（コスト・容量・二重起動）
- データ階層化（hot/warm/cold）、保持/削除ポリシー
- 再計算（rebuild）手順と検証
- 自宅PC運用の冗長化（スナップショット、フェイルオーバー）

## W. Reporting / Accounting / Export（集計・税務・外部出力）
- 取引履歴整形、損益計算、税務向けエクスポート
- 監査証跡（改竄不能性、保持）

## X. Productization / Distribution（製品化・配布・ライセンス）
- 配布チャネル（stable/beta）、更新、ロールバック、署名/検証
- ライセンスモデル（個人/商用/台数/期限等）のSSOT化
- インストール/アップグレード標準化
- 診断パッケージ（Support Bundle）出力の標準化

## Y. Supportability / Diagnostics（サポート容易性・診断）
- support_bundle（秘密赤塗り：起動レポート、Gate結果、Integrity、監査イベント、設定差分）
- エラーコード体系（説明可能な一意ID）
- 再現導線（Deterministic Replayへのリンク）
- ※固定仕様は `docs/specs/crosscut/support_bundle_spec.md`

---

## 変更ルール（SemVer）
- ドメイン追加：MINOR
- ドメイン削除/統合：MAJOR
- 箇条書きの意味追加：MINOR
- 表現整形：PATCH
