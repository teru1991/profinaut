# Backtest / Forward Test / Simulation Reproducibility Core Spec v1.0（固定仕様）
Deterministic-by-input / Evidence replay / Dataset pinning / Experiment integrity

- Document ID: RT-REPRO-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): H（Research/Testing）
- Depends-on（Fixed）:
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Execution Safety（live誤接続防止）: `docs/specs/execution/runtime_execution_safety_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
- Policy separation（固定しない）:
  - 計算資源上限、並列数、保持期間、許容誤差、seed戦略 → `docs/policy/**`
  - 運用手順（実験登録、成果物保管、審査） → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、バックテスト/フォワードテスト/シミュレーションの結果を **再現可能**にし、
“都合のよい結果だけ残る” 状態を防ぎ、実運用への移行を安全にする。

必達要件（固定）：
1) **Deterministic-by-input**：同じ入力データ範囲＋同じ設定＋同じバイナリで同じ結果、または差分説明が残る
2) **Dataset pinning**：データは必ず “範囲/版/ハッシュ” で固定される（曖昧参照禁止）
3) **Experiment registry**：実験のIDとメタデータが監査可能に残る（audit_event）
4) **Separation from live**：研究実行は live 実行と物理/論理に分離され、誤発注ゼロ
5) **Evidence replay**：replay_pointers により “どの入力を使ったか” が追跡できる
6) **Integrity-aware**：データの integrity FAIL/UNKNOWN は結果に明示的に反映される（隠さない）
7) **Performance safe**：大規模実験でも本番の安定を壊さない（資源隔離はPolicy/運用だが分離は固定要求）

---

## 1. 範囲（in / out）
### 1.1 In
- Backtest（過去データでの検証）
- Forward test（リアルタイムデータの検証：実発注なしが基本）
- Simulation（仮想市場/仮想約定/スリッページ等）
- Dataset pinning（入力範囲固定）
- Experiment identity / metadata / artifacts
- Repro evidence（replay pointers / integrity evidence）
- Live guard（誤接続防止）

### 1.2 Out
- 特定の戦略アルゴリズムの内容
- 特定のMLモデル設計（別ドメインで扱える）
- UIの詳細設計

---

## 2. Experiment Identity（固定）
### 2.1 必須ID（固定）
- `experiment_id`：実験の一意ID（人間に提示される）
- `run_id`：実行プロセス単位（Platform Foundation）
- `trace_id`：主要処理の相関（可能な範囲）
- `schema_version`：出力の契約版
- `dataset_ref`：入力データ参照（後述の pinning を満たす）
- `config_hash`：戦略/実験設定のハッシュ（secret無し）
- `binary_hash`：実行バイナリ識別（startup_report参照）

### 2.2 監査（固定）
- 実験開始/終了/失敗/中断は audit_event に必ず残す
- “結果だけ” を保存するのは禁止（実験メタが無い結果は無効）

---

## 3. Dataset Pinning（固定）
### 3.1 原則（固定）
データ参照は以下のいずれかを満たす “固定参照” のみ許可：
- (A) content-addressed（ハッシュ）＋範囲（window/partition）
- (B) object store key などの安定キー＋ハッシュ（改ざん検知）
- (C) immutable snapshot id

“latest” や “昨日のデータ” のような曖昧参照は禁止。

### 3.2 最低限の pinning 要件（固定）
dataset_ref は少なくとも：
- 対象（venue/stream/symbol/channel）
- window（start/end UTC）
- storage location reference（key/partition）
- hash/manifest reference
を含められること。

---

## 4. Reproducibility Model（固定）
### 4.1 Deterministic-by-input（最低保証）
与えられた：
- dataset_ref（固定参照）
- config_hash（設定）
- binary_hash（実装）
で、以下のいずれかを満たす：
- 結果が一致（同一指標・同一トレード列 等）
- 一致しない場合、差分の理由が構造化され audit_event.details に残る
  - 例：浮動小数の非決定、並列順序差、外部依存差

### 4.2 Replay type（固定）
最低限 Type B（Evidence replay）を必須：
- gap/quality 判定、指標集計、統計結果が同じになる、または差分説明が残る

Type A（完全一致）は将来拡張（固定仕様では要求しない）

---

## 5. Forward Test（固定）
### 5.1 原則（固定）
- Forward test は “実発注なし” を基本（shadow）
- 実発注が必要な場合は、それは実験ではなく “Controlled Live” として execution safety を満たす（dangerous op）

### 5.2 データ品質の扱い（固定）
- 監視欠損やquarantineは結果に反映（UNKNOWN/DEGRADED）
- “綺麗な区間だけ評価” する場合、そのフィルタ条件は設定に明示し監査する

---

## 6. Execution Guard（誤発注防止：固定）
研究/検証環境は、実発注へ繋がらないことが固定条件。

固定要求：
- 研究プロセスは default で killswitch=BLOCK 相当（実発注APIが呼べない）
- 誤って live endpoint が設定されても pre-trade gate で拒否される構造
- もし実発注をするなら control plane の dangerous op を経由し、明示的に “research->live” を切替する（常用禁止）

---

## 7. Integrity awareness（固定）
- integrity_report FAIL/UNKNOWN の区間を含む場合、その事実を結果メタに必ず残す
- “FAIL区間を除外” する場合、その除外条件を設定として固定し、監査に残す

---

## 8. Artifacts（固定：最低限の成果物）
実験は最低限以下を成果物として持てること：
- experiment metadata（id, hashes, dataset_ref, window）
- result summary（指標）
- optional: event-level outputs（トレード列、シグナル列）
- replay_pointers_ref（入力範囲の参照）
- integrity evidence refs（integrity_report_ref）

成果物は secret-free。

---

## 9. Audit（固定）
最低限、以下の audit_event を残す：
- `experiment.start`（dataset_ref + config_hash + binary_hash）
- `experiment.end`（summary + artifacts refs）
- `experiment.fail`（error.kind/code + evidence refs）
- `experiment.dataset.pinned`（pinning details）
- `experiment.integrity.note`（FAIL/UNKNOWN含む場合の注記）
- `support_bundle.created`（必要時）

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) dataset_ref が曖昧参照を許さない
2) 同じ dataset_ref+config_hash+binary_hash で結果が一致、または差分説明が残る
3) integrity FAIL/UNKNOWN が結果メタに反映される
4) 研究実行が live 発注に繋がらない（killswitch/pre-trade gate）
5) audit_event が start/end/fail を必ず残す
6) replay_pointers で入力範囲が辿れる

---

## 11. Policy/Runbookへ逃がす点
- 計算資源上限、並列数、保持期間、許容誤差、seed戦略
- 実験登録/審査/保存の運用手順
→ Policy/Runbookへ（意味は変えない）

---
End of document
