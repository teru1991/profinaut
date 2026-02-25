# DOC-AUDIT-001: docs 監査レポート（理想定義充足＋重複意図検出）

- Task ID: `DOC-AUDIT-001`
- Scope: `docs-audit`
- 実行モード: SINGLE-RUN（調査＋レポートのみ）
- 実施日: 2026-02-16

---

## A) docs/ 全体マップ（意図カテゴリ分類）

### A-1. ルート/主要ディレクトリの役割

| パス | 主カテゴリ | 補助カテゴリ | 監査メモ |
|---|---|---|---|
| `docs/README.md` | reference / index | docs入口 | docsハブとして機能。主要文書への導線あり。 |
| `docs/plans/roadmap.md` | workplan | progress-snapshot | Step 0〜21完了を示す「計画＋進捗」の要約。 |
| `docs/changelog.md` | status/progress | release-history | 実装履歴の時系列ログ（Step単位）。 |
| `docs/assumptions.md` | rules | baseline constraints | Stepごとの前提条件（運用・実装前提）。 |
| `docs/workplan/**` | north-star/spec | MRU/work decomposition | Ultimate Gold機能カタログ＋MRU運用規則。 |
| `docs/status/**` | status/progress | evidence/log | 進捗台帳とUG-P0更新ログ。 |
| `docs/specs/**` | north-star/spec | API/UI/安全仕様 | ドメイン別仕様。 |
| `docs/runbooks/**` | runbooks | operations | 手順書（運用・検証・復旧）。 |
| `docs/verification/**` | verification | test-procedure/evidence | 検証手順と実行結果。 |
| `docs/troubleshooting/**` | runbooks | incident triage | 障害切り分け。 |
| `docs/audits/**` | audits/reference | governance health | 監査・ギャップ分析・比較。 |

### A-2. ファイル単位の分類（一覧）

#### 1) index / governance
- `docs/README.md` → `reference/index`
- `docs/assumptions.md` → `rules/baseline`
- `docs/plans/roadmap.md` → `workplan + progress`
- `docs/changelog.md` → `status/release-log`

#### 2) north-star/spec
- `docs/workplan/ultimate-gold-implementation-feature-list.md` → `north-star/spec + MRU catalog`
- `docs/specs/crosscut/parallel_task_safety_spec.md` → `rules/spec (parallel safety SSOT)`
- `docs/specs/controlplane-bots.md` → `API spec`
- `docs/specs/ui-bots.md` → `UI spec`
- `docs/specs/ui-marketdata.md` → `UI spec`
- `docs/context/notes/execution.md` → `service spec (generic)`
- `docs/context/notes/execution-gmo.md` → `provider profile spec`
- `docs/specs/simple-bot.md` → `bot behavior spec`
- `docs/specs/crosscut/dangerous_ops_taxonomy.md` → `safety policy taxonomy`
- `docs/specs/crosscut/dangerous_ops_confirmation.md` → `dangerous-op confirmation flow spec`

#### 3) runbooks / troubleshooting / verification
- `docs/runbooks/e2e-smoke-runbook.md` → `runbook/smoke`
- `docs/runbooks/paper_e2e.md` → `runbook/manual deep check`
- `docs/runbooks/reconcile-mismatch-repair.md` → `runbook/incident repair`
- `docs/runbooks/marketdata-local.md` → `runbook/local env`
- `docs/runbooks/marketdata-replay.md` → `runbook/replay`
- `docs/runbooks/supply-chain-security.md` → `runbook/security ops`
- `docs/troubleshooting/bots-502.md` → `troubleshooting`
- `docs/verification/marketdata-data-platform-smoke.md` → `verification procedure`
- `docs/verification/marketdata-data-platform-smoke-results.md` → `verification evidence`

#### 4) status/progress / audits
- `docs/status/ultimate-gold-progress-check.md` → `status dashboard`
- `docs/status/progress-updates/UG-P0-101.md`〜`UG-P0-112.md` → `append-only progress evidence`
- `docs/audits/docs-content-overlap.md` → `docs overlap audit`
- `docs/audits/repo-progress-audit-2026-02-14.md` → `repo progress audit`
- `docs/audits/ui-current-vs-spec.md` → `UI gap audit`

---

## B) 理想定義（North Star）充足度チェック

### B-1. 「最終的に何が完成か」は固定されているか

### 判定: **部分的に固定（入口はあるが単一SSOTとしては未固定）**

- 現在、North Star定義候補が複数存在:
  - `docs/workplan/ultimate-gold-implementation-feature-list.md`（理想機能カタログ）
  - `docs/status/ultimate-gold-progress-check.md`（理想との差分と進捗判定）
  - `docs/plans/roadmap.md`（Stepベースの完了計画）
