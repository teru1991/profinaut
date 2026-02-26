# Level 1 SSOT Outline — R. Equity/IR Analytics（株：決算/IR・格付け・検索）v1.8（固定）

## 0. メタ
- ドメイン文字: R
- 対象: 株式（JP/US想定。将来拡張可）
- 目的: 開示/IR/決算情報の収集→構造化→指標計算→独自スコア（格付け）→検索→アラートを一気通貫で提供
- 固定範囲: 本体（0〜5章）＋ Appendix A（R1〜R15）＋ Appendix B（R16〜R20）＋ Annex-RX（RX-01〜RX-38）
- 変更ポリシー: 原則「追加のみ」（意味変更・削除禁止）。破壊変更は Appendix-R14 に従う。

---

## 1. Non-negotiable（目的と到達点）
### 1.1 目的
- R は株式の開示/IR/決算情報を確実に収集・構造化し、指標計算・独自スコアを再現可能に生成し、スクリーニング、イベント/テキスト検索、アラートを提供する。

### 1.2 到達点（完了定義）
- 収集→構造化→正規化→指標→スコア→検索→アラートが一気通貫で動作
- 訂正/差替え/取り下げに耐え、最新版と履歴を保持、派生物が自動再計算
- 出典（provenance）と計算再現性（versioned compute）担保
- 検索品質（日本語/表記揺れ/同義語/否定/ランキング）が実用域
- 通知疲れ対策（重複抑制・重要度・静穏時間・解除条件）組み込み
- 欠損・遅延・解析失敗がSLO超過したら検知/通知/復旧導線
- Point-in-Time（過去時点再現）が成立（研究/検証で破綻しない）
- 市場カレンダー/時刻整合（JP/US、DST）込みでイベント時刻が一貫
- セキュリティ/プライバシー（関心銘柄・条件・通知先）保護
- 説明品質（Explainability SLA）と改善ループ（監査付き）が成立
- 製品として破綻しない（Fail-safe、SLO/SLI、互換性、ライフサイクル、支援束）
- Deterministic Rules（固定則）と移行手順固定により実装・運用で割れない
- 受け入れ/コーパス/性能予算/リリース/検証証拠が固定され、確実に完成できる
- 機能拡張（Annex-RX）は本体を壊さず追加できる

---

## 2. スコープ（責務 / 非責務）
### 2.1 R の責務（Rが“持つ”）
- 収集：開示/IR/決算ドキュメント・メタ情報取得（定期/イベント駆動/バックフィル）
- 構造化：本文/表/数値抽出、単位正規化、訂正統合、信頼度推定
- 正規化モデル：Filing/Statement/KPI/Guidance/Event/TextIndex の canonical 化
- 指標計算：財務指標算出、期間変換、欠損戦略、再計算トリガ、versioned compute
- 独自スコア（格付け）：説明可能な因子、セクター補正、不確実性抑制
- 検索：数値/イベント/全文（日本語）検索、保存検索、再現可能ランキング
- アラート：イベント/条件/異常、重複抑制、重要度、静穏時間、監査、解除条件、dry-run
- CA・株式数時系列：希薄化/自社株/分割併合/配当等の正規化と検索/通知
- 決算カレンダー・予実・サプライズ：予定（可能範囲）と予実突合、差分保持
- Point-in-Time & ユニバース統治：当時見えていた情報のみで再現可能
- 市場カレンダー・時刻意味論：休場/DST/市場TZとUTC整合、effective_trading_date
- 規約メタ：取得頻度・保存/再配布可否・引用要件など保持
- 品質ゲート：会計整合・異常値・桁ズレ・訂正差分異常を検知/隔離
- パーサ回復：フォーマット変更耐性、回帰テスト、source自動降格
- セキュリティ/プライバシー：認可、レート制限、監査マスキング
- マルチテナント準備：tenant/workspaceで保存物・辞書・通知を分離
- HITL：triageキュー、手動補正パッチ、監査・再現性
- 安全な保持/再構築：Retention、スロットリング、partial index、運用継続
- 安定イベント契約：schema_version / idempotency_key / is_informational
- テスト戦略：コーパス、契約、回帰、負荷
- 説明SLA/改善ループ：説明最低要件、フィードバック、変更影響評価
- Cross-source突合：不一致保持と差異イベント化
- Freshness/Staleness：鮮度表示と抑制
- リンク死耐性：raw_refsフォールバック
- UX契約：Timeline/Explain/Screenのpayload要件
- Fail-safe挙動：品質低下時の降格/抑制/品質イベント
- SLO/SLI契約：何を測るか固定
- データライフサイクル：保管/圧縮/アーカイブ/例外削除
- 互換性ポリシー：SemVer適用範囲
- 安全（非助言）ルール：スコア/アラート意味論固定
- Support Bundle：再現導線の固定

