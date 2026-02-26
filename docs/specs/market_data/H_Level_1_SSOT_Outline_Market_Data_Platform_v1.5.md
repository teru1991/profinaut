# Level 1 SSOT Outline — H（Market Data Platform）v1.5

## 0. Metadata
- **Domain**: H
- **Title**: Market Data Platform 実装目標機能詳細設計（清書・最終 SSOT v1.5）
- **Scope**: 多源データ（CEX/DEX/オンチェーン/株/ニュース/IR 等）の収集〜正規化〜品質評価〜保存〜配信〜監査〜運用自動化まで
- **Out of Scope（直接は担わない）**: Execution / Accounting / Risk / Strategy / Backtest / Forward-test（ただし連携点は固定）

---

## 1. Non-negotiables（目的と到達点）
### 1.1 目的
- 多源データを、欠損・遅延・重複・順序乱れ・再接続・プロバイダ障害・訂正/差替え・再起動・ディスク逼迫・依存更新・監視欠損・仕様変更（ドリフト）・ローカルPC特有の障害を前提に、**正規化 → 品質評価 → 保存 → 配信 → 監査 → 運用自動化**まで一気通貫で提供する。

### 1.2 到達点（11項目）
1. **設定だけで追加**：source追加で ingest→canonical→quality→store→serve が自動起動し、/capabilities・/catalog に反映
2. **壊れても止まらない**：bulkhead で部分障害隔離、縮退/failover/backfill で継続
3. **品質が数値化＋説明可能**：タグ化・低下理由・decision_trace 追跡
4. **再現できる**：Raw→Silver/Gold を決定的に再生成（証跡・固定依存・回帰）
5. **用途別レーン分離**：bot/analysis/monitor/replay を契約として分離、quality-aware routing
6. **止めずに進化**：schema移行・段階ロールアウト・安全ロールバック
7. **運用可能**：/healthz /metrics /capabilities /catalog + support bundle + runbook導線
8. **事故を防ぐ**：分類/ライセンス/欠損埋め/検疫/手動介入点をSSOT化し監査
9. **サプライチェーン堅牢**：SBOM・脆弱性ゲート・依存更新の回帰/性能/カオス検証
10. **ドリフト/DoS耐性**：仕様変更早期検知、Serve側の重い要求から基盤防御
11. **自動レポート**：品質・遅延・欠損・reputation・容量等を日次/週次で把握

---

## 2. Scope（Hが担う責務）
### 2.1 Hが提供するもの
- 収集（WS/REST/RPC/ファイル等）
- 正規化（Canonical/Instrument/Time/Causality）
- 品質評価＋異常検知＋検疫（説明責任含む）
- 保存（Raw/Silver/Gold、hot/warm/cold、manifest、checkpoint、rebuild、signed snapshot）
- 配信（Stream/Query/Snapshot/Replay、QoS、cost control/quotas、license/data_class enforcement）
- ガバナンス（schema semver、consumer pin、lineage、retention、impact、migration、license）
- 運用（隔離、縮退、診断、監査、容量予算、障害注入、段階リリース/ロールバック、手動操作I/F）
- 依存固定（SBOM、脆弱性ゲート、依存更新回帰）
- ドリフト検知（schema/semantic）
- 合成データ生成（回帰固定）

### 2.2 Hが直接担わないもの
- Execution/Accounting/Risk/Strategy/Backtest/Forward-test（連携点は固定）

---

## 3. Logical Architecture（層）
1. Source Connectors  
2. Ingest Runtime（再接続/順序/バックプレッシャ/隔離/チェックポイント/レート配分）  
3. Normalizer（Canonical/Time/Instrument/Causality）  
4. Quality + Anomaly + Quarantine + Explainability  
5. Drift Detection（schema/semantic）  
6. Storage（Raw/Silver/Gold + hot/warm/cold + manifest + rebuild + signed snapshot）  
7. Serve（Stream/Query/Snapshot/Replay + QoS + cost model/quotas）  
8. Governance（Schema/Contract/Lineage/Retention/Impact/Migration/License）  
9. Ops（health/metrics/capabilities/catalog/support bundle/runbook/auto reports/monitoring-of-monitoring）  
10. Release（段階ロールアウト/ロールバック、failover/failback標準手順）  
11. Supply Chain（SBOM/脆弱性/依存更新回帰）  
12. Chaos + Synthetic Data（障害注入＋合成データ生成）

---

## 4. SSOT（宣言・設定）
### 4.1 Source Registry（H0-1）
- **必須フィールド**:  
  - source_id, source_kind, capabilities, auth_requirement, priority  
  - schema_version_in  
  - policy_ref（再接続/縮退/間引き/保持/チェックポイント）  
  - bulkhead_key  
  - qos_profile（bot/analysis/monitor/replay）  
  - data_class_policy_ref  
  - license_ref  
  - rollout_group  
  - reputation_profile_ref  
  - drift_profile_ref  
  - dos_policy_ref
