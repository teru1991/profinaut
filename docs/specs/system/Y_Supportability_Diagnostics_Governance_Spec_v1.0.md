分類理由: 運用・規約・管理の正本（SSOT）を定義するガバナンス文書のため「診断仕様」に分類。

# Y: Supportability / Diagnostics Governance Spec v1.0（SSOT / Level 1）

Source: Y Appendix: Supportability / Diagnostics Governance Spec v1.0（清書） :contentReference[oaicite:0]{index=0}

---

## 1.1 目的 / 位置づけ（SSOT）
- 本ドキュメントは、Yドメイン（Supportability / Diagnostics）の機能群を「確実・安全・堅牢・高速」に運用するための **非機能要件・規約・検証・運用プロセス** をSSOTとして固定する。 :contentReference[oaicite:1]{index=1}
- 本書は **Y Core Spec（機能仕様）と同格** の「運用上の正本」であり、Y関連の変更は本書の規約に従う。 :contentReference[oaicite:2]{index=2}

TODO:
- 本書が参照する「Y Core Spec」の識別子（版/パス/コミット等）を明記する。

---

## 1.2 設計規約（Logging / Metrics / Tracing / IDs / SemVer）

### 1.2.1 構造化ログ規約
- 必須フィールド（全サービス共通） :contentReference[oaicite:3]{index=3}
  - `ts`（UTC推奨）, `level`, `service`, `env`, `version`
  - `trace_id`, `run_id`（該当時）, `incident_id`（該当時）
  - `error_id`（エラー時は必須）, `blame_class`（該当時）
- 禁止事項 :contentReference[oaicite:4]{index=4}
  - 秘密情報・認証情報（APIキー、JWT、署名素材、Authorizationヘッダ等）のログ出力は禁止
  - 個人情報（PII）・外部共有で問題になる識別子の無断出力は禁止
- 例外運用 :contentReference[oaicite:5]{index=5}
  - 例外が必要な場合は `approval_id` を伴う承認が必須（期限付き）
  - 例外ログは通常ログと分離した診断チャネルへ出す（本番ログ汚染防止）

TODO:
- 「診断チャネル」の具体（ログ先/ラベル/ルーティング/保持期間）を別SSOTへリンクする。

### 1.2.2 メトリクス命名・ラベル規約
- 命名：`<domain>_<subsystem>_<metric>_<unit>` を基本 :contentReference[oaicite:6]{index=6}
- 型の使い分け :contentReference[oaicite:7]{index=7}
  - Counter：件数/発生回数
  - Gauge：瞬間値（キュー深さ等）
  - Histogram：遅延・サイズ等（SLOに直結）
- ラベル制限（Cardinality上限） :contentReference[oaicite:8]{index=8}
  - `error_id` のような高次元は TopK集約や階層化を前提にする
  - `user_id` 等のユニーク値ラベルは禁止（PII/爆発防止）

TODO:
- 「Cardinality上限」の具体値（上限/例外/検知/CIゲート）を定義する。

### 1.2.3 トレース規約（OTel想定）
- span属性に PII/secret 禁止 :contentReference[oaicite:9]{index=9}
- 必須属性：`service/env/version`, `error_id`（該当時）, `run_id`（該当時） :contentReference[oaicite:10]{index=10}
- サンプリングは環境ポリシーで管理（prodは保守的、Break-Glassで拡張） :contentReference[oaicite:11]{index=11}

TODO:
- 「環境ポリシー」の参照先（policy.md等）と、Break-Glassで拡張する範囲/手順を明記する。

### 1.2.4 ID体系（相関ID）規約
- 固定ID：`trace_id`, `run_id`, `schema_version` :contentReference[oaicite:12]{index=12}
- 運用ID：`incident_id`, `change_id`, `approval_id`, `repair_id`, `silence_id`, `experiment_id` :contentReference[oaicite:13]{index=13}
- 生成規約 :contentReference[oaicite:14]{index=14}
  - 衝突しない一意性（ULID等推奨）
  - 伝播を最優先（ログ/メトリクス/トレース/チケット/KBで一致）

TODO:
- ULID等の具体（形式/長さ/文字種/ライブラリ/生成元）をSSOT化する。