### 2.2 R の非責務（ただし連携必須）
- 売買実行（I Execution）／リスク統制（J Risk）
- 監視基盤（C Observability）※Rはメトリクス/ログを出す
- 契約統治（G Data Contracts）※Rはスキーマ提供・Gate対象
- 監査基盤（O Audit/Replay）※Rは根拠参照IDを残す

---

## 3. 本体仕様（固定）
### 3.1 UX Contract（Timeline / Explain / Screen）
- Timeline Payload
  - filings（公開/訂正/取り下げ）＋ events（決算/修正/CA/差異/品質）＋ score snapshots を時系列統合
  - 各イベントに semantic_type / severity / root_refs（根拠束）必須
- Explain Payload
  - factors寄与 + explain_refs（filing/kpi/provenance） + diff_refs + hit_snippets（位置情報）
  - freshness / confidence / missingness_impact を必ず添付
  - SLA未達は explainability_grade を下げ、重要度を自動降格可能
- Screen Payload
  - screen_definition（条件正規化）＋ rule_hits（なぜヒット）＋根拠束
  - 保存/通知のため saved_object_schema_version を返す

### 3.2 Fail-safe Semantics
- 解析品質崩壊時：スコア降格／条件アラート抑制／quality_event生成
- 再構築中：全文は旧継続を原則、数値はpartial警告または停止、通知はdry-run優先

### 3.3 SLO/SLI Contract（測る指標固定）
- 収集：成功率、遅延p95、欠損率、429率
- 解析：失敗率、confidence分布、異常値率、差分異常率
- 検索：応答p95、タイムアウト率、partial率
- 通知：配信成功率、重複率、誤通知率（フィードバック由来）、抑制率
- 品質：突合差異イベント件数、staleness比率

### 3.4 Data Lifecycle Policy
- Raw肥大化前提：hash重複排除・圧縮/アーカイブ（層別）
- Derived/Index は rebuild 前提で短期可
- 原則削除なし（監査）。規約必須時のみ例外削除手順（監査・影響評価・再現不能の明示）

### 3.5 Compatibility Policy（SemVer scope）
- 最優先の後方互換：event stream（schema_version）/ SavedObject（schema_version）
- KPI/Scoreは compute_version で互換担保
- 破壊変更ゲート：影響評価、回帰コーパス、移行手順、監査ログ

### 3.6 Safety & Non-advisory Rules
- スコア/アラートは情報（推奨禁止）
- 注意喚起条件（低confidence、低freshness、差異、partial等）を固定
- 下流にも is_informational=true を原則

### 3.7 Support Bundle Spec
- raw_refs（hash含む）、取得メタ、parseログ、provenance一覧、diff_refs、terms_ref、compute inputs、index状態、再現導線

---

## 4. データモデル（最小固定スキーマ：概念）
- Canonical（事実）: issuer / ticker / filing / statement / guidance / event / terms / provenance
- Derived（計算・推定）: kpi / score / index / summaries / rankings / テーマ / impact 等
- User scope（tenant/workspace）: saved objects / alerts / dictionaries / feedback / notes / endpoints
- 詳細な表現固定: Appendix-R1〜R15 に従う（※ここでは概念のみ）

TODO: Appendix-R1〜R15 の「詳細スキーマ（フィールド定義・型・必須/任意・制約）」は別途参照元に明記がないため、Level 2 で整理しつつ TODO を残す。

---

## 5. 完了条件（DoD）
- 本体仕様（第3章）を満たす
- Appendix-R1〜R20 の固定ルールを満たす
- 検証証拠は Appendix-R20 に従って docs/verification/ に残す

---

