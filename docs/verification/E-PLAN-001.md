# E-PLAN-001 verification

## 1) Changed files (git diff --name-only)
- docs/contracts/README.md
- docs/status/trace-index.json
- libs/contracts_bridge/safety_bridge.py
- tests/test_safety_bridge.py
- docs/verification/E-PLAN-001.md

## 2) What/Why (3〜7行)
- E（Safety Controller）の本実装に先立ち、SSOT契約 `docs/contracts/safety_state.schema.json` と
  legacy safe_mode/command の橋渡し（libs/contracts_bridge）を契約準拠に統一した。
- これにより `safety_state.mode` は 3値（NORMAL/SAFE/EMERGENCY_STOP）のみが出力され、
  legacy語彙（SAFE_MODE/HALT/DEGRADED）が mode に漏れないことをテストで固定した。
- 未知入力は SAFE に倒れる（fail-closed）、緊急系（HALT/EMERGENCY）は EMERGENCY_STOP に倒れることを保証した。
- 併せて contracts README に mode 語彙境界（SSOT 3値/legacy正規化）を最小追記した。

## 3) Self-check results
- Allowed-path check OK:
  - command: `git diff --name-only | awk '{ ok=($0 ~ /^docs\// || $0 ~ /^libs\// || $0 ~ /^contracts\// || $0 ~ /^bots\// || $0 ~ /^dashboard_api\// || $0 ~ /^worker\// || $0 ~ /^tests\// || $0 ~ /^tools\// || $0 ~ /^\.github\/workflows\// || $0=="pyproject.toml" || $0=="requirements-dev.txt"); if(!ok) print $0 }'`
  - result: （出力なし）
- docsリンク簡易チェック:
  - `docs/contracts/README.md` の `docs/` 参照 3件を検査し、欠損 0 件を確認。
- trace-index json.tool OK（更新した場合）:
  - `python -m json.tool docs/status/trace-index.json > /dev/null` : OK
- Secrets scan（出金先・鍵・トークン等がdocsに含まれていない）:
  - command: `rg -n "AKIA|BEGIN PRIVATE KEY|vault|token|secret|0x[0-9a-fA-F]{40}" docs | head`
  - result: token などの一般ドキュメント記述は検出されたが、秘密鍵・認証情報実値の混入は確認されず。

## 4) ★履歴確認の証拠（必須）
- git log/merges/merge-base の要点（SHAと結論）
  - log: `git log --oneline --decorate -n 50` で HEAD `8358914`（PR #433 merge）まで確認。Safety bridge 対象変更はこの範囲では未更新。
  - graph: `git log --graph --oneline --decorate --all -n 80` で最近は UCEL 系統中心で、対象ファイル競合頻度は低い。
  - merges: `git log --merges --oneline -n 30` で直近 merge 群を確認し、bridge/schema直接変更なし。
  - latest show: `git show 8358914` は trace-index と ucel-testkit 周辺変更であり SafetyState bridge 直接影響なし。
  - merge-base: remote 未設定のため `git merge-base HEAD work` を使用し `8358914626d433807780273ef5d9030144e550ec` を確認。
  - branch: `git branch -vv` で `feature/e-plan-001-001` と `work` は同一点から分岐。
  - reflog: `git reflog -n 30` で当該ブランチ作成履歴を確認。
- blame で判明した制約/意図（該当があれば）
  - libs/contracts_bridge/safety_bridge.py: `e8f4f297` 初回導入時に fail-closed はあるが legacy語彙（SAFE_MODE/HALT）を `current_mode` に出していた。
  - docs/contracts/safety_state.schema.json: `d9d008f9` で mode enum が `NORMAL/SAFE/EMERGENCY_STOP` に固定され `additionalProperties=false` が明示。
- 判定:
  - `safety_state.mode` は履歴上も schema 上も `NORMAL/SAFE/EMERGENCY_STOP` のみが正。
  - legacy safe_mode（SAFE_MODE/HALTED/DEGRADED）は bridge で正規化し契約 mode に漏らさない責務へ修正。

## 5) Test / Build evidence
- `python -m pytest -q tests/test_safety_bridge.py`
  - result: OK（5 passed）
- `python -m pytest -q`
  - result: fail（既存 async テスト 6件が pytest-asyncio 未導入で失敗。今回差分起因ではない）
- `bash tools/validate_schemas/validate.sh`
  - result: OK（All schemas compiled successfully: 7）
