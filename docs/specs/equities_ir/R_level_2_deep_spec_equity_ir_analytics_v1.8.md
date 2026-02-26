# Level 2 Deep Spec — R. Equity/IR Analytics v1.8（整理のみ / 新仕様追加なし）

## 0. 適用範囲
- 本ドキュメントは Level 1 を “実装/検証に使える形” に粒度整理したもの。
- 新しい仕様は追加しない。不明点は TODO を残す。

---

## 1. システム境界（Rの責務の分解）
### 1.1 パイプライン（E2E）
1) Ingest（収集）
- 対象: 開示/IR/決算ドキュメント + メタ
- 方式: 定期 / イベント駆動 / バックフィル
- Terms/規約メタ保持（取得頻度、再配布可否、引用要件等）

2) Parse/Extract（構造化）
- 本文/表/数値抽出
- 単位正規化（Appendix-R2）
- 訂正統合（Appendix-R4）
- 信頼度推定（confidence）

3) Canonicalize（正規化モデル）
- Filing / Statement / Guidance / Event / TextIndex の canonical 化
- Canonical vs Derived 境界（Appendix-R8）

4) Compute（指標計算・スコア）
- 財務指標算出、期間変換（Appendix-R3）
- 欠損戦略（missingness_impactを明示）
- versioned compute（compute_version）

5) Index/Search（検索）
- 数値検索 / イベント検索 / 全文（日本語）検索
- 保存検索（saved objects）
- 再現可能ランキング

6) Alert（通知）
- イベント/条件/異常
- 重複抑制、重要度、静穏時間、解除条件
- dry-run対応
- 監査（通知決定の根拠束/抑制理由）

7) PIT（Point-in-Time）
- as_of 指定で “当時可視の情報のみ” で再現（Appendix-R4, R10）
- ユニバース統治（当時の同定/銘柄集合）

8) Ops（運用継続）
- Fail-safe（品質低下時の降格/抑制/quality_event）
- 再構築（全文旧継続、数値partial警告or停止、通知dry-run優先）
- Support Bundle による再現導線固定

TODO: 各ステップの入出力スキーマ（event stream / canonical entities / index docs）の具体フィールドは原文に“詳細固定”として Appendix-R1〜R15 参照とあるが、本文に展開されていないため未確定。

---

## 2. UX Contract（Timeline / Explain / Screen）
### 2.1 Timeline Payload（契約）
- 統合対象（時系列）:
  - filings: 公開 / 訂正 / 取り下げ
  - events: 決算 / 修正 / CA / 差異 / 品質
  - score snapshots
- 必須フィールド（各イベント）:
  - semantic_type
  - severity
  - root_refs（根拠束）

TODO: semantic_type の列挙、severity のスケール（例: int/enum）、root_refs の参照形（ID参照/URL/hash）を確定する必要あり（本文未記載）。

### 2.2 Explain Payload（契約）
- 必須:
  - factors 寄与
  - explain_refs（filing/kpi/provenance）
  - diff_refs
  - hit_snippets（位置情報必須：Appendix-R5）
  - freshness / confidence / missingness_impact
- Explainability SLA未達:
  - explainability_grade を下げる
  - 重要度の自動降格が可能

TODO: explainability_grade のグレード体系、SLA閾値、降格ルールの固定式は本文未記載。

### 2.3 Screen Payload（契約）
- 必須:
  - screen_definition（条件正規化）
  - rule_hits（なぜヒット）
  - 根拠束
  - saved_object_schema_version（保存/通知互換のため）

TODO: screen_definition の正規化ルール（単位/期間/比較演算/否定/同義語切替）詳細は本文未記載。

---

## 3. Fail-safe Semantics（固定挙動）
### 3.1 解析品質崩壊時
- スコア降格
- 条件アラート抑制
- quality_event 生成

### 3.2 再構築中
- 全文: 旧継続が原則
- 数値: partial警告 または 停止
- 通知: dry-run優先

TODO: 「崩壊判定（どのSLI/閾値で）」と「降格の段階（何段階）」は本文未記載。

---

## 4. SLO/SLI Contract（測定項目固定）
### 4.1 収集
- 成功率
- 遅延 p95
- 欠損率
- 429率

### 4.2 解析
- 失敗率
- confidence 分布
- 異常値率
- 差分異常率

### 4.3 検索
- 応答 p95
- タイムアウト率
- partial率

### 4.4 通知
- 配信成功率
- 重複率
- 誤通知率（フィードバック由来）
- 抑制率

### 4.5 品質
- 突合差異イベント件数
- staleness 比率

TODO: “SLO（目標値）”自体の数値は本文未記載（項目のみ固定）。

---

## 5. データライフサイクル（固定）
- Raw: hash重複排除、圧縮/アーカイブ（層別）
- Derived/Index: rebuild前提（短期可）
- 原則削除なし（監査）。規約必須時のみ例外削除手順（監査・影響評価・再現不能の明示）

TODO: Retention年限やアーカイブ階層の具体（ホット/ウォーム/コールド等）は本文未記載。

---

## 6. 互換性ポリシー（SemVer scope）
- 最優先の後方互換:
  - event stream（schema_version）
  - SavedObject（schema_version）
- KPI/Score:
  - compute_version で互換担保
- 破壊変更ゲート:
  - 影響評価、回帰コーパス、移行手順、監査ログ

参照: Appendix-R14（Mandatory Migration Procedure）