## 6. SSOTとの整合（Rの最小要件）
- 開示/IR/決算情報の収集・構造化
- 指標計算と独自スコア（格付け）
- 任意条件スクリーナー、イベント/テキスト検索
- アラート（決算、修正、増資等）

---

## 7. Deterministic Rules（Appendix A：R1〜R15）
> 変更禁止（追加のみ）。実装/テスト/運用/移行は必ず本Appendixに従う。

### 7.1 Appendix-R1: Stable IDs & Dedupe Keys（安定IDと重複鍵）
- R1-1 原則：IDは安定性優先、生成は正規化済み入力のみ（R11参照）
- R1-2 issuer_id：issuer_key（jurisdiction + legal_identifier(可能なら) + normalized_issuer_name + valid_from）から生成／社名変更では不変／統合分割はR1-6
- R1-3 ticker_id：衝突前提で market + code + valid_from + valid_to 必須／結合は表示コードでなく ticker_id（R12）
- R1-4 filing_id：source + issuer_id + doc_type + period + disclosed_at(なければpublished_at) + primary_raw_hash
- R1-5 event_id / dedupe_key：event_id=semantic_type + issuer_id + effective_at + related_filing_id + key_fields_hash／dedupe_keyはsemantic_typeごと固定式（変更禁止）
- R1-6 merge/split履歴：mergeは新issuer_id、旧に successor_issuer_id／splitはpredecessor残し新issuer作成／法的同一性優先

### 7.2 Appendix-R2: Numeric Canonicalization（数値固定）
- float禁止（整数最小単位 or 高精度Decimal）
- 計算は丸めない（丸めは表示専用）
- 比率は 0..1 のDecimal（%は表示）
- △/括弧/赤字は符号付きへ正規化
- unit_scale必須、桁ズレ検知は品質ゲート必須

### 7.3 Appendix-R3: Period Semantics（期間固定）
- period正規形：FY{YYYY} / FY{YYYY}-Q{1..4} / TTM@{as_of_date}
- issuerに fiscal_year_start_month 保持
- 期ズレは period生成時に吸収、検索は正規period
- TTM欠損期：既定は計算不能（missingness_impactに理由）。推定はDerived（R8）

### 7.4 Appendix-R4: Revision Application Policy（訂正適用/再通知）
- 既定表示/検索：latest（最新版）
- as_of指定：当時値（point-in-time）
- 再通知既定：重大影響は再通知、軽微は通知せずTimelineに必ず記録
- 値が戻る訂正も差分要約・理由保持、重要度再評価

### 7.5 Appendix-R5: Text Extraction Determinism（抽出決定性）
- 同一raw_hash→常に同一テキスト（非決定性禁止）
- 正規化規則はR11
- スニペットは位置情報（offset/ページ/座標/表IDのいずれか）必須

### 7.6 Appendix-R6: Error Code Taxonomy（reason_code体系）
- reason_codeは後方互換（追加のみ、意味変更禁止）
- ingest/parse/compute/index/notify で固定列挙
- reason_codeに severity と対応（retry/降格/隔離/人手）を紐づけ

### 7.7 Appendix-R7: Accounting/Segment Consistency（整合ルール）
- 会社開示値がある場合は開示値優先（計算値は補助）
- セグメント合計≠全社合計：差額を reconciliation_gap として明示・タグ化

### 7.8 Appendix-R8: Canonical vs Derived Boundary（正本/派生境界）
- Canonical：観測された事実（filing/抽出値/メタ/出典）のみ
- Derived：計算・推定・要約・分類・ランキング・AI分類すべて
- 推定/解釈のCanonical混入は禁止

### 7.9 Appendix-R9: AI Usage Boundary & Versioning（AI境界）
- AI利用はDerived限定
- model_id / prompt_hash / params_hash / generated_at 記録
- 非決定性前提で再生成可能にする
- 根拠は常に filing/provenance（AI結論を根拠にしない）

### 7.10 Appendix-R10: Time Truth Ordering（時刻優先順）
- 基準時刻：disclosed_at > published_at > fetched_at
- as_of_visible_at（システムが知った時刻）必須

### 7.11 Appendix-R11: Japanese Text Normalization（日本語正規化）
- UTF-8
- NFKC、全角半角統一、連続空白圧縮、改行統一
- 絵文字/機種依存：保存はする、索引は固定規則で除去/置換
- 正規化前の原文はrawで保持

