# UCEL Golden Compatibility Notes v1.0（固定）
Document ID: UCEL-GOLDEN-COMPAT-NOTES
Status: Canonical / Fixed Contract
Scope: profinaut内のUCEL境界仕様（docs/specs/ucel/**）と、UCEL Golden（docs/specs/ucel_golden/**）の関係を固定する

## 0. 目的（Non-negotiable）
UCELは「利用境界仕様（このリポの実装と接続）」と「思想/正本（Golden）」が混線しやすい。
本書は、どちらを正とするかをケース別に固定し、SSOT分裂を防ぐ。

## 1. 正本の原則
- UCEL Golden（docs/specs/ucel_golden/** + docs/policy/ucel_golden/** + docs/plans/ucel/**）:
  - UCELの思想・不変要件・運用値の出典・フェーズ計画の正本
- profinaut UCEL（docs/specs/ucel/**）:
  - profinautがUCELをどう使うか（connector/boundary）の正本

固定ルール:
- 同じ主題を二重に書かない（Goldenとprofinaut境界で役割分離）

## 2. 参照の決め方（ケース別）
### 2.1 “不変要件（契約）” を参照したい
- 正本: docs/specs/ucel_golden/ucel_golden_core_spec.md

### 2.2 “運用値（しきい値/保持/上限）” を参照したい
- 正本（原文）: docs/policy/ucel_golden_policy_pack.md
- 正本（実装向け）: docs/policy/ucel_golden/README.md（TOML索引）

### 2.3 “計画（フェーズ/進捗/順序）” を参照したい
- 正本: docs/plans/ucel/ucel_implementation_plan_phase_playbook.md

### 2.4 “このリポでどう接続するか” を参照したい
- 正本: docs/specs/ucel/**（connector specs）

## 3. 変更のルール
- Golden Core の意味を変える変更は最重（SemVer major/minorの判断）
- 運用値の変更は policy として扱い、Core SemVer を動かさない
- profinaut側のconnector変更は、このリポの実装境界変更として扱う（影響範囲を明記）

## 4. DoD（検証）
最低限:
1) UCEL Golden入口（docs/specs/ucel_golden/README.md）から全て辿れる
2) profinaut UCEL（docs/specs/ucel/**）とGoldenの役割がREADMEで分かる
3) Goldenとprofinaut境界の重複SSOTが増殖していない
