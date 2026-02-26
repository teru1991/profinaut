# P — AI/ML Platform（Level 1 SSOT Outline）
出典：P. AI/ML Platform — 実装目標機能詳細設計 Final SSOT v2.0（清書・固定版）:contentReference[oaicite:0]{index=0}

## 1.0 目的（Non-negotiable）
### P0. 目的（Non-negotiable）
- P（AI/ML Platform）は、取引・分析システムにおける **学習・評価・最適化・提案（LLM含む）** を、最大限 **安全・堅牢・高速・正確・再現可能** に運用するための基盤。
- Pの成果物は「モデル」そのものではない。
- “使ってよいモデル”を証明するための **再現性・契約・評価・監査・昇格制御・安全縮退** を含む **署名付き成果物** を提供する。

---

## 1.1 ドメイン境界（Pが責任を持つもの）
### P1. ドメイン境界
#### P1-1. Pの責務一覧（固定）
- 再現性（run_id / fingerprint / deterministic / replay）
- 学習ジョブ管理（隔離・失敗耐性・優先度・コスト上限）
- 時間規律（Time Discipline：event/recv/processingの混在禁止、例外カレンダー）
- データ/特徴量ガバナンス（リーク、スキュー、スナップショット、バイアス、マルチ時間軸整合）
- ラベル生成SSOT（Label Factory：定義版本・訂正差分）
- 指標定義SSOT（Metric Registry：同名異義禁止、二重計算）
- 目的関数SSOT（Objective：HPO/WF/昇格でブレない）
- 評価（CV/WF/ストレス/遅延ラベル/microstructure/有意性/回帰検知）
- HPO/探索（Bounded Autonomy：上限＋クールダウン、危険設定検知）
- RL（安全制約必須、破綻防止、policy-conditioned）
- 推論契約（Inference Contract：I/O・SLO・NO-TRADE縮退・confidence・互換性レベル）
- Quarantine推論（契約外/OOD入力隔離）
- ドリフト/劣化/腐敗（急性＋慢性＋staleness）
- shadow A/B・canary計測（判定と提案まで。切替実行は承認経由）
- ML Circuit Breaker（異常連鎖停止：昇格/提案停止、triage優先生成）
- Model Gate（自動判定、期限付き例外、二重ゲート）
- Promotion Lock（昇格時参照ロック）
- Model Registry（署名、互換、適用範囲、配布形態、ロールバック、廃止、TTL）
- Impact Graph（SSOT更新→影響範囲解析→再評価/再学習提案）
- 依存更新安全化（Dependency Drift Gate、Migration Playbook）
- 変更統制/承認UX（差分要約、危険差分、承認誤操作防止）
- 説明責任（Model/Dataset Card、Release Note）
- 保持/削除（Retention）と再現性の両立
- 軽量コンプライアンスログ（記録のみ）

#### P1-2. 入出力（契約）
**入力（参照）**
- dataset_ref / dataset_card_ref（例外カレンダー含む）
- code_ref（生成AI関与メタ含む）
- params_ref
- policy_ref（Risk/Safety制約）
- inference_contract_ref（互換性レベル含む）
- portfolio_policy_ref
- evaluation_policy_ref
- promotion_policy_ref
- retention_policy_ref
- metric_registry_ref
- label_definition_ref
- objective_ref
- fill_cost_model_ref

**出力（提供）**
- model_ref（署名付き）
- report_ref（機械可読）
- decision_ref（提案/根拠/承認/推奨アクション）
- deployable_artifact（Runtime向け）
- model_card_ref / dataset_card_ref / release_note_ref
- triage_pack_ref
- impact_report_ref（影響範囲解析）

---

## 1.2 不変条件（Invariants：SSOTで強制）
### P2. 不変条件（Invariants）
CI/Gateで必ず強制：
1. No-Trade by Default（異常時は必ずNO-TRADE）
2. No Secret Access by Default（Pは秘密に触れない）
3. No External Egress by Default（外部送信は原則禁止、例外は期限＋監査）
4. No Silent Promotion（差分・承認・ロックなし昇格禁止）
5. No Unbounded Optimization（探索は上限つき）
6. No Train-Serving Skew（契約/特徴量差分は致命、deploy不可）
7. No Undocumented Model（Cardなしdeploy不可）
8. No Unobservable Model（必須メトリクス欠落deploy不可）
9. No Research-to-Live Path（研究成果物のlive直行禁止）
10. Time Discipline（時間基準混在は禁止・致命）
11. Objective Fixed（目的関数の恣意変更禁止）
12. Quarantine on OOD（契約外入力は隔離）