### 7.12 Appendix-R12: Identity Collision Handling（同一性衝突）
- ticker再利用・市場再編を前提
- ticker_idは (market, code, valid_from, valid_to) 必須
- 結合は安定ID（issuer_id/ticker_id）
- 表示コードはUI用（同定に使わない）

### 7.13 Appendix-R13: Dependency Degradation（依存停止時）
- 拡張依存（価格/ニュース等）が落ちてもRは稼働（拡張のみdegrade）
- payloadに degraded_dependencies[]
- 根拠不足/partial中は fail-safe で降格/抑制

### 7.14 Appendix-R14: Mandatory Migration Procedure（移行手順）
- 破壊変更は原則禁止。必要な場合は必ず:
  1) 旧新両対応期間
  2) backfill + verify
  3) 影響評価
  4) 回帰コーパス通過
  5) feature flag切替
  6) 監査ログに理由・手順記録

### 7.15 Appendix-R15: Exception Event Types（例外イベント型）
- 開示遅延/サイト障害/添付欠落/ページ差替え等は例外イベントとして型固定
- reason_code と影響（どの機能がdegradeしたか）必須
- Timelineに必ず記録

---

## 8. Completion Guarantee（Appendix B：R16〜R20）
### 8.1 Appendix-R16: Acceptance Test Catalog（受け入れテスト：固定）
- R16-T01 基本E2E：決算→KPI→Score→Screen→Alert（根拠束付き）
- R16-T02 訂正連鎖：遡及修正→diff_refs→再計算→再通知ポリシー
- R16-T03 CA：分割/自社株/増資→株式数時系列→ca_event→検索ヒット
- R16-T04 Point-in-Time：as_ofで当時値再現（当時可視filingのみ）
- R16-T05 フォーマット変更：fixtures回帰→自動降格→quality_event→fail-safe
- R16-T06 欠損/異常値：隔離→triage→manual_override→再処理
- R16-T07 規約メタ/429：rate制御→429→retry/降格→復旧（terms_ref監査）
- R16-T08 検索品質：日本語揺れ/同義語ONOFF/否定/フレーズ/位置スニペット
- R16-T09 Downstream契約：schema_version/idempotency_key で二重処理防止

### 8.2 Appendix-R17: Golden Corpus Definition（コーパスSSOT）
- raw_refs hash で固定（業種×資料タイプ×例外含む）
- コーパス保持（期待抽出/期待イベント/期待スコア/例外）

### 8.3 Appendix-R18: Performance/Capacity Budget（性能・容量予算：項目固定）
- 収集：コネクタ別頻度上限、同時取得数、リトライ上限
- 解析：同時パース数、Critical Queue枠
- Index：全文/数値の再構築許容時間、partial許容率
- 検索：p95応答、タイムアウト率上限
- 通知：p95配信遅延、重複率上限
- 保存：Raw年次増加想定（1年/3年）、Retention検証項目

### 8.4 Appendix-R19: Release Playbook（リリース手順：固定）
- feature flag段階：shadow → partial → full
- 互換性ゲート必須：契約テスト、回帰コーパス、影響評価
- ロールバック条件（項目固定）：SLI悪化、品質イベント急増、重大整合性破綻

### 8.5 Appendix-R20: Verification Evidence Format（検証証拠：固定）
- docs/verification/ に最低限残す:
  - 受け入れテスト結果（R16対応）
  - fixturesハッシュ一覧（R17証明）
  - 差分ログ（revision impact/diff typing）
  - SLO/SLIスナップショット（収集/解析/検索/通知）
  - 互換性ゲート通過記録（SemVer/contract tests）
  - 影響評価レポート（変更前後差分）

---

## 9. Annex（機能拡張：Annex-RX v1.2-Complete）
### 9.1 Annex-RX-0 位置づけ（固定）
- R範囲内の追加機能列挙
- 追加機能は原則Derivedで価値提供（Canonical汚染禁止：R8）
- Explainability / Point-in-Time（可能範囲）/ tenant分離 / fail-safe / versioning を満たす

