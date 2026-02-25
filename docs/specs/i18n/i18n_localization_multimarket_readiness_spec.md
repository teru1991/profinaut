# Internationalization / Localization / Multi-market Readiness Core Spec v1.0（固定仕様）
Time zones / locale-safe formatting / currency & units / calendars / legal jurisdiction awareness

- Document ID: I18N-LOCALE-MULTIMARKET-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): Y（I18N / Localization）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Data Catalog/Lineage: `docs/specs/data_governance/data_catalog_lineage_spec.md`
  - Reporting truth: `docs/specs/reporting/reporting_dashboard_explainability_spec.md`
  - Tax/Compliance: `docs/specs/tax_compliance/tax_compliance_legal_reporting_spec.md`
  - Equities IR: `docs/specs/equities_ir/equities_ir_financials_ingestion_scoring_spec.md`
  - FX/Macro: `docs/specs/fx_macro/fx_macro_ingestion_normalization_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
- Policy separation（固定しない）:
  - 対応言語、表示ルール、休日カレンダーの採用、通貨表示の慣習 → `docs/policy/**`
  - 運用手順（翻訳更新、用語統一、地域別出力）→ `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
多市場対応は「翻訳」よりも「意味を壊さない」ことが重要。
本仕様は、時刻・数値・通貨・単位・カレンダー・法域を **安全に扱い**、誤認・誤計算・誤提出を防ぐ不変条件を固定する。

必達要件（固定）：
1) **UTC truth**：内部の真実はUTCで保持し、表示のみローカルへ変換する
2) **Locale-safe formatting**：数値/通貨/単位/小数点/区切りをロケールで誤解させない
3) **Currency/unit explicit**：通貨・単位は常に明示され、暗黙変換しない
4) **Calendar aware**：市場休日/サマータイム/発表時刻精度を扱える
5) **Jurisdiction aware**：税・コンプラ報告は法域と規則版を明示し、混同しない
6) **Stable identifiers**：tickers/issuer_id/series_key 等は表記揺れで壊れない
7) **Evidence-linked output**：レポート/出力は replay_pointers で入力・規則版に辿れる
8) **No silent conversion**：勝手に丸め・通貨換算して“正しい風”に見せない

---

## 1. 範囲（in / out）
### 1.1 In
- タイムゾーン/日時表示ルール
- 数値・通貨・単位の表示/丸め/桁区切り
- 言語（UI文言、用語）と用語統一
- 市場カレンダー（休日、取引時間、DST）
- 法域（jurisdiction）・規則版の明示
- 多市場識別（ticker/ISIN/法人番号等）の扱い
- 監査/再現に必要な出力メタ（evidence refs）

### 1.2 Out
- 実際の翻訳文言の作成（Policy/運用）
- UIデザイン詳細
- 国別法解釈（Tax/Compliance側）

---

## 2. Time semantics（固定）
### 2.1 UTC as canonical（固定）
- すべての内部イベント時刻はUTCで保持
- ローカル表示は “表示層” のみで変換
- 変換時は `timezone` を明示（例：Asia/Tokyo, America/New_York）

### 2.2 Precision（固定）
- 発表時刻・予定時刻は precision（exact/approx/unknown）を持てる
- precisionが低いものを高精度として表示しない（品質注記）

### 2.3 DST（固定）
- DSTのある地域は IANA timezone を基準に変換
- 手動オフセット固定は誤差の原因なので禁止（例外はPolicyで運用するが、出力に明示）

---

## 3. Number formatting（固定）
### 3.1 Explicit units（固定）
- 数値は必ず unit を持つ（%, JPY, USD, shares, contracts, index points 等）
- 省略表示（K/M/B等）をする場合でも、元のunitを辿れる

### 3.2 Locale-safe separators（固定）
- 小数点/桁区切りはロケールに合わせるが、誤読を避ける：
  - 重要画面/提出物では “明示的表記” を優先（例：1,234.56 USD）
- 表示丸めは Policy だが、丸めた事実を隠さない（tooltip/注記）

---

## 4. Currency（固定）
### 4.1 Currency explicit（固定）
- すべての金額は currency code（ISO 4217 等）を持つ
- 暗黙の通貨換算は禁止（換算するなら “換算した” と明示）

### 4.2 FX conversion evidence（固定）
換算を行う場合（税・レポート等）：
- 使用したFX series_key
- 使用したレートの時刻/窓
- policy snapshot ref（採用戦略/丸め）
を replay_pointers で参照できる（Evidence-linked）

---

## 5. Multi-market identifiers（固定）
### 5.1 Stable internal IDs（固定）
- 外部表記（ticker, symbol）は揺れるため、内部は安定IDを使う：
  - equities: issuer_id
  - macro: series_key
  - crypto: normalized symbol (UCEL)
  - onchain: chain_id + address

固定ルール：
- 表記が変わっても内部IDは不変
- 表記は表示層でローカライズ可能（ただし内部計算は内部ID）

---

## 6. Calendars（固定）
### 6.1 Market calendar（固定）
- 取引所/市場の休日・取引時間を扱える
- 祝日や取引時間が変わる場合は “policy/plan” として版管理できる
- 時間に依存する集計（週次/月次）は calendar に従う（曖昧にしない）

### 6.2 Event calendar（固定）
- 経済指標発表・決算発表のスケジュールは scheduled_at_utc と precision を持つ
- “予定” と “実績（actual release time）” を区別できる

---

## 7. Jurisdiction awareness（固定）
### 7.1 Reporting jurisdiction（固定）
税・コンプラ報告は必ず：
- jurisdiction
- regime_id / regime_version
- policy snapshot ref
を表示/出力に含める。

### 7.2 Mixed-jurisdiction prevention（固定）
- 異なる法域の報告物を混在させない（出力単位で分離）
- 混在が疑われる場合は quality=UNKNOWN とし提出前に止める（運用で補完）

---

## 8. Language & Terminology（固定）
### 8.1 Canonical terminology（固定）
- 用語の正本は `docs/specs/system/terminology.md`
- UI翻訳はその意味を変えない（直訳で意味が壊れるなら注釈）

### 8.2 Locale fallbacks（固定）
- 対応言語がない場合の fallback を定義できる（Policy）
- fallback時は “未翻訳” を隠さない（誤認防止）

---

## 9. Evidence-linked outputs（固定）
重要な出力（レポート/税/スコア）は replay_pointers で参照できる：
- 入力範囲（dataset refs）
- 規則版（regime_version）
- 変換ルール（policy snapshot）
- 出力対象ロケール（language/timezone）

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) 内部時刻がUTCで保持され、表示変換はIANA timezoneで行われる
2) 数値/通貨/単位が明示され、暗黙換算が起きない
3) DST/休日カレンダーに基づく表示・集計が可能
4) 法域/規則版が出力に明示され、混同が起きない
5) 内部IDが表記揺れで壊れない
6) 出力が replay_pointers で証拠参照できる

---

## 11. Policy/Runbookへ逃がす点
- 対応言語、表示ルール、休日カレンダー採用、通貨表示慣習
- 翻訳更新手順、用語統一運用
→ Policy/Runbookへ（意味は変えない）

---
End of document
