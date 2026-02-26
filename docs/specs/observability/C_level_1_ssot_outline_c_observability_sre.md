# Level 1 — SSOT Outline（C / Observability・SRE）

## 0. Metadata
- Domain: **C（Observability / SRE）**
- Source: **C（Observability / SRE）SSOT — Required Items Final (WBS)**
- Scope: Observability/SRE に必要な規約・実装要件・運用/ガバナンス一式（Logging/Metrics/Tracing/Alerting/SLO/監査/運用など）

---

## 1. SSOT（Level 1）

### 1.1 C-0 不変条件（Non-negotiables）
- **C-0-1** 観測は嘘をつかない（欠損は OK ではなく **UNKNOWN/DEGRADED**）
- **C-0-2** 観測欠損は安全側へ波及できる（縮退判断の入力）
- **C-0-3** 証拠優先（重大事象は evidence 参照へ辿れる）
- **C-0-4** Secret-free（観測データに秘密・機微情報を出さない）
- **C-0-5** 低オーバーヘッド（観測でサービスを殺さない）

---

### 1.2 C-1 観測規約SSOT（Contracts）
- **C-1-1** 相関ID規約（`run_id / instance_id / trace_id / event_uid / schema_version / op`）
  - TODO: 各IDの生成責務（どの層で発行するか）、フォーマット（UUIDv4等）、寿命/スコープ
- **C-1-2** 互換性ルール（ログキー・メトリクス名/ラベル・トレース属性の破壊的変更禁止）
  - TODO: 破壊的変更の定義、例外、移行手順、互換期間
- **C-1-3** `/capabilities` 規約（未実装/縮退を隠さない、理由を返す）
  - TODO: レスポンススキーマ、reasonコード体系、バージョニング

---

### 1.3 C-2 Logging（構造化ログ）
- **C-2-1** JSONログ固定スキーマ（必須/推奨/error形式）
  - TODO: 必須キー一覧、型、error表現（例：`error.kind` 等）
- **C-2-2** Redaction/Guard（secrets/PII/内部情報の禁止キー・検知時アクション固定）
  - TODO: 禁止キー一覧の正本、検知時アクション（drop/mask/failなど）の優先順位
- **C-2-3** ログ搬送健全性（`drop/queue/flush` をメトリクス化）
  - TODO: 指標名・ラベル、閾値、アラート条件
- **C-2-4** 保持/検索要件（retention、`trace_id/run_id` で辿れる）
  - TODO: retention期間（環境/レベル別）、検索UI/導線要件

---

### 1.4 C-3 Metrics（Prometheus）
- **C-3-1** 命名規約（prefix/type/unit）
  - TODO: prefix設計（service/domain共通）、counter/gauge/histogram等の規約
- **C-3-2** ラベル規約（低カーディナリティ、禁止/許可/上限）
  - TODO: 上限値、禁止ラベル例、例外運用
- **C-3-3** 全サービス共通必須メトリクス（`build_info/uptime/health_status/capabilities_present/heartbeat` 等）
  - TODO: 指標の確定名、型、ラベル、期待値
- **C-3-4** ドメイン別必須メトリクス（collector/execution/storage の最低限固定）
  - TODO: 各ドメインの最低限セットを列挙
- **C-3-5** 欠損＝UNKNOWN（scrape missing / stale / exporter down の状態遷移）
  - TODO: 状態遷移の定義、観測の観測（Obs-of-Obs）との接続

---

### 1.5 C-4 Tracing（OTel）
- **C-4-1** Span taxonomy（collector/execution/storage の固定）
  - TODO: span名規約、属性一覧、親子関係
- **C-4-2** Sampling 方針（default低負荷、P0/P1 で辿れる導線）
  - TODO: P0/P1の定義との接続、強制サンプリング条件
- **C-4-3** Logs↔Traces 相関（`trace_id` の運用ルール）
  - TODO: ログに入れるキー名、相関の必須条件

---

### 1.6 C-5 Health / Ready / Diagnostics API
- **C-5-1** `/healthz`（`OK/DEGRADED/FAILED/UNKNOWN` + checks）
  - TODO: checksの標準項目、表現（配列/辞書）、HTTPステータス方針
- **C-5-2** `/readyz`（受け入れ可否：責務別基準）
  - TODO: 責務別基準（collector/execution/storage）、依存先判定
