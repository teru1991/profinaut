# Docs Gate Spec v1.0（固定）
Document ID: SYS-DOCS-GATE-SPEC
Status: Canonical / Fixed Contract
Scope: docsの品質ゲート（壊れたdocsをmainへ入れない）を固定する

## 0. 目的（Non-negotiable）
docsはSSOTであり、壊れると実装と運用が壊れる。
本仕様は「docsが壊れない」ための最低限の機械検証（Gate）を固定する。

必達:
1) 参照切れを許さない（リンク/パス）
2) SSOT分裂を検知する（正本の増殖）
3) 禁止情報（secret/PII）を混入させない
4) trace-indexの整合性が取れる
5) “UNKNOWNは合格ではない”（検証不能はFAIL/UNKNOWN扱い）

## 1. 対象（Gate対象）
- docs/contracts/**（schema整合）
- docs/specs/**（参照整合 / SSOT境界）
- docs/policy/**（フォーマット / 禁止キー）
- docs/plans/**（参照整合）
- docs/runbooks/**（参照整合）
- docs/status/trace-index.json（リンクSSOT）

## 2. Gateの分類（固定）
### 2.1 CI Gate（PR時）
- docs lint（リンク切れ/参照整合/禁止キー）
- schema lint（contractsがJSONとして妥当）
- trace-index lint（JSON妥当 / 参照先が存在）

### 2.2 Runtime Gate（実行時）
- observability欠損＝UNKNOWN（健康扱い禁止）
- gate_resultsにPASS/WARN/FAIL/UNKNOWNを出す（契約SSOT準拠）

### 2.3 Daily Gate（定期監査）
- integrity_report集計の継続性
- docsの増殖監査（SSOT分裂の兆候）

※ 実装は後で良い。本書は “何を検証するか” を固定する。

## 3. CI Gate: 具体チェック一覧（固定）
### 3.1 Link & Path Integrity（必須）
- docs内の相対パス参照が存在する
- legacy配下が “正本” として参照されていない（stub経由ならOK）
- docs/specs/ucel_golden/README.md など入口が参照可能

### 3.2 SSOT Split Detection（必須）
以下を検知したらFAIL（例）:
- 同じ役割のロードマップが複数（docs/roadmap.md と docs/plans/roadmap.md 等）
- 同じ役割のポリシーが複数正本として並立（READMEが二つ等）
- crosscut仕様が重複（安全/監査/バンドルの正本が複数）

### 3.3 Forbidden Key Scan（必須）
- docs全体に対して forbidden key をスキャンする
- 検知したらFAIL（例外なし）
- 例: api_key, secret, token, private_key, authorization, cookie（具体リストはPolicy）

### 3.4 TOML/JSON Parse（必須）
- docs/policy/*.toml はTOMLとしてパースできる
- docs/status/trace-index.json はJSONとして妥当
- docs/contracts/*.schema.json はJSONとして妥当

### 3.5 Trace Index Integrity（必須）
- trace-index.json にある path は全て存在する
- verification_evidence にあるファイルも存在する（存在しないならWARNではなくFAIL）

## 4. Gate結果の固定ルール
- PASS: すべて検証でき、問題なし
- WARN: 軽微（ただしSSOT/secret/参照切れはWARNにしない）
- FAIL: 参照切れ/secret混入/SSOT分裂/JSON・TOML壊れ
- UNKNOWN: 検証不能（監視欠損など）。UNKNOWNは合格ではない

## 5. 実装メモ（非SSOT / 参考）
実装は次のいずれでも良い:
- Pythonスクリプト（docs lint）
- GitHub Actions（CI）
- make task
ただし “何を検証するか” は本書に従う。