---

## 1.3 コア機能（Capabilities）
### P3. 再現性（Reproducibility Core）
- run_id（学習/評価/HPO/WF/RL/LLM単位）
- seed分離（初期化/分割/シャッフル/探索）
- env_fingerprint / data_fingerprint / code_fingerprint
- deterministic mode（非決定論禁止）
- run_id replay（不一致は差分診断＋ステータス付与）
- Evaluation Determinism Lock（評価は決定論で固定）

### P4. 学習ジョブ管理（Training Orchestrator）
- job_spec（kind/inputs/resources/safety/cost_budget）
- lifecycle（retryポリシー分離、checkpoint運用）
- 実行隔離：リアルタイム経路に影響させない
- 優先度：critical > important > research
- リソース逼迫時：researchを自動停止/縮退
- Anti-Thrash：再学習/再探索の連発をクールダウンで抑止（例外は承認）

### P5. 時間規律（Time Discipline）
- event_time / recv_time / processing_time を分離・混在禁止
- 例外カレンダー（メンテ/停止/休場/DST等）を dataset_card に同梱
- 時間基準混在や同期ズレはリーク同等の致命としてGateでreject

### P6. データ・特徴量ガバナンス
- feature_set_id / feature_transform_hash / snapshot（固定）
- leakage検知（時系列/異常相関/time基準/train-inference整合）
- train-serving skew検知（致命）
- マルチ時間軸整合（致命）
- バイアス検知（生存者/選別/ラベル互換）
- poisoning耐性（健全性スコア・隔離・Gate反映）
- Do Not Train Rules：品質/汚染/欠損/カレンダー異常なら学習せず triage_pack 生成
- Source Drift Gate：データソース変更検知→Impact Graph→隔離/再評価提案

### P7. Label Factory（ラベルSSOT）
- label_definition_version を固定（label_definition_ref）
- 遅延ラベル/訂正は差分適用し監査ログ保持
- version不一致の比較は無効（reportで明示）

### P8. Metric Registry（指標SSOT）
- 指標名/定義/算出/単位/閾値/用途を固定（metric_registry_ref）
- 同名異義は禁止（CIで検知）
- 重要指標はCross-check（二重計算）必須化（不一致は致命）

### P9. Objective SSOT（目的関数固定）
- 合成スコアを objective_ref として固定（例：収益×安定×リスク×回転抑制×約定成立性×レイテンシ耐性）
- HPO/WF/昇格は objective_ref に従う（変更は承認＋監査）

### P10. 評価（Evaluation Suite）
- 時系列CV / purged / embargo、WF互換
- 過学習/不安定性、取引視点、頑健性ストレス
- 遅延ラベル評価（訂正ログ反映）
- 不確実性（confidence/abstain）、説明可能性（軽量）
- Model Unit Tests（欠損/範囲外→NO-TRADE、NaN→fail、偏り検知）
- microstructure評価（約定率/キャンセル率/板の薄さ/スプレッド条件）
- Fill/Cost Model SSOTに基づく評価（手数料/スプレッド/滑り、約定確率）
- Overtrading/Churn Gate（回転過多・手数料負けをreject）
- Maintenance-aware（例外期間を通常/異常で分離集計）
- Failure Distribution Training（実運用分布で欠損/遅延注入）
- Latency Sensitivity（レイテンシ感度評価→候補落ち/適用範囲限定）
- Significance Gate（有意性＋効果量で偶然勝ち排除）
- Regression Gate（銘柄/時間帯/レジーム局所悪化を検知→reject）

### P11. HPO/探索・Config Safety
- 探索上限（試行/コスト/危険行動/権限/外部送信）
- 危険設定検知（params_refの危険差分をGateでreject）
- 重要変更は差分レポートで危険差分ハイライト

### P12. RL（安全制約＋破綻防止）
- policy_refでclip/禁止、危険行動率で停止
- reward hacking / 探索暴走検知
- Policy-conditioned training（運用制約を学習に反映）
- offline既定、online隔離（要承認）

### P13. LLM（提案・解析専用）
- Decision Envelope（prompt/context/evidence/gate/approval）
- 参照固定、逸脱（キー要求/外部送信/実行指示）検知→破棄＋監査
- 生成AI関与メタ（code_ref）→追加テスト要求
- triage_packに基づく推奨アクションを定型生成（実行は承認経由）

