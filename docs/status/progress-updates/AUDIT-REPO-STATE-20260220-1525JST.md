# AUDIT-REPO-STATE-20260220-1525JST

## Summary
- 結論: **CIは Red 扱い**。直近30 runは成功/skip中心だが、直近100 runに `CI` / `Secret Scan` / `Security Supply Chain Guardrails` の失敗が複数残存（再実行で収束したか未確認）。
- ブロッカー: GitHub CLI (`gh`) が環境に存在せず、`gh run view --log-failed` による失敗ログ取得が不可。REST APIで代替したが、jobログDLは403（admin権限不足）。
- 次アクション: 失敗runを担当者権限で再確認し、失敗原因をrunログで確定させる。`status.json` の stale lock を解消済み（open PR 0件と整合）。

## Active Task / Open PRs / Locks held（status.json整合）
- 監査前 `status.json`: `active_task=DOC-DATAPLATFORM-SSOT-002-FREE`, `open_prs=[]`, `locks_held=[LOCK:shared-docs]`。
- 監査結果: GitHub REST APIで **open PR 0件**（`/pulls?state=open`）。
- 整合判断: open PRが0件のため shared-docs lock保持は stale。監査反映として `status.json`/`HANDOFF.json` を `AUDIT-REPO-STATE-001` に更新し、`locks_held=[]` に同期。

## Commit/PR timeline（直近）
### default branch / HEAD
- GitHub API default branch: `master`
- local HEAD: `work` branch @ `decd975` (Merge PR #207)

### 直近主要PR（closed/merged 上位10件）
- #207 `dbf48f3d` Implement Kraken WS channels...  
- #206 `f8c03460` Add ucel-cex-kraken REST adapter...  
- #205 `f1e5d1bb` Kraken SSOT catalog rails...  
- #204 `ebd6899e` DP-DB-005 deterministic E2E hardening...  
- #203 `c12de523` Gold serving sinks...  
- #202 `88be6544` Silver recompute/backfill...  
- #201 `f23e2fd1` Bronze writer hardening...  
- #200 `f2a556ce` local data platform compose stack...  
- #199 `db760c90` CI/runtime regressions hardening...  
- #198 `ce0aa257` deterministic E2E harness...

## CI status
### Snapshot
- Actions run (latest 30): success 19 / skipped 11 / failure 0
- Actions run (latest 100): failure系 20件

### 代表的な failure run
1. `22209816671` Security Supply Chain Guardrails
   - failed job: `SBOM + Vulnerability Scan + Artifact Guard`
   - URL: https://github.com/teru1991/profinaut/actions/runs/22209816671
2. `22209474150` Secret Scan
   - failed job: `gitleaks`
   - URL: https://github.com/teru1991/profinaut/actions/runs/22209474150
3. `22208983745` CI
   - failed job: `docker-compose-smoke`
   - URL: https://github.com/teru1991/profinaut/actions/runs/22208983745

### 失敗原因の要約 / 再現手順
- 要約（確定度: 中）
  - `gitleaks` 失敗、`docker-compose-smoke` 失敗、`SBOM/Artifact Guard` 失敗を確認。
  - ただし jobログ本文は REST APIで403 (`Must have admin rights`) のため詳細原因までは確定不可。
- 再現手順（推奨）
  - `docker compose config`
  - `docker compose up -d`（該当stack）
  - `cargo test`（Rust workspace）
  - `gitleaks detect --source .`
  - `syft .` / `grype ...`（SBOM/脆弱性系のCI実装に準拠）

## “差分の危険領域”チェック
- 監査時 local dirty: `services/marketdata-rs/Cargo.lock`（lockfile）
- 監査タスク自体で forbidden paths は未変更。
- 直近履歴上は `.github`, `contracts`, `infra`, `docker-compose.yml`, lockfile への変更が含まれる（監査対象として閲覧のみ）。

## ローカル検証（可能範囲）
- `ucel` で `cargo fmt --check` 実行: **Fail**（フォーマット差分検出）
- `cargo clippy` / `cargo test` は fmt failのため未実施（同一コマンド連結）

## 既知の警告と影響評価
- `gh` コマンド未導入: PR/Run詳細のCLI取得不可（REST APIで代替）。
- Actions job logs API 403: 失敗原因の詳細行（stacktrace/stderr）確認不可。
- 影響: CI failureの「存在」は確認済み、ただし root cause は暫定記述（要追加確認）。

## Evidence
- Repo: https://github.com/teru1991/profinaut
- Open PR list (0): https://api.github.com/repos/teru1991/profinaut/pulls?state=open&per_page=100
- Closed PR list: https://api.github.com/repos/teru1991/profinaut/pulls?state=closed&per_page=10
- Actions runs: https://api.github.com/repos/teru1991/profinaut/actions/runs?per_page=100
- Failed run URLs:
  - https://github.com/teru1991/profinaut/actions/runs/22209816671
  - https://github.com/teru1991/profinaut/actions/runs/22209474150
  - https://github.com/teru1991/profinaut/actions/runs/22208983745
- SSOT docs:
  - docs/SSOT/README_AI.md
  - docs/status/status.json
  - docs/handoff/HANDOFF.json
  - docs/decisions/decisions.md
  - docs/status/trace-index.json