- `docs/README.md` は入口として有効だが、
  - 「最終到達点の canonical 定義はこれ」という明示は弱い。

**結論:** 実運用上は `workplan` が最もNorth Starに近いが、docs全体として「単一のDefinition of Done入口」が明文化されていない。

### B-2. Epic / MRU / Contracts-SSOT / 運用原則の接続性

### 判定: **概ね接続しているが、横断リンクの規約化が不足**

- 良い点
  - MRU概念（`1PR=1scope`, Depends-on, Allowed/Forbidden paths, DoD）が `workplan` に整理済み。
  - `parallel-task-safety` に `1PR=1scope` が明文化されている。
  - `roadmap` に Contracts SSOT guardrail がある。
- 不足/弱点
  - `workplan` の各MRUと `status/progress-updates` の対応関係が機械的に追えない（命名規約はあるが索引なし）。
  - 「Contractsが絶対SSOT」であることは docs内に分散記述され、統治入口（governance hub）が未固定。
  - Epic(UG-xx) ↔ MRU ↔ PR/commit ↔ runbook/spec のリンク密度にムラがある。

### B-3. 不足項目（追加が望ましいもの）

1. **North Star canonical入口の明示**
   - 例: `docs/north-star.md`（または `docs/README.md` 冒頭）で「最終到達点はこの文書」と固定。
2. **Decision Log の明示的SSOT入口**
   - `workplan` では言及があるが、docsツリー内で決定ログ体系が見えにくい。
3. **機械可読進捗インデックス**
   - 例: `status/index.json` 相当で `UGF-ID -> status/progress-update/PR` を紐付け。
4. **handoff運用の明示文書**
   - 並行開発向けに「引継ぎ時の必須記録」を固定フォーマット化。
5. **DoD定量基準の集中管理**
   - 現状は複数文書に分散（progress-check, workplan, runbook）しており、判定基準が揺れうる。

---

## C) 重複意図ファイルの検出（グループ化＋統廃合方針）

> ここでの「重複」は、同一役割または境界が曖昧で将来的に差分発生しやすい組み合わせを指す。

### Group-01: 全体進捗/現状説明の重複

- 対象:
  - `docs/plans/roadmap.md`
  - `docs/changelog.md`
  - `docs/status/ultimate-gold-progress-check.md`
  - `docs/audits/repo-progress-audit-2026-02-14.md`
- 症状:
  - 「どこまで進んだか」を複数文書が保持。
- Canonical提案:
  - **現時点の進捗状態:** `docs/status/ultimate-gold-progress-check.md`
  - **時系列履歴:** `docs/changelog.md`
  - **計画ステップ:** `docs/plans/roadmap.md`
  - **監査レポート:** `docs/audits/*`（非SSOT）
- 統合/廃止候補:
  - `repo-progress-audit-2026-02-14.md` の進捗説明部分は将来監査では参照リンク中心へ縮約。
- 注意点:
  - README/外部リンクのリンク切れ防止。
  - 同じ進捗数値（%）を複数箇所で持たない。

### Group-02: Ultimate Gold要件の重複表現

- 対象:
  - `docs/workplan/ultimate-gold-implementation-feature-list.md`
  - `docs/status/ultimate-gold-progress-check.md`
  - `docs/status/progress-updates/UG-P0-*.md`
- 症状:
  - 要件、進捗、証跡が混ざって記載される箇所がある。
- Canonical提案:
  - **要件（What）:** `workplan`
  - **現在状態（Now）:** `status/ultimate-gold-progress-check.md`
  - **証跡（Evidence）:** `status/progress-updates/*`
- 統合/廃止候補:
  - `progress-check` から要件本文の再記述を削減し、`workplan` 参照へ。
- 注意点:
  - `UGF-ID` と `UG-P0-xxx` の対応表を導入しないと参照迷子が起きる。

### Group-03: 安全運用ルール（1PR=1scope/運用規律）の重複

- 対象:
  - `docs/specs/crosscut/parallel_task_safety_spec.md`
  - `docs/workplan/ultimate-gold-implementation-feature-list.md`（MRU運用ルール章）
  - `docs/README.md`（導線）
- 症状:
  - 同一原則が複数箇所に存在し、更新漏れリスク。
- Canonical提案:
  - **運用原則SSOT:** `docs/specs/crosscut/parallel_task_safety_spec.md`
  - `workplan` は MRU実装テンプレートに限定し、原則本文は参照化。
