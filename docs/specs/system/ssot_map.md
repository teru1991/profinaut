# SSOT Map — Docs OS Canonical Index v1.0（固定）
Document ID: SYS-SSOT-MAP
Status: Canonical / Fixed Contract
Scope: docs全体の「正本（SSOT）」「参照順序」「役割境界」を固定する

## 0. 目的（Non-negotiable）
本書は、docsが増えても **正が割れず、迷わず、再現性を保つ**ための “地図” である。
- どの情報がどこにあるかを 1分で辿れる
- SSOT分裂（同じ役割の文書が複数）を防ぐ
- Core（固定）/ Policy（運用値）/ Plan（計画）/ Runbook（手順）/ Legacy（隔離）/ Context（補助）を厳密に分離する

## 1. 必読の順序（Must-read order）
1) docs/README.md
2) docs/specs/system/ssot_map.md（本書）
3) docs/specs/system/versioning_policy.md
4) docs/contracts/README.md（存在すれば） + docs/contracts/*.schema.json（契約SSOT）
5) docs/specs/crosscut/*（横断固定仕様）
6) docs/specs/**（各ドメイン固定仕様）
7) docs/policy/README.md + docs/policy/**（運用値）
8) docs/plans/README.md + docs/plans/**（計画）
9) docs/runbooks/README.md + docs/runbooks/**（手順）
10) docs/decisions/decisions.md（意思決定ログ）
11) docs/status/trace-index.json（リンクSSOT）
12) docs/legacy/**（参考：正本ではない）
13) docs/context/**（AI/補助：正本ではない）

## 2. SSOTレイヤ（役割の固定）
### 2.1 Contracts（契約SSOT / 変更影響大）
- 正本:
  - docs/contracts/*.schema.json
- 説明:
  - docs/contracts/README.md（存在する場合）
- ルール:
  - 契約は機械可読で唯一の正。破壊変更は新バージョンへ（versioning_policy準拠）

### 2.2 Core Specs（固定仕様 / 変更影響大）
- 正本:
  - docs/specs/**（特に docs/specs/crosscut/**）
- ルール:
  - Coreは意味が不変。運用値・計画・手順は含めない（Policy/Plan/Runbookへ）

### 2.3 Policy（運用値 / 環境依存で可変）
- 正本:
  - docs/policy/**（TOML/MD）
- ルール:
  - 値の調整で動作を変える。Core SpecのSemVerは動かさない

### 2.4 Plan（計画 / 可変）
- 正本:
  - docs/plans/**
- ルール:
  - 進捗・順序・フェーズはPlanへ。Coreと混ぜない

### 2.5 Runbook（手順 / 可変）
- 正本:
  - docs/runbooks/**
- ルール:
  - “どう直すか” “どう復旧するか”はRunbookへ。Core/Policyと混ぜない

### 2.6 Decisions（意思決定ログ / 追記のみが基本）
- 正本:
  - docs/decisions/decisions.md
- ルール:
  - 重要方針の変更はここで記録し、参照を trace-index に繋ぐ

### 2.7 Status/Trace（リンクSSOT）
- 正本:
  - docs/status/trace-index.json
- ルール:
  - 「正本のリンク一覧」はtrace-indexが唯一の正。増殖しない

### 2.8 Legacy（隔離 / 正として使わない）
- 正本:
  - docs/legacy/**
- ルール:
  - 正本ではない。必要ならstubで正本へ誘導する

### 2.9 Context（補助 / 正として使わない）
- 正本:
  - docs/context/**
- ルール:
  - AI/補助情報。SSOTと誤認されないよう明示する

## 3. UCEL関連の境界（SSOT分裂防止）
### 3.1 profinaut内UCEL（利用境界仕様）
- 正本:
  - docs/specs/ucel/**（profinautのconnector/spec等）
- 役割:
  - “このリポでどう使うか” の固定境界

### 3.2 UCEL Golden（思想/固定仕様/運用値/計画の正本）
- 正本:
  - docs/specs/ucel_golden/**（Core固定）
  - docs/policy/ucel_golden_policy_pack.md（Policy原本）
  - docs/policy/ucel_golden/**（Policy正規化TOML）
  - docs/plans/ucel/**（Plan原本）
- 役割:
  - “UCELの正本（Core/Policy/Plan）” をこのリポ内で固定する

### 3.3 参照ルール
- UCELの思想や運用値を参照する場合は、まず以下の入口にリンクする:
  - docs/specs/ucel_golden/README.md
- profinautの実装境界仕様は:
  - docs/specs/ucel/README.md（存在する場合）

## 4. “正本が割れそうな兆候” チェック
次が見つかったらSSOT分裂のサイン：
- 同じ役割のREADMEが複数
- roadmapが複数（plans直下とdocs直下等）
- “危険操作” “並列安全” など横断仕様が複数ファイルに分散
- legacyが正本として参照されている

対応:
- 正本へ統合し、非正本は legacy + stub（リンク誘導）にする

## 5. 変更の原則（Summary）
- Coreを動かすのは最後。まずPolicyで調整する
- Plan/Runbookは積極的に更新してよい
- SSOT分裂を起こさない（増やす前に統合）