### 1.2.5 診断SemVer（diag_semver）運用
- `diag_semver` は診断I/F全体の互換性を示す :contentReference[oaicite:15]{index=15}
- 破壊変更（MAJOR）は原則禁止。必要時は以下を必須とする :contentReference[oaicite:16]{index=16}
  - deprecation期間
  - 移行ガイド
  - analyzerの後方互換

TODO:
- `diag_semver` のスキーマ（どこに埋めるか、bundle内フィールド名など）を contract として固定する。

---

## 1.3 セキュリティ・権限・監査（Security / RBAC / Audit）

### 1.3.1 最小権限（Least Privilege）
- 診断コンポーネントは読み取り専用が原則 :contentReference[oaicite:17]{index=17}
- Secretsは「存在確認」等の最小操作を除きアクセスしない :contentReference[oaicite:18]{index=18}
- ネットワーク到達先は allowlist 制御（Permissions Reportで可視化） :contentReference[oaicite:19]{index=19}

TODO:
- Permissions Report の場所/生成方法/更新頻度/閲覧権限を明記する。

### 1.3.2 Break-Glass規約
- TTL必須・理由必須・監査必須 :contentReference[oaicite:20]{index=20}
- prodでは承認必須（Approval workflow） :contentReference[oaicite:21]{index=21}
- 事後レビュー（なぜ必要だったか、改善点）をKBへ残す :contentReference[oaicite:22]{index=22}

TODO:
- Approval workflow の定義（承認者/経路/記録先/テンプレ）をSSOT化する。

### 1.3.3 外部共有（Export/Transfer）規約
- 原則：最小開示 + Audit Pack :contentReference[oaicite:23]{index=23}
- 外部共有は **4-eyes（2人承認）** を推奨（環境ポリシーで強制可能） :contentReference[oaicite:24]{index=24}
- 送付は受け手公開鍵で暗号化（Out-of-band） :contentReference[oaicite:25]{index=25}
- 受領確認・削除証跡まで監査ログに残す :contentReference[oaicite:26]{index=26}

TODO:
- Audit Pack の内容（含める/含めない/マスキング規約）を定義する。

### 1.3.4 監査ログ保全
- 改ざん検知（署名/証拠チェーン）を持つ :contentReference[oaicite:27]{index=27}
- 保管期間とアクセス権を環境ポリシーに従って固定 :contentReference[oaicite:28]{index=28}
- “silence（抑制）” など運用操作も監査対象に含める :contentReference[oaicite:29]{index=29}

---

## 1.4 信頼性・性能・容量（Reliability / Performance / Capacity）

### 1.4.1 診断処理のSLO（Y自身のSLO）
対象指標： :contentReference[oaicite:30]{index=30}
- `bundle_ready_time`
- `triage_time`
- `analyzer_time`
- `redaction_violation_rate`（目標0）
- `obs_gap_time`
- `silent_failure_time`

TODO:
- 各SLOの定義（計測点/単位/目標値/集計窓）を定義する。

### 1.4.2 本番影響の上限（診断のコスト制御）
- bundle生成はCPU/I/O/同時実行数に上限 :contentReference[oaicite:31]{index=31}
- deepは原則禁止（Break-Glassのみ） :contentReference[oaicite:32]{index=32}
- 同一incidentでの連打禁止（クールダウン） :contentReference[oaicite:33]{index=33}
- 診断ログは分離し、送信前にサンプリング/レート制限 :contentReference[oaicite:34]{index=34}

TODO:
- クールダウン時間、上限値（CPU/I/O/並列数）、deepの定義範囲を明記する。

### 1.4.3 容量計画（Retention）
- ログ/メトリクス/トレース/bundleの増加見積もりを持つ :contentReference[oaicite:35]{index=35}
- 上限超過時の劣化（fail-safe） :contentReference[oaicite:36]{index=36}
  - “要約＋参照”
  - 重要度（severity）優先
  - 外部共有は最小開示へ

TODO:
- Retentionの具体（保持期間/階層/削除順/例外）を環境ポリシーとして固定する。

---

---

## 1.6 リリース・変更管理（Release / Change Management）

### 1.6.1 change_id の徹底
- 診断I/F、ポリシー、Runbook、Catalog、Analyzerルールの変更は `change_id` を必ず付与し、影響評価を残す :contentReference[oaicite:45]{index=45}