---

## 7. 安全（非助言）ルール（固定）
- スコア/アラートは情報（推奨禁止）
- 注意喚起条件（低confidence、低freshness、差異、partial等）を固定
- 下流にも is_informational=true を原則

TODO: “注意喚起条件”の具体閾値/テンプレは本文未記載。

---

## 8. Support Bundle（固定）
- 含める:
  - raw_refs（hash含む）
  - 取得メタ
  - parseログ
  - provenance一覧
  - diff_refs
  - terms_ref
  - compute inputs
  - index状態
  - 再現導線

TODO: Support Bundle のフォーマット（json/zip構造、フィールド名）は本文未記載。

---

## 9. Canonical / Derived / User Scope（概念整理）
### 9.1 Canonical（事実）
- issuer
- ticker
- filing
- statement
- guidance
- event
- terms
- provenance

### 9.2 Derived（計算・推定）
- kpi
- score
- index
- summaries
- rankings
- theme / impact 等

### 9.3 User scope（tenant/workspace）
- saved objects
- alerts
- dictionaries
- feedback
- notes
- endpoints

境界規則:
- Appendix-R8（Canonical vs Derived）
- Appendix-R9（AI利用はDerived限定）

TODO: 各エンティティの canonical schema は「Appendix-R1〜R15に従う」とあるが本文に展開がないため、別途“詳細表現”の参照が必要。

---

## 10. Deterministic Rules（Appendix A：要点を運用単位に再配置）
### 10.1 Identity/ID（Appendix-R1 / R12）
- R1-1〜R1-6（issuer_id / ticker_id / filing_id / event_id / dedupe_key / merge-split）
- Appendix-R12（ticker再利用・市場再編の衝突前提）

### 10.2 数値・期間（Appendix-R2 / R3）
- Appendix-R2（float禁止、unit_scale、桁ズレ検知）
- Appendix-R3（FY/Q/TTM、fiscal_year_start_month、TTM欠損の扱い）

### 10.3 訂正と時刻（Appendix-R4 / R10）
- Appendix-R4（latest / as_of、再通知ポリシー、差分要約）
- Appendix-R10（disclosed_at > published_at > fetched_at、as_of_visible_at必須）

### 10.4 テキスト決定性と正規化（Appendix-R5 / R11）
- Appendix-R5（raw_hash→同一テキスト、snippet位置必須）
- Appendix-R11（NFKC等、絵文字は保存/索引は除去or置換）

### 10.5 失敗/例外（Appendix-R6 / R13 / R15）
- Appendix-R6（reason_code後方互換、severityと対応）
- Appendix-R13（拡張依存停止でもR本体稼働、degraded_dependencies[]）
- Appendix-R15（例外イベント型固定、Timeline記録）

### 10.6 正本境界とAI（Appendix-R8 / R9）
- Appendix-R8（Canonical汚染禁止）
- Appendix-R9（model_id/prompt_hash/params_hash/generated_at、根拠はfiling/provenance）

### 10.7 移行（Appendix-R14）
- 旧新両対応期間、backfill+verify、影響評価、回帰コーパス、feature flag切替、監査記録

---

## 11. Behavior/Tests（Appendix B：固定カタログ）
### 11.1 Acceptance Tests（Appendix-R16）
- R16-T01〜R16-T09（E2E、訂正、CA、PIT、フォーマット変更、欠損/異常、429/規約、検索品質、下流契約）

### 11.2 Golden Corpus（Appendix-R17）
- raw_refs hashで固定
- 期待抽出 / 期待イベント / 期待スコア / 例外（訂正、取り下げ、添付欠落、桁ズレ等）

### 11.3 Performance/Capacity Budget（Appendix-R18）
- 収集/解析/Index/検索/通知/保存の“項目”固定

### 11.4 Release Playbook（Appendix-R19）
- shadow → partial → full
- 互換性ゲート必須
- ロールバック条件（SLI悪化、品質イベント急増、重大整合性破綻）

### 11.5 Verification Evidence（Appendix-R20）
- docs/verification/ への証拠出力要件（R16〜R20対応）

---

## 12. Annex-RX（拡張ユニット：整理）
### 12.1 位置づけ（Annex-RX-0）
- 追加機能は本体を壊さず追加
- Derived中心、Explainability/PIT/tenant分離/fail-safe/versioning準拠

### 12.2 機能群
- Annex-RX-1: RX-01〜RX-12
- Annex-RX-2: RX-13〜RX-27
- Annex-RX-3: RX-28〜RX-38
- Annex-RX-4: 共通要件（固定）
- Annex-RX-5: 推奨優先順位（固定）

TODO: 各RXの入出力（report_bundle、research_pack等）のスキーマは本文未記載。

---

## 13. Capability Index（ID一覧）
- R1-1, R1-2, R1-3, R1-4, R1-5, R1-6
- Appendix-R2, Appendix-R3, Appendix-R4, Appendix-R5, Appendix-R6, Appendix-R7, Appendix-R8, Appendix-R9, Appendix-R10, Appendix-R11, Appendix-R12, Appendix-R13, Appendix-R14, Appendix-R15
- Appendix-R16: R16-T01, R16-T02, R16-T03, R16-T04, R16-T05, R16-T06, R16-T07, R16-T08, R16-T09
- Appendix-R17, Appendix-R18, Appendix-R19, Appendix-R20
- Annex-RX-0
- RX-01〜RX-38