- **受入基準**:  
  - 追加だけで自動起動し、/capabilities と /catalog に反映

### 4.2 Data Classification SSOT（H0-2）
- PUBLIC/SENSITIVE/SECRET をイベント種/フィールド単位で分類
- 保存・配信・ログ・support bundle への含有可否
- redaction 規則を中央管理し強制適用（混入検査と連携）

### 4.3 License SSOT（H0-3）
- 再配布可否、出典表示、保存期間制限、公開範囲
- serve layer は license に従い公開制御
- 監査：取得条件を追跡可能

### 4.4 Consumer Contract SSOT（レーン契約）（H0-4）
- **bot**：低遅延優先、推定混入禁止、低confidence遮断
- **analysis**：整合性優先、確定/未確定区別、推定可（必ず明示）
- **monitor**：網羅性優先、欠損検知とアラート材料
- **replay**：決定的リプレイ（検証/回帰/バックテスト）

### 4.5 Release SSOT（段階リリース/ロールバック）（H0-5）
- rollout group で段階適用（少数→拡大）
- ロールバック条件（品質/遅延/異常の悪化）
- impact report と連携

### 4.6 Failover Playbook SSOT（H0-6）
- primary→secondary の自動/手動手順
- 同期すべき状態：checkpoint、book epoch、キャッシュ、route
- failback の安全条件と段階手順（検証付き）

### 4.7 Supply Chain SSOT（H0-7）
- SBOM 生成
- 既知脆弱性チェックの CI ゲート
- 依存更新時の回帰（golden + perf + chaos）必須化

---

## 5. Ingest Runtime（共通収集ランタイム）— H1
### 5.1 状態機械（H1-1）
- DISABLED / STARTING / RUNNING / DEGRADED / BACKFILLING / PAUSED / FAILED  
- 状態遷移は監査イベント化（原因コード、閾値、ルールID）

### 5.2 Bulkhead / Circuit breaker / Kill switch（H1-2）
- bulkhead_key 単位でリソース分離
- breaker 段階：connect/subscribe/parse/normalize/store
- kill switch：source/event_type/lane 単位で即時停止（監査必須）

### 5.3 Backpressure / Load shedding（H1-3）
- キュー上限（件数/バイト/時間）
- 永続キュー差し替え可能
- 溢れ時：pause→間引き→優先drop→shutdown（lane別）
- 観測負荷もサンプリング制御

### 5.4 Ordering / Dedup / Late arrival（H1-4）
- event_id 重複排除（best-effort + 記録）
- 順序乱れは品質に反映
- late arrival は lane 別に混在/分離/破棄の閾値を固定

### 5.5 Gap detection（H1-5）
- sequence gap / time-window gap 検知し backfill 発火

### 5.6 Checkpoint / Offset（H1-6）
- 進捗永続化、commit point、再試行規約
- 冪等境界（Raw/Silver/Index/Serve cache）を SSOT 化

### 5.7 Rate budget allocation（H1-7）
- consumer 別に予算配分（公平＋優先度プリエンプション）
- 超過時挙動（待機/拒否/縮退）を固定

---

## 6. Backfill / Reconciliation（穴埋め・整合）— H2
- 欠損範囲決定（優先度/予算）
- 手段選択（REST/別ベンダー/再同期）
- ライブ阻害回避（lane/時間帯/優先度）
- 統合（priority/revision/newer/confidence）
- 確定度付与（provisional/final）
- 冗長整合（乖離検知、quorum、finality/reorg）

---

## 7. Canonical Normalization（正規化）— H3
### 7.1 Canonical Envelope（H3-1）
- schema_version
- source_id, venue, instrument_id, event_type
- event_time, recv_time, persist_time
- event_id（決定的）
- sequence（任意）
- payload（型付き）
- quality（confidence/flags/anomaly/quarantine）
- provenance（raw_ref/run_id/code_ref/config_hash）
- session_id（接続世代）
- epoch（再同期境界）
- corr_id（相関/因果）

### 7.2 Instrument / Time semantics（H3-2）
- instrument 正規化（symbol_native/display、刻み、契約仕様、状態イベント）
- マスタは SSOT 参照（H は二重管理しない）
- clock skew 推定、補正適用の事実記録
- 用途別に採用時刻を固定

---

## 8. Quality / Anomaly / Quarantine / Explainability — H4
### 8.1 Quality（H4-1）
- 遅延/欠損/重複/順序違反/スキーマ不正/板整合性/切断/MTTR 等

### 8.2 Explainability（H4-2）
- why_low_confidence（構造化理由）
- decision_trace（縮退/failover/検疫の根拠：ルールID・閾値・メトリクス）
- quality_budget（consumer別許容上限）