### P14. 推論契約（Inference Contract）・互換性レベル
- I/Oスキーマ、SLO、NO-TRADE縮退、confidence
- 互換性：compatible / additive / breaking
  - breakingは追加承認＋追加テスト必須
- Shadow-Contract Sync：実データで契約遵守・縮退を定期検証

### P15. Quarantine Inference（隔離レーン）
- 契約外/OOD/異常入力は NO-TRADE + Quarantine
- quarantine率増加→ drift/汚染/仕様変更疑い→ triage_pack 発火
- 本番影響ゼロで解析可能

### P16. Model Gate（中央ゲート）
- 自動判定で合格モデルのみを候補化：
  - leak_ok / skew_ok / data_health_ok
  - overfit_ok / stability_ok / robustness_ok
  - delayed_labels_ok / microstructure_ok / fill_cost_ok
  - churn_ok / maintenance_ok / latency_sensitivity_ok
  - significance_ok / regression_ok
  - inference_contract_ok / unit_tests_ok / observability_ok
  - config_safety_ok / risk_ok / deploy_ok
- 不合格：rejected＋理由＋次実験提案（report_ref）
- 期限付き例外：scope/理由/期限/監査必須
- 研究→本番：二重ゲート（追加テスト＋追加承認）

### P17. Promotion Lock（昇格時ロック）
- 昇格（候補→shadow→live等）時に参照をロックし“後から動かない”ことを保証：
  - dataset snapshot / feature snapshot / code_ref / params_ref
  - inference_contract_ref / label_definition_ref / metric_registry_ref
  - objective_ref / fill_cost_model_ref
- ロックなし昇格は禁止

### P18. Model Registry（署名・配布・廃止）
- 署名とロード時検証（hash/署名/互換性）
- 適用範囲メタデータ（有効/禁止範囲）
- 配布形態（shadow/paper/forward/live）とロールバック
- TTL（期限でshadow再評価or再学習提案）
- staleness_score（市場構造/手数料/メンテ頻度変化で腐敗検知）

### P19. ドリフト・劣化・Circuit Breaker
- 急性ドリフト＋慢性劣化（Slow Drift）を監視
- stalenessで影響も加味
- ML Circuit Breaker：異常連鎖（quarantine率/NO-TRADE率/極端出力率/drift）で
  - live提案/昇格を停止
  - shadow/paperのみ継続
  - triage_packをcritical生成

### P20. Impact Graph（影響範囲解析）
- metric/label/contract/feature_set/fill_cost_model/docs SSOTの変更時：
  - 影響する model_ref/run_id/report_ref を自動列挙
  - impact_report_ref を生成し、再評価/再学習提案リストを作る

### P21. 依存更新・移行（Dependency Drift / Migration Playbook）
- env_fingerprint差分で依存更新を検知
- 再現性が崩れる更新は昇格禁止
- 重大更新は migration_playbook_ref を必須化（手順＋ロールバック＋検証）

### P22. 変更統制・承認UX（Approval Safety）
- 差分レポート（data/feature/code/params/gate）＋危険差分ハイライト
- breaking変更、scope/期限、例外承認は二重確認
- 緊急承認テンプレ（理由必須）
- 監査に耐える承認ログ固定

### P23. 説明責任（Cards / Release Notes）
- Model Card：用途/範囲/弱点/リスク/Gate結果/SLO/縮退/互換性/TTL/廃止条件
- Dataset Card：取得元/期間/欠損遅延偏り/除外/リーク対策/汚染検知/例外カレンダー
- Release Note：変更点、効果、リスク、適用範囲、ロールバック手順

### P24. 保持/削除（Retention）
- 期限/容量を定義しつつ、再現性維持の要約（fingerprint/統計/card/report/impact）は必ず保持
- “消したら再現不能”を禁止

### P25. 軽量コンプライアンスログ（記録のみ）
- 判断根拠の最低限ログ
- LLM利用/外部送信の有無
- 情報種別（公知/非公知/推測）の分類ログ（分類のみ）

---

## 1.4 完成条件（Definition of Done）
### P. 完成条件（DoD Final）
- Time Discipline / Label Factory / Metric Registry / Objective / Fill-Cost がSSOT固定
- Gateでリーク/スキュー/汚染/遅延ラベル/約定成立性/回転過多/有意性/回帰/契約遵守を自動判定
- Promotion Lockなし昇格不可、研究→本番は二重ゲート
- Quarantine＋Circuit Breaker＋Do Not Trainで異常時は安全側（NO-TRADE）
- Impact GraphでSSOT更新の影響範囲と再評価対象が自動で出る
- 依存更新は安全化され、移行プレイブックで長期運用が破綻しない
- Card/Release Note/監査ログで説明責任と引継ぎが成立する

