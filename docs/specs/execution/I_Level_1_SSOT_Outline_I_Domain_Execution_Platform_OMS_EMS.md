# Level 1 SSOT Outline — I Domain: Execution Platform（OMS/EMS）

## 0. Meta
- ドメイン文字: **I**
- 文書名: I. Execution Platform（OMS/EMS）SSOT 最終清書（vFinal.4 Consolidated）
- スコープ: Bot/UI/バッチ/復旧/再送を含む全発注入力を「唯一の出口（Single Egress）」へ統一し、安全・整合・監査・復旧・性能・運用・証明を強制する。
- TODO: 文書オーナー / 承認者 / 適用環境（prod/stg） / 変更履歴ポリシー / リポジトリ配置先

---

## 1. 目的
Iドメインは、取引所差分を吸収しつつ、全発注入力を唯一の出口（Single Egress）として統一し、安全・整合・監査・復旧・性能・運用・証明を強制する。攻撃・誤操作・異常データ・依存崩壊に対しても破綻しない。

---

## 2. 絶対要件（Non-negotiable）
1. 二重発注ゼロ：冪等 + outbox + dedupe + 再送安全
2. Unknown許容・必ず収束：照合で最終確定
3. Cancel/Close最優先：Lane0別枠で飢餓ゼロ
4. paper/shadow/live同一経路：live誤爆は物理防止
5. 説明可能：理由コード＋decision record＋replay＋lineage
6. 完全監査（改ざん耐性）：hash chain / seal / 署名枠
7. 再起動・二重起動耐性：leader-only live、bootstrap収束
8. NFR統合：性能/容量/耐久/保持/供給網/DR
9. Assurance（証明）：CI成果物で“満たした”を提示可能
10. 運用SSOT：runbook/telemetry/config/change/governance
11. 最終セーフティゲート：live送信の抜け道ゼロ

---

## 3. 実装単位（コンポーネントSSOT）
- intent_gateway/：受付・冪等・構文検証
- router/：capability/丸め/client_order_id
- preflight/：署名/時刻/ガード/権限/整合性/引当
- throttler/：rate limit/優先レーン/順序/バックプレッシャー
- executor/：new/cancel/amend/ops
- state_machine/：遷移/競合/Unknown
- fill_processor/：fills/dedupe/fee
- reconciler/：照合/収束
- persistence/：outbox/inbox/events/projection
- observability/：metrics/tracing/SLO
- audit/ ＋ audit/tamper_proof/：監査・改ざん耐性
- replay/ ＋ lineage/：decision record / 根拠系譜
- reservation/：引当台帳
- compound/：OCO/Bracket等
- guards/ ＋ impact/ ＋ quote_protection/：異常停止/板影響/自己板保護
- health/：venue health scoring
- rollout/：段階導入
- ha/ ＋ session/：leader/standby、bootstrap
- verification/：形式検証Lite
- assurance/：E2E不変条件・証拠・ゲート
- high_risk_ops/：資金移動枠（hard-disable）
- contracts/：外部契約（ゲート化）

TODO:
- 各コンポーネントの入出力インタフェース（API/イベント/DBスキーマ）定義
- コンポーネント間の責務境界（誰が何を“正”として持つか）明文化

---