### 8.3 Source Reputation（H4-3）
- 過去N日（例：30日）の欠損率/遅延/切断/整合から source_reputation_score
- ルーティング/優先度に反映（A/B 運用可）

### 8.4 Anomaly（H4-4）
- スパイク、出来高、板形状、単位ミス、負値、精度崩れ 等

### 8.5 Quarantine（H4-5）
- 隔離条件
- 復帰条件（自動/承認/永久）
- quarantine query
- 監査証跡

### 8.6 Missing Data Policy（欠損埋めSSOT）（H4-6）
- 欠損のまま / 補間（推定） / 外部補完 を用途別に固定
- 原則：推定は analysis 限定、bot へ混入禁止
- 埋めた場合は根拠とフラグ必須

---

## 9. Drift Detection（仕様/意味変更の検知）— H4+
### 9.1 Schema Drift（H-DRIFT-1）
- フィールド追加/欠落/型変化/ネスト変化をランタイム検知
- drift 時挙動（警告→degraded→隔離等）を drift_profile_ref で固定

### 9.2 Semantic Drift（H-DRIFT-2）
- 値域/分布/精度/単位変化（価格スケール等）を統計的に検知
- symbol 表記規則の変化を検知（Instrument Registry差分）

### 9.3 Provider Changefeed（任意）（H-DRIFT-3）
- 公式アナウンスを取り込み impact report に接続

---

## 10. Order Book 完全性（板）— H4++
- snapshot+delta 整合
- L2/L3
- 再同期
- 品質計測
- 圧縮/間引き（lane別）
- 復元検証

---

## 11. Storage（保存）— H5
- Raw/Silver/Gold（hot/warm/cold）
- retention/compaction/downsampling（証跡付き）
- index/partition
- manifest 整合
- deterministic rebuild（run_id/code_ref/config_hash）

### 11.1 Sampling/Compression Semantics（H5-1）
- 可逆/不可逆を明示
- 不可逆なら損失内容を契約化
- 生成時検証（統計量一致等）

### 11.2 暗号学的整合性（推奨）（H5-2）
- manifest 署名、ハッシュ連鎖
- support bundle も署名して改ざん検知

### 11.3 Signed Snapshot Artifact（H5-3）
- 指定期間/対象のデータを成果物として生成
- 署名・ハッシュ・manifest・provenance（根拠Raw/コード/設定）同梱
- 外部共有可否は license/data_class に従う

---

## 12. Serve（配信）— H6
### 12.1 基本I/F（H6-1）
- Stream/Query/Snapshot/Replay
- lane別QoS
- quality-aware routing
- license/data_class enforcement

### 12.2 Cost Model（DoS耐性）（H6-2）
- query/subscribe/replay のコスト見積（CPU/IO/bytes）
- 上限超過は拒否/縮退（粒度落とし、期間短縮等）
- result size cap / time cap 強制

### 12.3 Quota / Rate limit（consumer別）（H6-3）
- consumer別に quota 配布
- 超過時の扱い（待機/拒否/縮退）固定

### 12.4 Replay Throttle（H6-4）
- replay が本番を潰さないよう帯域・同時数・優先度制御（必要なら時間帯分離）

---

## 13. Governance（契約・互換・影響・移行・ライセンス）— H7
- schema semver
- consumer version pin
- lineage/provenance
- impact report
- migration（shadow/dual/backfill見積）
- license enforcement

---

## 14. Ops / Observability（運用）— H8
### 14.1 API（H8-1）
- /healthz /metrics /capabilities /catalog（期間・対象・品質・lineage検索）

### 14.2 Support bundle（H8-2）
- redaction 強制
- 署名
- 生成失敗も監視対象

### 14.3 Audit Log Preservation（H8-3）
- retention、索引、改ざん検知
- 状態遷移・decision_trace・品質推移から時系列再構成可能

### 14.4 Monitoring the monitoring（H8-4）
- metrics/traces/logs の途切れ自体を検知
- bundle生成不能・マスク失敗も検知

### 14.5 Auto Reports（H8-5）
- 日次/週次で品質・遅延・欠損・reputation・容量予測・failover履歴・ドリフト件数を集計
- 週次比較で悪化傾向を検知

### 14.6 Windows実戦の現実対応（H8-6）
- スリープ復帰/電源断/時刻同期ズレ（NTP）/ネット再確立
- ディスクSMART/容量逼迫の予兆
- ログローテ/圧縮/上限

### 14.7 Stopless maintenance（active/standby）（H8-7）
- active/standby切替
- ストレージ入替手順
- rebuild優先順
- 切替証跡

---

## 15. Manual Control Plane（人間の介入点）
- 強制 failover
- 強制 縮退
- 強制 検疫解除
- 強制 バックフィル停止/開始
- dry-run、段階適用、権限制御
- 全操作は監査必須（誰が/いつ/なぜ）