---

## 9.0 Capability Index（ID保持）
- P0：目的（Non-negotiable）
- P1：ドメイン境界
  - P1-1：責務一覧（固定）
  - P1-2：入出力（契約）
- P2：不変条件（Invariants）
- P3：再現性（Reproducibility Core）
- P4：学習ジョブ管理（Training Orchestrator）
- P5：時間規律（Time Discipline）
- P6：データ・特徴量ガバナンス
- P7：Label Factory（ラベルSSOT）
- P8：Metric Registry（指標SSOT）
- P9：Objective SSOT（目的関数固定）
- P10：評価（Evaluation Suite）
- P11：HPO/探索・Config Safety
- P12：RL（安全制約＋破綻防止）
- P13：LLM（提案・解析専用）
- P14：推論契約（Inference Contract）・互換性レベル
- P15：Quarantine Inference（隔離レーン）
- P16：Model Gate（中央ゲート）
- P17：Promotion Lock（昇格時ロック）
- P18：Model Registry（署名・配布・廃止）
- P19：ドリフト・劣化・Circuit Breaker
- P20：Impact Graph（影響範囲解析）
- P21：依存更新・移行（Dependency Drift / Migration Playbook）
- P22：変更統制・承認UX（Approval Safety）
- P23：説明責任（Cards / Release Notes）
- P24：保持/削除（Retention）
- P25：軽量コンプライアンスログ（記録のみ）
# P — AI/ML Platform（Level 2 Deep Spec）
※入力に「Non-negotiable（P0）」「Canonical Model/Contract（P1-2/P14）」「Behavior/Tests相当（P2/P10/P16 等）」が揃っているため、整理のみでDeep Specを併記（新規仕様は追加しない）。:contentReference[oaicite:1]{index=1}

## 0. 原則（Non-negotiable）
### P0
- 目的：学習・評価・最適化・提案（LLM含む）を、安全・堅牢・高速・正確・再現可能に運用。
- 成果物：モデルそのものではなく、**“使ってよいモデル”を証明する署名付き成果物**。

---

## 1. 正式な境界と責務
### P1 / P1-1
- Pが持つ責務はP1-1列挙の通り（固定）。
- TODO: Pの外部（Runtime/Execution/Tradingなど）との責務分界（Owner/Oncall/運用SLO）を、別SSOTがあれば参照リンクとして追記。

---

## 2. Canonical Artifacts（参照・成果物）
### P1-2（入出力）
**Inputs（参照）**
- dataset_ref / dataset_card_ref（例外カレンダー含む）
- code_ref（生成AI関与メタ含む）
- params_ref
- policy_ref（Risk/Safety制約）
- inference_contract_ref（互換性レベル含む）
- portfolio_policy_ref
- evaluation_policy_ref
- promotion_policy_ref
- retention_policy_ref
- metric_registry_ref
- label_definition_ref
- objective_ref
- fill_cost_model_ref

**Outputs（提供）**
- model_ref（署名付き）
- report_ref（機械可読）
- decision_ref（提案/根拠/承認/推奨アクション）
- deployable_artifact（Runtime向け）
- model_card_ref / dataset_card_ref / release_note_ref
- triage_pack_ref
- impact_report_ref（影響範囲解析）

**Artifactの最低要件（本文に明記されている範囲で整理）**
- model_ref：署名/互換性/適用範囲/配布形態/ロールバック/廃止/TTL（P18）
- report_ref：Gate不合格理由、次実験提案（P16）
- decision_ref：提案＋根拠＋承認＋推奨アクション（P1-2/P13）
- triage_pack_ref：Do Not TrainやCircuit Breaker時に生成（P6/P19/P13）
- impact_report_ref：SSOT変更の影響範囲列挙（P20）
- TODO: 各refの物理フォーマット（JSON/YAML/Parquet等）と必須フィールド定義（別文書があれば参照）。

---

## 3. 強制不変条件（CI/Gateで強制）
### P2（Invariants）
- No-Trade by Default
- No Secret Access by Default
- No External Egress by Default（例外：期限＋監査）
- No Silent Promotion
- No Unbounded Optimization
- No Train-Serving Skew
- No Undocumented Model
- No Unobservable Model
- No Research-to-Live Path
- Time Discipline
- Objective Fixed
- Quarantine on OOD