### 1.6.2 段階リリース
- dev → staging → prod の順で適用 :contentReference[oaicite:46]{index=46}
- canary適用で差分（bundle/perf/error）を比較 :contentReference[oaicite:47]{index=47}
- SLO悪化、redaction違反、obs_gap増大はロールバック基準 :contentReference[oaicite:48]{index=48}

### 1.6.3 Deprecation運用
- 古いフィールド/メトリクス/ログ項目の廃止は猶予期間を設け、移行ガイドを提供する :contentReference[oaicite:49]{index=49}

TODO:
- 影響評価（impact assessment）のテンプレ（評価観点/承認者/記録先）を定義する。

---

## 1.7 運用プロセス（Ops Process）

### 1.7.1 Severity分類と標準対応
- severity（S0〜S4）をSSOT化し、以下を決める :contentReference[oaicite:50]{index=50}
  - 必須の生成物（bundle / audit pack）
  - Forensic Hold要否
  - Self-Healの可否（提案止まり/自動可）
  - 通知強度

TODO:
- S0〜S4 の定義（基準/例/対応SLA）を本文または用語集に固定する。

### 1.7.2 標準フロー
順序（標準） :contentReference[oaicite:51]{index=51}
1. triage（ワンコマンド）
2. bundle生成（ポリシー準拠）
3. runbook実施（human/auto分離）
4. 復旧（self-heal/repair tx）
5. audit pack作成（必要時）
6. KB更新（再発防止）
7. SLO/KPIレビュー

TODO:
- 「ワンコマンド triage」のI/F（CLI/HTTP/権限/出力）を契約として固定する。

### 1.7.3 GameDay（演習）
- stagingで定期演習し、bundle/analyzer/self-healを必ず使用する :contentReference[oaicite:52]{index=52}
- 演習結果からRunbook/Policy/Catalogを更新し、改善を回す :contentReference[oaicite:53]{index=53}

### 1.7.4 ポストモーテム
- `incident_id` を中心にテンプレ化 :contentReference[oaicite:54]{index=54}
- 原因・検知・対応・再発防止（DoD）を必須項目として固定 :contentReference[oaicite:55]{index=55}

TODO:
- ポストモーテム・テンプレの格納場所（docs/ など）を明記する。

---

## 1.8 ドキュメントとSSOT運用（Docs Governance）

### 1.8.1 正本の置き場（推奨）
推奨パス一覧 :contentReference[oaicite:56]{index=56}
- `docs/specs/y_supportability_diagnostics/core.md`（機能仕様）
- `docs/specs/y_supportability_diagnostics/governance.md`（本書）
- `docs/specs/y_supportability_diagnostics/policy.md`
- `docs/specs/y_supportability_diagnostics/error_catalog.md`
- `docs/runbooks/diagnostics_*.md`
- `docs/contracts/diagnostics_*.schema.json`
- `docs/tools/diagnostics_analyzer.md`

### 1.8.2 docs更新検知と整合性ゲート
- docs変更時に差分要約＋影響範囲をCIで出す :contentReference[oaicite:57]{index=57}
- link切れ、legacy stub不整合、contract破壊（SemVer）をfail-fastで検知 :contentReference[oaicite:58]{index=58}

### 1.8.3 用語固定
- incident/change/approval/repair/silence 等の用語は用語集に固定し、意味の揺れを禁止する :contentReference[oaicite:59]{index=59}

TODO:
- 用語集（glossary）の正本パスを明記する。

---

## 1.9 アーキテクチャ境界（Architecture Rules）

### 1.9.1 Providerプラグイン責務境界
- 各ドメインは `DiagnosticsProvider` として診断寄与（contribution）を提供する :contentReference[oaicite:60]{index=60}
- Yは “統合と安全保証（redaction/整合/監査/ポリシー強制）” を担う :contentReference[oaicite:61]{index=61}

### 1.9.2 依存禁止
- 診断が本番の秘密領域へ循環依存しない :contentReference[oaicite:62]{index=62}
- “診断が診断を呼ぶ”構造を禁止（自己DoS防止） :contentReference[oaicite:63]{index=63}
- 例外が必要な場合は Approval + 期限付きで明示する :contentReference[oaicite:64]{index=64}

TODO:
- `DiagnosticsProvider` の契約（I/F、提供可能なcontributionの種類）を contract として参照する。

---