### 9.2 Annex-RX-1（一次：王道12）
- RX-01 企業比較（Peer Compare）
- RX-02 変化点ダッシュボード（Delta Hub）
- RX-03 重要度推定（Event Impact Scoring）
- RX-04 重要語彙トリガー（Risk Lexicon Triggers）
- RX-05 テーマ抽出（Topic/Theme Index）
- RX-06 ガイダンス確度（Guidance Confidence）
- RX-07 訂正影響ビュー（Revision Impact View）
- RX-08 IR Q&A 取り込み（IR Q&A）
- RX-09 シナリオ監視（Scenario Monitoring）
- RX-10 研究パック出力（Research Pack）
- RX-11 クエリビルダー補助（Query Builder Assist）
- RX-12 監査可能なメモ/注釈（Analyst Notes）

### 9.3 Annex-RX-2（二次：漏れやすい実務15）
- RX-13 多段ユニバース（Universe Builder）
- RX-14 決算ブリッジ（要因分解）
- RX-15 KPI一貫性チェック（矛盾検知）
- RX-16 KPI Discovery（独自KPI候補発見）
- RX-17 ガイダンス的中度ダッシュボード
- RX-18 アラート説明テンプレ（Readable Alert Composer）
- RX-19 Diff Typing（差分種類分類）
- RX-20 Event Chain（イベント連鎖束ね）
- RX-21 キーワードアラート精度改善（辞書提案）
- RX-22 Severity Calibration（重要度校正）
- RX-23 Smart Period Picker（比較期間提案）
- RX-24 IR資料スライド構造化
- RX-25 重要提出物優先キュー（Critical Filing Queue）
- RX-26 Question Mining（想定問答抽出）
- RX-27 トーン変化モニタ（言い回し変化）

### 9.4 Annex-RX-3（三次：v1.2追補 11）
- RX-28 比較レポート生成（Comparison Report）
- RX-29 イベント逆引き探索（Event→Entities Explorer）
- RX-30 文書異常検知（Filing Anomaly Detector）
- RX-31 簡易財務モデル（Light Financial Model）
- RX-32 通知最適化（Alert Optimization）
- RX-33 共有サブスクリプション（Shared Subscriptions）
- RX-34 辞書一括管理（Dictionary Manager）
- RX-35 原因タグ付け（Cause Tagging）
- RX-36 企業定例KPIボード（Issuer KPI Board）
- RX-37 一括再評価（Bulk Re-score / Bulk Re-index）
- RX-38 教師データ出力（Labeling/Training Export）

### 9.5 Annex-RX-4 共通要件（追加機能の縛り：固定）
- Canonical/Derived境界（推定・要約・分類・ランキングはDerived）
- Explainability SLA（根拠束必須）
- Point-in-Time（可能範囲でas_of対応）
- tenant/workspace分離＋監査マスキング
- fail-safe（品質低下/partial/根拠不足は降格・抑制＋quality_event）
- versioning（辞書/保存物/テンプレ/ルール/モデルは版管理＋影響評価）

### 9.6 Annex-RX-5 推奨優先順位（固定）
- 最優先：RX-02 / RX-03 / RX-07 / RX-10 / RX-01 / RX-32 / RX-34 / RX-30 / RX-36
- 次点：RX-04 / RX-14 / RX-25 / RX-24 / RX-28 / RX-29 / RX-37 / RX-38
- 拡張：RX-05 / RX-16 / RX-17 / RX-20 / RX-27 / RX-31 / RX-35 / RX-33

---

## 10. Capability Index（ID保持・索引）
### 10.1 Deterministic Rules（Appendix A）
- R1-1, R1-2, R1-3, R1-4, R1-5, R1-6
- Appendix-R2
- Appendix-R3
- Appendix-R4
- Appendix-R5
- Appendix-R6
- Appendix-R7
- Appendix-R8
- Appendix-R9
- Appendix-R10
- Appendix-R11
- Appendix-R12
- Appendix-R13
- Appendix-R14
- Appendix-R15

### 10.2 Completion Guarantee（Appendix B）
- Appendix-R16: R16-T01, R16-T02, R16-T03, R16-T04, R16-T05, R16-T06, R16-T07, R16-T08, R16-T09
- Appendix-R17
- Appendix-R18
- Appendix-R19
- Appendix-R20

### 10.3 Extensions（Annex-RX）
- Annex-RX-0
- RX-01〜RX-38