**CI/Gate適用の位置づけ（整理）**
- すべての昇格/配布候補化/デプロイ可能化（deployable_artifact生成）に先立ち強制される。
- TODO: CI/Gateの実装境界（どのリポジトリ/パイプライン/ステージで走るか）を参照リンクで明確化。

---

## 4. Reproducibility Core（再現性）
### P3
- run_id：学習/評価/HPO/WF/RL/LLM単位で付与
- fingerprint：env_fingerprint / data_fingerprint / code_fingerprint
- deterministic mode：非決定論禁止
- replay：不一致は差分診断＋ステータス付与
- Evaluation Determinism Lock：評価は決定論で固定

**Operational implications（整理）**
- env/data/codeの差分は、Promotion/Registry/Dependency Drift Gate（P21）と連動して“動かない”を担保する。
- TODO: replay時の「差分診断」出力仕様（report_ref内のセクション/フィールド）を参照リンクで確定。

---

## 5. Training Orchestrator（学習ジョブ管理）
### P4
- job_spec：kind/inputs/resources/safety/cost_budget
- lifecycle：retryポリシー分離、checkpoint運用
- 実行隔離：リアルタイム経路に影響させない
- 優先度：critical > important > research
- 資源逼迫時：researchの自動停止/縮退
- Anti-Thrash：クールダウン（例外は承認）

**TODO**
- job_specのkind一覧、resource表現（CPU/GPU/メモリ/時間）、cost_budget単位/上限の正式定義。

---

## 6. Time Discipline（時間規律）
### P5
- event_time / recv_time / processing_time を分離・混在禁止
- 例外カレンダー（メンテ/停止/休場/DST等）を dataset_card に同梱
- 混在/同期ズレはリーク同等の致命としてGate reject

**TODO**
- dataset_card内の例外カレンダー表現（フォーマット/粒度/優先順位）。

---

## 7. Data & Feature Governance（データ・特徴量）
### P6
- feature_set_id / feature_transform_hash / snapshot固定
- leakage検知（時系列/異常相関/time基準/train-inference整合）
- skew検知（致命）
- マルチ時間軸整合（致命）
- バイアス検知（生存者/選別/ラベル互換）
- poisoning耐性（健全性スコア・隔離・Gate反映）
- Do Not Train Rules：品質/汚染/欠損/カレンダー異常→学習せず triage_pack生成
- Source Drift Gate：データソース変更検知→Impact Graph→隔離/再評価提案

**TODO**
- 健全性スコアの算出要素、閾値の所在（metric_registry_refか別SSOTか）を参照で明確化。

---

## 8. Label / Metric / Objective SSOT（定義固定）
### P7（Label Factory）
- label_definition_version固定（label_definition_ref）
- 遅延ラベル/訂正：差分適用＋監査ログ
- version不一致比較は無効（report明示）

### P8（Metric Registry）
- 指標名/定義/算出/単位/閾値/用途を固定（metric_registry_ref）
- 同名異義禁止（CI検知）
- 重要指標は二重計算Cross-check必須（不一致は致命）

### P9（Objective）
- objective_refとして合成スコア固定
- HPO/WF/昇格はobjective_refに従う（変更は承認＋監査）

---

## 9. Evaluation Suite（評価）
### P10
- CV：時系列CV / purged / embargo、WF互換
- 取引視点評価：microstructure（約定率/キャンセル率/板/スプレッド）
- Fill/Cost Model SSOTに基づく評価（手数料/スプレッド/滑り、約定確率）
- Gate群：Overtrading/Churn、Maintenance-aware、有意性、回帰検知、Latency sensitivity 等
- Model Unit Tests：
  - 欠損/範囲外→NO-TRADE
  - NaN→fail
  - 偏り検知

**TODO**
- 各Gateの閾値・判定基準の正本（metric_registry_ref / evaluation_policy_ref 等）への参照を追加。

---

## 10. HPO / RL / LLM（最適化・提案）
### P11（HPO/探索）
- 探索上限（試行/コスト/危険行動/権限/外部送信）
- params_ref危険差分をGate reject
- 重要変更は危険差分ハイライト

### P12（RL）
- policy_refでclip/禁止、危険行動率で停止
- reward hacking / 探索暴走検知
- Policy-conditioned training
- offline既定、online隔離（要承認）