## 1.10 例外運用（Exception Handling）
- 例外は必ず `approval_id` を伴う :contentReference[oaicite:65]{index=65}
- 期限付き（TTL） :contentReference[oaicite:66]{index=66}
- 事後にKBへ記録し、恒久対応（仕様/運用の改善）へ繋げる :contentReference[oaicite:67]{index=67}

---

## 1.11 Capability Index（ID / 用語 / 観測名 / アーティファクト）
※入力内に登場する番号/ID/識別子・重要語を **保持して列挙**（追加仕様なし）

### 1.11.1 相関ID / 運用ID / 変更管理ID
- `trace_id` :contentReference[oaicite:68]{index=68}
- `run_id` :contentReference[oaicite:69]{index=69}
- `incident_id` :contentReference[oaicite:70]{index=70}
- `error_id` :contentReference[oaicite:71]{index=71}
- `blame_class` :contentReference[oaicite:72]{index=72}
- `schema_version` :contentReference[oaicite:73]{index=73}
- `change_id` :contentReference[oaicite:74]{index=74}
- `approval_id` :contentReference[oaicite:75]{index=75}
- `repair_id` :contentReference[oaicite:76]{index=76}
- `silence_id` / “silence（抑制）” :contentReference[oaicite:77]{index=77} :contentReference[oaicite:78]{index=78}
- `experiment_id` :contentReference[oaicite:79]{index=79}

### 1.11.2 ログ必須フィールド
- `ts`, `level`, `service`, `env`, `version` :contentReference[oaicite:80]{index=80}

### 1.11.3 診断I/F互換性
- `diag_semver` :contentReference[oaicite:81]{index=81}
- deprecation期間 / 移行ガイド / analyzer後方互換 :contentReference[oaicite:82]{index=82}

### 1.11.4 SLO / 監視指標名（Y自身）
- `bundle_ready_time` :contentReference[oaicite:83]{index=83}
- `triage_time` :contentReference[oaicite:84]{index=84}
- `analyzer_time` :contentReference[oaicite:85]{index=85}
- `redaction_violation_rate` :contentReference[oaicite:86]{index=86}
- `obs_gap_time` :contentReference[oaicite:87]{index=87}
- `silent_failure_time` :contentReference[oaicite:88]{index=88}

### 1.11.5 運用分類 / プロセス用語
- severity：`S0〜S4` :contentReference[oaicite:89]{index=89}
- `triage`（ワンコマンド） :contentReference[oaicite:90]{index=90}
- bundle / Golden Bundle（golden） :contentReference[oaicite:91]{index=91}
- analyzer :contentReference[oaicite:92]{index=92}
- Redaction回帰（疑似JWT/鍵/秘密っぽい文字列） :contentReference[oaicite:93]{index=93}
- Runbook Drift検査 :contentReference[oaicite:94]{index=94}
- canary :contentReference[oaicite:95]{index=95}
- GameDay :contentReference[oaicite:96]{index=96}
- ポストモーテム（DoD含む） :contentReference[oaicite:97]{index=97}
- Audit Pack :contentReference[oaicite:98]{index=98}
- Forensic Hold :contentReference[oaicite:99]{index=99}
- Self-Heal :contentReference[oaicite:100]{index=100}
- Break-Glass :contentReference[oaicite:101]{index=101}
- allowlist / Permissions Report :contentReference[oaicite:102]{index=102}
- “要約＋参照” / fail-safe :contentReference[oaicite:103]{index=103}

### 1.11.6 アーキテクチャ用語
- `DiagnosticsProvider` / contribution :contentReference[oaicite:104]{index=104}

### 1.11.7 SSOT推奨配置（ファイルパス）
- `docs/specs/y_supportability_diagnostics/core.md` :contentReference[oaicite:105]{index=105}
- `docs/specs/y_supportability_diagnostics/governance.md` :contentReference[oaicite:106]{index=106}
- `docs/specs/y_supportability_diagnostics/policy.md` :contentReference[oaicite:107]{index=107}
- `docs/specs/y_supportability_diagnostics/error_catalog.md` :contentReference[oaicite:108]{index=108}
- `docs/runbooks/diagnostics_*.md` :contentReference[oaicite:109]{index=109}
- `docs/contracts/diagnostics_*.schema.json` :contentReference[oaicite:110]{index=110}
- `docs/tools/diagnostics_analyzer.md` :contentReference[oaicite:111]{index=111}
