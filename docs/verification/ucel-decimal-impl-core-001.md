# Verification — UCEL-DECIMAL-IMPL-CORE-001

## 1) Changed files (`git diff --name-only`)
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CORE-001.md
- docs/patches/ucel/UCEL-DECIMAL-IMPL-CORE-001.patch
- docs/status/trace-index.json
- docs/verification/ucel-decimal-impl-core-001.md

## 2) What/Why
- ucel-core に Decimal運用ルール（policy/guard/tick-step/serde/newtype）を実装するための完全パッチを docs に固定しました。
- このタスクでは実コードは変更せず、後続反映で `git apply` できる形をSSOTとして確定しています。
- 追加で必要な再点検事項（serde_json依存、波及影響、tie-break、例外方針）をパッチ説明末尾に追記しました。
- trace-index は `UCEL-DECIMAL-IMPL-CORE-001` エントリのみ更新し、成果物と検証証跡を紐付けました。

## 3) Self-check results
- json.tool trace-index OK
- docs/以外変更なし OK
- link existence check（今回触ったファイル内の `docs/` 参照のみ）OK
- MISSING: none