- **C-5-3** `/metrics`（固定）
- **C-5-4** `/capabilities`（`enabled/disabled/degraded` + reasons）
  - TODO: reasonのフォーマット、安定ID、schema_version

---

### 1.7 C-6 SLI/SLO（UNKNOWNを含む）
- **C-6-1** SLIカテゴリ固定（Availability/Freshness/Completeness/Correctness/Latency/Safety）
- **C-6-2** SLO値は policy 分離（thresholds）
- **C-6-3** エラーバジェット + burn rate（短期/長期）
  - TODO: burn rateウィンドウ（例：5m/1h/6h/3d 等）の確定
- **C-6-4** SLI算出不能＝UNKNOWN（通知/縮退方針まで）
  - TODO: UNKNOWN時の通知先/優先度、縮退ルール

---

### 1.8 C-7 Alerting（行動に接続）
- **C-7-1** Severity（P0–P3）固定定義
  - TODO: 各Pの定義（影響/対応時間/エスカレーション条件）
- **C-7-2** Alert payload 固定（symptom/causes/evidence/runbook）
  - TODO: payloadスキーマ、evidenceリンク形式
- **C-7-3** ノイズ制御（dedupe/group/inhibit/silence）
  - TODO: 既定ルール、権限、SOP
- **C-7-4** 観測の観測（Prom/Loki/Grafana/Alertmanager 停止は P0/P1）
- **C-7-5** オンコール/エスカレーション/未Ack検知（人間側SLO）
  - TODO: 未Ack閾値、通知経路、当番表連携

---

### 1.9 C-8 Dashboards（UNKNOWNを隠さない）
- **C-8-1** 最低限のダッシュボード群（Overview/Collector/Execution/Storage/SLO/Alerts/Obs-of-Obs）
  - TODO: 各ダッシュボードの必須パネル定義
- **C-8-2** UNKNOWN/DEGRADED 表示ルール（緑に見せない）
  - TODO: 表示ルール（色/状態/文言）、UNKNOWN時のデフォルト

---

### 1.10 C-9 Integrity Report（証拠の生成と利用）
- **C-9-1** 必須項目（欠損/差分/観測欠損/ハッシュ紐付け）
  - TODO: ハッシュ対象、ハッシュ方式、参照導線
- **C-9-2** 粒度（任意期間/venue別/priority別）
- **C-9-3** 保存・保持・改ざん検知（secret-free）
  - TODO: 保持期間、改ざん検知方式

---

### 1.11 C-10 Support Bundle（診断パッケージ）＋監査
- **C-10-1** トリガ条件（P0/P1、Integrity FAIL/UNKNOWN 等）
- **C-10-2** 収集内容（health/metrics/logs tail/versions/policy hash/audit tail）
  - TODO: 収集上限（容量/期間）、秘匿情報除外の検証
- **C-10-3** 生成の監査（必ず audit_event）
- **C-10-4** evidence link（アラート→ログ/ダッシュボードへ1クリック導線）
  - TODO: 1クリック導線の具体（URLテンプレ/パラメータ）

---

### 1.12 C-11 Policy Pack（運用可変値のSSOT）
最低限：
- **C-11-1** thresholds（SLO/alert）
- **C-11-2** forbidden_keys（secrets/PII/内部情報）
- **C-11-3** retention（logs/metrics/traces）
- **C-11-4** remote_access（Grafana/Access）
- **C-11-5** hardware_policy（SMART/UPS/IO/ディスク）
- **C-11-6** clock_policy（drift）
- TODO: Policyの正本配置、schema、リリース手順

---

### 1.13 C-12 CI/ガバナンス（観測品質を壊させない）
- **C-12-1** Observability Lint（ログ必須キー、メトリクス名/ラベル、禁止キー、runbook link）
  - TODO: lint実装（ツール/規則/失敗時の挙動）
- **C-12-2** 破壊的変更ブロック（catalog snapshotで検知）
  - TODO: snapshotの生成/比較方式、例外申請
- **C-12-3** 欠損注入 E2E 観点（targets down → UNKNOWN → alert）
  - TODO: テストシナリオ、合否基準、環境（staging等）

---