---

## 16. Capacity / Performance Budget
- events/sec、bytes/sec、p99遅延、ディスク増加率、圧縮率の自動見積
- 性能回帰ゲート
- 観測負荷上限

---

## 17. Release & Rollback（段階運用）
- rollout group で段階適用
- 品質/遅延/異常悪化で自動ロールバック
- impact report と統合
- failover/failback 標準手順に従う

---

## 18. Supply Chain（SBOM/脆弱性/依存更新回帰）
- SBOM生成
- 脆弱性スキャンゲート
- 依存更新時：golden + perf + chaos（＋drift/serve cost）回帰を必須化

---

## 19. Chaos + Synthetic Data（障害注入＋合成データ生成）— H9
### 19.1 Chaos/Failure Injection（H9-1）
- WS切断/黙死/欠損/順序乱れ/重複
- REST 429/遅延
- ディスク逼迫
- 時刻ジャンプ
- RPC不一致/reorg
- 訂正/撤回  
**期待挙動**：巻き込み停止なし、品質反映、縮退/failover/backfill、監査/証跡

### 19.2 Synthetic Data Generator（SSOT）（H9-2）
- 欠損/重複/順序乱れ/遅延/異常/訂正/reorg をパラメトリック合成
- 生成パラメータを SSOT 化し回帰テストに固定（再現性）

---

## 20. Tests（テスト戦略）
- 単体
- 結合（障害注入E2E）
- 回帰（golden + deterministic replay + perf + drift + serve cost + supply chain）を CI に固定

---

## 21. Definition of Done（H完了条件）
- 設定追加でE2E稼働し、/capabilities・/catalog に反映
- 品質/異常/欠損/訂正がタグ化され、説明可能（decision_trace含む）
- checkpoint + 冪等境界 + bulkhead で再起動/部分障害に耐える
- Raw→再生成可能（決定的リプレイ + 証跡 + 固定依存）
- lane/QoS分離、quality-aware routing、license/data_class enforcement
- 検疫運用、欠損埋めSSOT、手動介入I/Fが監査付きで存在
- stopless maintenance と failover/failback標準手順が存在
- 段階リリース/ロールバックが可能
- SBOM/脆弱性ゲート、依存更新回帰がCIで固定
- ドリフト検知があり、仕様変更を早期に検知し縮退できる
- Serve側DoS耐性（コスト/クォータ/上限）がある
- 合成データ生成器で回帰が再現可能
- Windows実戦の現実障害（電源/時刻/ディスク/ログ）に耐える
- 日次/週次レポートで劣化傾向を把握できる
- chaos注入で期待挙動が固定されている

---

## 22. Capability Index（ID保持）
- **H0（SSOT/宣言）**: H0-1, H0-2, H0-3, H0-4, H0-5, H0-6, H0-7
- **H1（Ingest Runtime）**: H1-1, H1-2, H1-3, H1-4, H1-5, H1-6, H1-7
- **H2（Backfill/Reconciliation）**: H2（※子IDなし）
- **H3（Canonical Normalization）**: H3-1, H3-2
- **H4（Quality/Anomaly/Quarantine/Explainability）**: H4-1, H4-2, H4-3, H4-4, H4-5, H4-6
- **H4+（Drift Detection）**: H-DRIFT-1, H-DRIFT-2, H-DRIFT-3
- **H4++（Order Book）**: （※子IDなし）
- **H5（Storage）**: H5-1, H5-2, H5-3
- **H6（Serve）**: H6-1, H6-2, H6-3, H6-4
- **H7（Governance）**: （※子IDなし）
- **H8（Ops/Observability）**: H8-1, H8-2, H8-3, H8-4, H8-5, H8-6, H8-7
- **H9（Chaos/Synthetic Data）**: H9-1, H9-2

---

## TODO（不足・未記載扱い：推測で増やさない）
- TODO: /capabilities /catalog のスキーマ定義（返却項目、フィルタ、ページング、整合条件）
- TODO: policy_ref / qos_profile / data_class_policy_ref / license_ref / reputation_profile_ref / drift_profile_ref / dos_policy_ref の具体スキーマと参照解決方式
- TODO: decision_trace のイベントスキーマ（ルールID命名規則、閾値表現、メトリクス参照）
- TODO: quarantine query のクエリI/F仕様
- TODO: signed snapshot artifact のファイルレイアウト、署名方式、検証手順
- TODO: serve の Stream/Query/Snapshot/Replay 各 API 仕様（認可、互換、SLO、エラーコード）
- TODO: chaos 注入の実装インタフェース（注入点、制御面、観測・評価の自動判定）
- TODO: golden / deterministic replay のアーティファクト仕様（保存形式、比較方法、許容差、差分分類）
