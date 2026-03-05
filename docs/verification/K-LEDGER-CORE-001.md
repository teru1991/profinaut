# Verification: K-LEDGER-CORE-001

## Changed files
- docs/specs/domains/K_portfolio_treasury.md
- services/portfolio/app/__init__.py
- services/portfolio/app/k_ledger_types.py
- services/portfolio/app/k_ledger_store.py
- services/portfolio/app/k_sbor.py
- services/portfolio/app/k_valuation.py
- services/portfolio/app/k_pnl.py
- services/portfolio/app/k_explain.py
- services/portfolio/tests/test_k_ledger_hash_chain.py
- services/portfolio/tests/test_k_sbor_replay_determinism.py
- services/portfolio/tests/test_k_pnl_fifo_lot.py
- services/portfolio/tests/test_k_explain_confidence.py
- scripts/ci/portfolio_tests.sh
- .github/workflows/ci.yml
- docs/verification/K-LEDGER-CORE-001.md

## What / Why
- K の中核台帳（イベントソーシング + hash chain）を実装し、改ざん検知を必須化。
- SBOR replay により任意期間再計算でき、決定論を回帰テストで固定。
- Valuation + Confidence（欠損理由）と FIFO lot PnL（基礎帰属）を実装。
- Explain で lineage / confidence / pnl breakdown を一体として提示可能にした。

## Self-check results
- Allowed-path check OK: pass（docs/services/scripts/.github/workflows のみ変更）
- Tests added/updated:
  - services/portfolio/tests/test_k_ledger_hash_chain.py
  - services/portfolio/tests/test_k_sbor_replay_determinism.py
  - services/portfolio/tests/test_k_pnl_fifo_lot.py
  - services/portfolio/tests/test_k_explain_confidence.py
- Commands:
  - `PYTHONPATH=services/portfolio pytest -q services/portfolio/tests` => 4 passed
- Secrets scan:
  - `grep -RInE '(API_KEY|SECRET|TOKEN|Authorization:|Bearer )' services/portfolio/app docs/specs/domains/K_portfolio_treasury.md scripts || true`
  - 既存 scripts 内の既知パターン一致のみ（新規K実装には秘密値なし）。

## ★履歴確認の証拠（必須）
- git log --oneline -n 50:
  - HEAD起点は `d85fa44`（I/E/J導入済み）。Kは新規 `services/portfolio` を追加して競合最小化。
- git log --merges --oneline -n 30:
  - 直近 #450〜#453 merge を確認。Kの新規実装と直接競合する履歴は見当たらない。
- git merge-base HEAD origin/<default-branch>:
  - 実行不可（この環境では `origin` remote 未設定）。
- Findings:
  - 既存リポには ledger/portfolio 関連の散在ファイルはあるが、K中核イベントソーシング実装は未整備。
  - 重複実装を避けるため `services/portfolio` 新規追加で局所化し、既存サービスへの侵襲を最小化した。