### 1.14 C-13 観測スタック可用性（バックアップ/復旧/構成管理）
- **C-13-1** Prom/Loki/Grafana/OTel の永続化（データ・設定）
- **C-13-2** 観測設定バックアップ（rules/dashboards/datasources）
- **C-13-3** 復旧手順（観測だけ先に復旧できる順序）
- **C-13-4** 容量計画（growthモデル/限界監視/予防・自動調整）
- TODO: RPO/RTO、バックアップ頻度、復旧検証

---

### 1.15 C-14 アクセス制御と監査（RBAC/変更履歴）
- **C-14-1** Grafana RBAC（閲覧/編集/管理）
- **C-14-2** 変更履歴（アラートルール/ダッシュボード/データソース）
- **C-14-3** 認証基盤（Cloudflare Access 等）の監査ログ保持
- TODO: 監査ログの保存期間/保全要件

---

### 1.16 C-15 データ分類（Secrets以外も含む）
- **C-15-1** ログ分類（Public/Internal/Restricted）と禁止事項
- **C-15-2** 内部URL/IP/host、注文詳細の過剰露出の制限
- **C-15-3** debug時も守る “出してよい最小” 規約
- TODO: 分類判定ルール、検知/強制方法

---

### 1.17 C-16 コスト制御（観測で破産しない）
- **C-16-1** ログ量上限（サービス別/日次/秒間）
- **C-16-2** レベル別保持（ERROR長期、INFO短期など）
- **C-16-3** 高頻度イベントの集計ログ化（サマリ）
- **C-16-4** 予算超過時の動作（degraded + 通知 + 自動調整）
- TODO: 上限値、超過判定、調整アルゴリズム

---

### 1.18 C-17 カーディナリティ爆発防御（自動ブレーキ）
- **C-17-1** 増殖検知（ラベル値の急増）
- **C-17-2** 抑止動作（ラベル削除/サンプル落とし/DEGRADED化/通知）
- **C-17-3** CI lint + Runtime guard（両輪の採否）
- TODO: 検知閾値、抑止の優先順位、例外運用

---

### 1.19 C-18 観測仕様のバージョン管理（catalog/schema）
- **C-18-1** `metrics_catalog_version / log_schema_version` の固定
- **C-18-2** 互換期間と移行ルール（いつ消すか）
- **C-18-3** 旧新混在の扱い（段階移行）
- TODO: バージョンの粒度、参照方法、移行手順

---

### 1.20 C-19 シンセティック監視（E2Eの穴埋め）
- **C-19-1** “targetsは生きてるがデータが死んでる” の検知指標
- **C-19-2** 疑似イベント（購読/永続化/照合）の採否
- **C-19-3** 失敗分類（exchange停止/ネット/clock/設定ミス）
- TODO: 具体的な合成シナリオ、実行頻度、通知ルール

---

### 1.21 C-20 性能観測（runtime telemetry / profiling）
- **C-20-1** CPU/メモリ/FD/スレッド/キュー深さ
- **C-20-2** tokioタスク詰まり・レイテンシ（Rust想定）
- **C-20-3** overhead上限（観測で重くしない）
- TODO: profiling採用可否、収集頻度、上限値

---

### 1.22 C-21 インシデント運用（状態機械）＋ Runbook-as-Code
- **C-21-1** 状態遷移（Detected→Ack→Mitigating→Resolved→Postmortem）
- **C-21-2** 記録（誰が/いつ/何を/なぜ）
- **C-21-3** runbook 定期検証（リンク切れ/手順陳腐化/最低限の実行可能性）
- TODO: Postmortemテンプレ、検証頻度、DoD

---

### 1.23 C-22 依存関係マッピング（原因推定の材料）
- **C-22-1** 依存関係の定義範囲（internal/external）
- **C-22-2** 機械可読な依存グラフ（JSON/YAML）
- **C-22-3** 連鎖障害のルール（上流障害→下流DEGRADE/UNKNOWN）
- **C-22-4** Alert/Runbook への組込み（依存先の状態を添える）
- TODO: グラフの正本、更新手順、可視化

---

### 1.24 C-23 Chaos / Fault Injection（採否確定）
- **C-23-1** 注入カタログ（targets down / ingest stop / 429 / clock drift / disk IO 等）
- **C-23-2** 期待反応（UNKNOWN表示・正しいalert・evidence導線・必要なら縮退）
- **C-23-3** 実行タイミング（staging定期/リリース前/手動訓練）
- TODO: 最小注入セット、合否基準、実施権限

