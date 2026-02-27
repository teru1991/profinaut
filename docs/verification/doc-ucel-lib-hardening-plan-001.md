# Verification: DOC-UCEL-LIB-HARDENING-PLAN-001

## 1) Changed files (`git diff --name-only`)
```bash
docs/README.md
docs/status/trace-index.json
```

(補足: 新規追加ファイルは `git status --porcelain` 上で確認)
- `docs/plans/ucel/ucel_library_hardening_filelevel_plan.md`
- `docs/verification/doc-ucel-lib-hardening-plan-001.md`

## 2) What / Why
- UCEL を完成ライブラリへ収束させるための「ファイル別追記指示 SSOT」を新規作成した。
- P0〜P5 と connector 共通テンプレを定義し、各ファイルに対して Insert at / Add / Risks / Tests を固定フォーマットで記述した。
- trace-index に TASK-ID の branch / artifacts / verification_evidence を補完し、後続実装タスクの入口を一本化した。
- docs/README に canonical plan への入口リンクを最小追記した。

## 3) Self-check results
- `python -m json.tool docs/status/trace-index.json > /dev/null`: OK
- docs/以外変更なし: OK
- link existence check（今回触ったファイル内の `docs/` 参照）: OK
- MISSING: none