### P13（LLM）
- Decision Envelope（prompt/context/evidence/gate/approval）
- 逸脱（キー要求/外部送信/実行指示）検知→破棄＋監査
- 生成AI関与メタ（code_ref）→追加テスト要求
- triage_packに基づく推奨アクション定型生成（実行は承認経由）

---

## 11. Inference Contract / Quarantine（推論契約と隔離）
### P14（Inference Contract）
- I/Oスキーマ、SLO、NO-TRADE縮退、confidence
- 互換性：compatible / additive / breaking（breakingは追加承認＋追加テスト）
- Shadow-Contract Sync：実データで契約遵守・縮退を定期検証

### P15（Quarantine Inference）
- 契約外/OOD/異常入力 → NO-TRADE + Quarantine
- quarantine率増加 → drift/汚染/仕様変更疑い → triage_pack発火
- 本番影響ゼロで解析可能

**TODO**
- OOD判定の実装方式/参照SSOT（policy_refかinference_contract_refか）を明確化。

---

## 12. Gate / Promotion / Registry（昇格と配布）
### P16（Model Gate）
- 判定項目（okフラグ群）で合格のみ候補化
- rejected：理由＋次実験提案（report_ref）
- 期限付き例外：scope/理由/期限/監査必須
- 研究→本番：二重ゲート（追加テスト＋追加承認）

### P17（Promotion Lock）
- 昇格時ロック対象：
  - dataset snapshot / feature snapshot / code_ref / params_ref
  - inference_contract_ref / label_definition_ref / metric_registry_ref
  - objective_ref / fill_cost_model_ref
- ロックなし昇格は禁止

### P18（Model Registry）
- 署名/ロード時検証（hash/署名/互換性）
- 適用範囲（有効/禁止）
- 配布形態（shadow/paper/forward/live）＋ロールバック
- TTL：期限でshadow再評価or再学習提案
- staleness_score：腐敗検知

---

## 13. Drift / Circuit Breaker / Impact / Dependency（運用安全）
### P19（Drift & Circuit Breaker）
- 急性＋慢性（Slow Drift）監視、stalenessも加味
- 異常連鎖条件（quarantine率/NO-TRADE率/極端出力率/drift）で：
  - live提案/昇格を停止
  - shadow/paperのみ継続
  - triage_packをcritical生成

### P20（Impact Graph）
- SSOT変更（metric/label/contract/feature_set/fill_cost_model/docs）で：
  - 影響する model_ref/run_id/report_ref を自動列挙
  - impact_report_ref生成、再評価/再学習提案リスト作成

### P21（Dependency Drift / Migration Playbook）
- env_fingerprint差分で依存更新検知
- 再現性が崩れる更新は昇格禁止
- 重大更新：migration_playbook_ref必須（手順＋ロールバック＋検証）

---

## 14. Change Control / Accountability / Retention / Compliance
### P22（Approval Safety）
- 差分レポート（data/feature/code/params/gate）＋危険差分ハイライト
- breaking変更、scope/期限、例外承認：二重確認
- 緊急承認テンプレ（理由必須）
- 承認ログ固定（監査耐性）

### P23（Cards / Release Notes）
- Model Card：用途/範囲/弱点/リスク/Gate結果/SLO/縮退/互換性/TTL/廃止条件
- Dataset Card：取得元/期間/欠損遅延偏り/除外/リーク対策/汚染検知/例外カレンダー
- Release Note：変更点、効果、リスク、適用範囲、ロールバック手順

### P24（Retention）
- 期限/容量を定義しつつ、再現性維持の要約（fingerprint/統計/card/report/impact）は必ず保持
- “消したら再現不能”禁止

### P25（軽量コンプライアンスログ）
- 判断根拠の最低限ログ
- LLM利用/外部送信の有無
- 情報種別（公知/非公知/推測）の分類ログ（分類のみ）

---

## 15. DoD（完成条件）
### P（DoD Final）
- Time Discipline / Label Factory / Metric Registry / Objective / Fill-Cost がSSOT固定
- Gateでリーク/スキュー/汚染/遅延ラベル/約定成立性/回転過多/有意性/回帰/契約遵守を自動判定
- Promotion Lockなし昇格不可、研究→本番は二重ゲート
- Quarantine＋Circuit Breaker＋Do Not Trainで異常時はNO-TRADE
- Impact GraphでSSOT更新の影響範囲と再評価対象が自動抽出
- 依存更新は安全化、移行プレイブックで長期運用が破綻しない
- Card/Release Note/監査ログで説明責任と引継ぎが成立