## 4. 正本データ（固定）
- order_intents（Intentの正）
- outbox（送信前永続）
- inbox（受信dedupe）
- order_events（append-only）
- orders, fills
- audit_events + audit_chain + audit_seals
- decision_records + execution_lineage
- reservations
- venue_health_snapshots / rollout_plans（必要時）
- artifacts/assurance/*.json（証拠パッケージ）

TODO:
- 各データのスキーマ・キー・一意制約・保持期間（retention）・アーカイブ方針
- append-onlyの具体的要件（追記以外の操作禁止の担保方法）

---

## 5. 送信前ガード順序（固定）
live送信までの固定順序（必ずこの順）：
1. Intent構文/必須
2. 権限/ポリシー（bot/user）
3. Modeガード（live誤爆防止：config→権限→鍵→環境→HA role）
4. データ健全性ガード（乖離/欠損/異常ティック/板薄）
5. 引当（reservation：FASTでも必須）
6. capability/丸め適用
7. 優先レーン投入（rate/順序/負荷制御）
8. outbox永続→送信→ack/照合で確定

TODO:
- 各ガードの判定基準（閾値、参照データ、失敗時アクション、理由コード体系の紐付け）
- “FAST”の定義と例外の禁止/許可条件

---

## 6. 優先レーン・順序・負荷制御（固定）
- Lane0：cancel/close/flatten（別枠・最優先）
- Lane1：reduce-only new
- Lane2：通常 new/amend
- Lane3：query/reconcile
- 順序保証：per-venue×per-symbol（必要ならaccount_ref）直列化
- Backpressure：全体/bot/symbol/account上限、過負荷時new拒否・cancel維持、retry雪崩抑止

TODO:
- レーンごとの上限（数値）と計測指標、拒否/遅延/再試行の規約
- “飢餓ゼロ”の検証条件（観測可能なSLO/メトリクス）

---

## 7. 状態機械・Unknown・照合（固定）

### 7.1 状態（列挙）
Draft→Validated→Enqueued→Sent→AckedOpen→(Partial)→Filled  
CancelRequested→CancelPending→Canceled  
Rejected/Expired/Failed/Unknown

### 7.2 不変条件（Invariants）
- 二重確定ゼロ
- fill単調増加
- trade重複ゼロ
- Unknown収束
- Lane0飢餓ゼロ

### 7.3 競合・異常吸収
- cancel ack不明/timeout→Unknown→照合で確定
- WS逆順・重複・遅延→inbox dedupe + 状態機械で吸収

### 7.4 Reconcile（照合）
open orders/trades/balances/positions を定期＋異常時で照合し収束

TODO:
- 状態遷移表（イベント→遷移→副作用）と禁止遷移の明文化
- Unknownの最大滞留時間/再照合頻度/照合優先順位

---

## 8. モード・段階導入・実行アルゴ（固定）
- paper/shadow/live同一経路
- rollout：paper→shadow→canary→full（自動停止あり）
- 標準実行：slicing、peg/post-only維持、anti-slippage、TTL、quote protection、impact
- SOR/Netting/学習系は明示ONのみ（暗黙禁止）

TODO:
- “live誤爆は物理防止”の実装要件（鍵/環境/HA role/権限の具体条件）
- rollout自動停止のトリガ条件（例：SLO違反、health低下、監査不整合）

---

## 9. 口座・権限・監査・改ざん耐性（固定）
- account_ref単位で分離（rate/照合/Unknown）
- policy（bot/user）で新規・取消・live切替を強制
- auditは principal/commit/config digest を必ず記録
- tamper-proof：hash chain + seal（任意署名枠）
- 改ざん検知時は原則 HALT/CANCEL_ONLY + forensics

TODO:
- principal/commit/config digest の形式（算出手順、格納先、検証頻度）
- “forensics bundle”の構成（最低限のログ/証跡/検証手順）

---

## 10. 再起動・二重起動・DR（固定）
- bootstrap：outbox再送、inflight Unknown化、照合で収束
- HA dual-run：leaderのみlive送信、standbyはshadow追従
- DR：復旧手順とRTO/RPO、DR GameDayで証明

TODO:
- RTO/RPOの目標値（数値）と測定方法
- GameDayの頻度・シナリオ・合格基準（CI成果物との対応）

---

## 11. NFR（固定）
- 性能契約（p95/p99）＋回帰
- 容量上限＋安全なload shedding
- outbox/inbox/events耐久性（低耐久はlive禁止）
- schema evolution（互換性破壊はlive禁止）
- security boundary（秘密漏洩ゼロ、短寿命、境界固定）
- supply chain（pin/SBOM/rollout）
- forensics bundle
- retention/compaction/archive
- dependency failure時は“送らない”

TODO:
- p95/p99の対象（API/venue別/レーン別）、負荷条件、測定環境
- “低耐久”の判定（ストレージ種別・冗長度・整合性保証の要件）

---

## 12. Assurance（固定：完成を証明する）
- E2E不変条件（総量保存、Lane0優先、leader-only live、Unknown収束、監査完全性）
- Evidence bundle（規格ファイルでCI成果物化）
- Golden trace scenarios（主要異常系）
- Security Verification / Contract Compliance / Release Gate
- Independent Audit Mode（read-only検証）

TODO:
- Golden traceの具体シナリオ一覧（入力→期待状態→期待監査→期待照合結果）
- Evidence bundleのスキーマ（assurance/*.jsonの必須フィールド）

---

## 13. 外部契約＆運用SSOT（固定）
- Interface contracts（Policy/Risk, Portfolio, DataQuality, Secrets, Observability）
- Runbook SSOT（Unknown、429雪崩、WS欠損、HA/DR、鍵ローテ、誤発注疑い等）
- Telemetry Canon、Production config policy、Change management、Incident taxonomy、Do Not Trade windows、Governance

TODO:
- 各Interface contractの参照先（ファイル名/エンドポイント/スキーマ）とゲート条件
- Runbookの格納場所と最低限の章立て（観測→判断→自動復旧→手動→監査→再発防止）

---

## 14. 最終セーフティ保証（固定）
- Threat model（防御/検知/復旧/証跡）
- Safe defaults（shadow、market禁止、明示ONのみ）
- Hardened runtime profile（権限分離/allowlist/ログ保護）
- Tamper response / Secrets compromise即応 / Market data integrity枠
- Policy/config tamper-proof（digest不一致はlive禁止）
- Emergency override（期限・理由・二人承認・縮退）
- Dependency failure safety（耐久不足はlive禁止）

TODO:
- overrideの運用フロー（承認者、ログ要件、期限切れ時の強制無効化）
- market禁止の例外有無（あるなら条件、ないなら明記）

---

## 15. I-100 Complete Safety Gate（最終ゲート：抜け道ゼロ）
**I-100**: live送信直前に必ずチェックし、PASSしない限り送らない。
1. HA leader
2. mode=live明示
3. policy=ALLOW（期限内）
4. guards PASS（data/impact/portfolio）
5. reservation確保済み
6. durability live許可
7. schema互換 PASS
8. health/SLOがCRITICALでない（例外はoverride）
9. audit chain/ seal 検証 PASS

TODO:
- “PASS”判定の具体条件（入力、参照データ、閾値、理由コード）
- I-100の実装配置（どのプロセス/どの境界で強制するか）

---

## 16. 付録（SSOT補助：I-101〜I-103）
- **I-101 Index**：章→モジュール→データ→証明の索引
- **I-102 Dictionaries**：状態/理由コード/エラー分類の正規辞書
- **I-103 Reference Patterns**：outbox/inbox、SMテンプレ、reservation台帳、safety gate順序

TODO:
- I-101の具体項目（リンク/パス/キー）
- I-102の辞書本体（列、正規化規約、互換性方針）
- I-103のテンプレ本文（最小実装・禁止事項）

---

## 17. 最終DoD
- 二重発注ゼロ、Unknown収束、Lane0飢餓ゼロ、live誤爆不可能
- 監査改ざん耐性＋replay/lineageが揃う
- 再起動/二重起動/DRでも二重発注しない
- NFRゲートとAssuranceゲートがCIでPASS
- 外部契約と運用SSOTが揃い、最終ゲート（I-100）に抜け道がない

TODO:
- DoDの検証手段（どのテスト/どのCI成果物/どのメトリクスで合格とするか）をI-101へ接続

---

## Capability Index（ID保持）
- **I-100** — Complete Safety Gate（最終ゲート：抜け道ゼロ）
- **I-101** — Index（章→モジュール→データ→証明の索引）
- **I-102** — Dictionaries（状態/理由コード/エラー分類の正規辞書）
- **I-103** — Reference Patterns（outbox/inbox、SMテンプレ、reservation台帳、safety gate順序）