- 統合/廃止候補:
  - `workplan` 内の規範文章を必要最小限にして詳細は safety specへリンク。
- 注意点:
  - タスク生成時に参照されるため、互換性のため見出し名は維持推奨。

### Group-04: 実行仕様（generic vs provider-specific）の境界重複

- 対象:
  - `docs/context/notes/execution.md`
  - `docs/context/notes/execution-gmo.md`
- 症状:
  - provider固有仕様がgeneric仕様へ漏れる可能性。
- Canonical提案:
  - **汎用実行仕様:** `execution.md`
  - **取引所プロファイル:** `execution-gmo.md`
- 統合/廃止候補:
  - 廃止ではなく責務分離を強化。
- 注意点:
  - API/挙動差分の記述先を固定し、将来のmulti-exchange追加時の重複爆発を防ぐ。

### Group-05: Bots仕様（API/UX）の項目重複

- 対象:
  - `docs/specs/controlplane-bots.md`
  - `docs/specs/ui-bots.md`
  - `docs/audits/ui-current-vs-spec.md`
- 症状:
  - 状態フィールド定義やdegraded表現が複数文書で説明される。
- Canonical提案:
  - **API項目・列挙値:** `controlplane-bots.md`
  - **表示/UX規則:** `ui-bots.md`
  - **差分監査:** `ui-current-vs-spec.md`（非SSOT）
- 統合/廃止候補:
  - `ui-bots.md` 内のAPI定義本文を最小化し、参照中心へ。
- 注意点:
  - UI監査文書を仕様本文の代替にしない（監査は時点情報）。

### Group-06: E2E/検証手順の重複

- 対象:
  - `docs/runbooks/e2e-smoke-runbook.md`
  - `docs/runbooks/paper_e2e.md`
  - `docs/verification/marketdata-data-platform-smoke.md`
  - `docs/verification/marketdata-data-platform-smoke-results.md`
- 症状:
  - 実行手順と検証証跡の境界が曖昧になる箇所がある。
- Canonical提案:
  - **運用向け実施手順:** runbooks
  - **検証仕様（テストケース）:** verification/*-smoke.md
  - **実行結果:** verification/*-results.md
- 統合/廃止候補:
  - 同一コマンド列の重複は runbook 側に寄せ、verification ではチェック観点中心にする。
- 注意点:
  - CI/手動運用で参照先が異なるため、想定読者（SRE/開発/QA）を明記する。

---

## D) 推奨整理方針（このタスクでは未実施）

### D-1. 全体方針

- 原則: **1PR=1scope** を守り、
  1) 構造変更（move/stub）
  2) 内容統合（canonical化）
  3) リンク修正
  を分離する。

### D-2. 次PRタスク分割案

1. `DOC-REORG-001`（move/stub only）
   - 目的: canonical以外を「薄いスタブ」にして導線統一。
   - 変更: ファイル移動/見出し維持/移転バナー追加のみ（内容改変最小）。

2. `DOC-REORG-002`（progress ownership split）
   - 目的: `roadmap/changelog/progress-check` の責務を明文化。
   - 変更: 重複文言削減、各文書先頭に責務宣言。

3. `DOC-REORG-003`（UG traceability index）
   - 目的: `UGF-ID ↔ UG-P0 update ↔ PR` 対応表を追加。
   - 変更: `docs/status/` 内に索引（md or json）追加。

4. `DOC-REORG-004`（safety governance canonicalization）
   - 目的: `1PR=1scope` 等の運用規範を `parallel-task-safety` に集約。
   - 変更: `workplan` の規範本文を参照化。

5. `DOC-REORG-005`（spec boundary hardening）
   - 目的: `execution` generic/provider, `controlplane/ui-bots` API/UX境界を明確化。
   - 変更: 重複定義削減、境界表追加。

6. `DOC-REORG-006`（verification/runbook layering）
   - 目的: 実施手順と証跡ドキュメントの責務分離。
   - 変更: runbookは運用手順中心、verificationは判定基準中心に整理。

### D-3. PR衝突を避ける運用メモ

- 「移動だけPR」「本文修正だけPR」「リンク修正だけPR」に分割する。
- 先に canonical 宣言PR を入れ、以後のPRはその宣言に従って編集する。
- `docs/README.md` は最後に更新（途中で導線を壊さない）。

---

## 再実行可能コマンド（機械的チェック用）

```bash
git ls-tree -r --name-only HEAD docs
rg -n "^#|^##" docs -S
rg -i "status|progress|workplan|roadmap|spec|design|decision|handoff|mru" -n docs
```