---

### 1.25 C-24 監査ログの完全性（追記専用・改ざん耐性）
- **C-24-1** audit_event の分離方針（一般ログとは別系統も検討）
- **C-24-2** 検証可能なハッシュ連鎖（secret-free）
- **C-24-3** 観測停止時の最低限証拠（auditだけは残る設計）
- TODO: 保存先、検証手順、復旧時の扱い

---

### 1.26 C-25 時刻・順序・単調性（Time Semantics）
- **C-25-1** UTC ts 規約（RFC3339）
- **C-25-2** monotonic/sequence 採否（調査破綻防止）
- **C-25-3** clock drift 超過時の扱い（degrade/unknown + 監査）
- **C-25-4** “順序保証しない” 明文化（相関ID + 時刻窓 + sequence）
- TODO: drift閾値、sequenceの有無とフォーマット

---

### 1.27 C-26 スキーマ/カタログSSOT運用（レジストリ）
- **C-26-1** 正本の場所（log schema / metrics catalog / tracing taxonomy）
- **C-26-2** 更新手順（互換性チェック/CI/リリース）
- **C-26-3** 参照導線（runbook/alert/policy がどの版を参照するか）
- TODO: レジストリ形式（repo/サービス/配布方式）

---

### 1.28 C-27 環境パリティ（dev/staging/prod）
- **C-27-1** 最小パリティ（エンドポイント/ログキー/メトリクス名は一致）
- **C-27-2** 差を許す範囲（retention/サンプリング/通知先＝policy）
- **C-27-3** パリティ検証（環境差事故を防ぐ）
- TODO: 検証方法、CIの組み込み箇所

---

### 1.29 C-28 異常検知（動的）— 採否確定
- **C-28-1** 最小（ルールベース）採否
- **C-28-2** 高度（統計/学習）採否とscope
- **C-28-3** severity/説明可能性/証拠導線
- TODO: 採否結果（Yes/No）と対象範囲の明記、説明可能性要件

---

### 1.30 C 洗い出し完了判定（Final Checklist）
- C-0〜C-28 が固定されていること（原文のチェックリスト項目を満たす）

---

## 2. Capability Index（ID保持）
- C-0, C-0-1, C-0-2, C-0-3, C-0-4, C-0-5  
- C-1, C-1-1, C-1-2, C-1-3  
- C-2, C-2-1, C-2-2, C-2-3, C-2-4  
- C-3, C-3-1, C-3-2, C-3-3, C-3-4, C-3-5  
- C-4, C-4-1, C-4-2, C-4-3  
- C-5, C-5-1, C-5-2, C-5-3, C-5-4  
- C-6, C-6-1, C-6-2, C-6-3, C-6-4  
- C-7, C-7-1, C-7-2, C-7-3, C-7-4, C-7-5  
- C-8, C-8-1, C-8-2  
- C-9, C-9-1, C-9-2, C-9-3  
- C-10, C-10-1, C-10-2, C-10-3, C-10-4  
- C-11, C-11-1, C-11-2, C-11-3, C-11-4, C-11-5, C-11-6  
- C-12, C-12-1, C-12-2, C-12-3  
- C-13, C-13-1, C-13-2, C-13-3, C-13-4  
- C-14, C-14-1, C-14-2, C-14-3  
- C-15, C-15-1, C-15-2, C-15-3  
- C-16, C-16-1, C-16-2, C-16-3, C-16-4  
- C-17, C-17-1, C-17-2, C-17-3  
- C-18, C-18-1, C-18-2, C-18-3  
- C-19, C-19-1, C-19-2, C-19-3  
- C-20, C-20-1, C-20-2, C-20-3  
- C-21, C-21-1, C-21-2, C-21-3  
- C-22, C-22-1, C-22-2, C-22-3, C-22-4  
- C-23, C-23-1, C-23-2, C-23-3  
- C-24, C-24-1, C-24-2, C-24-3  
- C-25, C-25-1, C-25-2, C-25-3, C-25-4  
- C-26, C-26-1, C-26-2, C-26-3  
- C-27, C-27-1, C-27-2, C-27-3  
- C-28, C-28-1, C-28-2, C-28-3
