# Level 1 SSOT Outline — G. Data Contracts / Schema Governance（実装目標機能詳細設計 v1.6）

- ドメイン文字: **G**
- 対象: **Schema Governance / Data Contracts**
- バージョン: **v1.6（最終清書・SSOT）**
- 目的: 全ドメイン（MarketData / Execution / Portfolio / Bot / Dashboard / Audit …）のデータI/Fを破壊から守る統治レイヤをSSOTとして定義する。
- 方針: 本Outlineは入力の整理のみ。不足は **TODO:** として残す（推測で補完しない）。

---

## 1. Scope & Intent（目的と位置づけ）
### 1.1 ミッション（不変）
- G（Schema Governance / Data Contracts）は、以下を一体として提供し「壊したら必ず止まる」状態を作る。
  - 仕様（Schema）
  - 互換性（Compatibility）
  - 契約テスト（Contract Tests）
  - CIゲート（Gate）
  - 依存グラフ（Dependency）
  - 移行（Migration）
  - 実行時防衛（Runtime Enforcement / Schema Firewall）

### 1.2 「契約」に含める範囲（型定義だけではない）
- 意味（単位/精度/丸め/刻み/時刻意味/順序）
- ポリシー（PII/秘匿/float禁止等）
- 性能（予算）
- 運用（deprecation/sunset/canary/反エントロピー/凍結）
- 品質（SLO/ドリフト/継続検証）
- 供給網（署名/SBOM/再現性）
- 緊急停止（キルスイッチ）
- 混在防止（backfill/契約セット固定）
- 人間運用（SLA/承認/教育）
- ゼロトラスト境界（内部境界含む）
- ※上記は「機械的に強制」する前提。

---

## 2. Non-negotiable（到達目標）
### 2.1 保証事項（要約）
以下を必ず満たす（PR/CI/Runtime/運用を含む）。
- 契約SSOTが一意に存在（Registry + Schemas + Rules + Locks + Policies + Tests + Releases）
- 互換性はPRごとに機械判定＋根拠生成
- SemVer＋Deprecation/Sunset（LTS可）が強制
- Producer/Consumer両面の契約テスト必須＋網羅率ゲート
- 依存関係可視化＋影響分析自動化（未対応はマージ不可）
- BREAKINGはメジャー＋移行（dual read/write）＋CCP＋承認＋監査証跡一式
- 実行時に mismatch/invalid を検知・隔離・縮退（Schema Firewall）
- 意味・時刻意味・順序・去重・分類/決定的マスキングを契約として強制
- 性能予算で肥大化を止める
- ドリフト検知＋継続検証でマージ後の破壊も検出
- Canary＋環境別ゲート＋凍結期間で本番事故を最小化
- 契約セットは署名され、実行時に検証される（改竄耐性）
- 契約ビルドは再現性を持ち、SBOMで透明性が担保される
- 反エントロピーで“部分的に古い契約”を撲滅
- Contract-firstで契約後追いを禁止
- backfillは契約セット固定で混在を防止
- キルスイッチで危険契約を即時拒否（監査付き）
- 影響半径予算（Blast Radius Budget）で巨大変更を抑止
- ロールバック手順を決定的プロトコルとして規格化
- 内部境界もゼロトラストで常に契約検証を通す

---

## 3. SSOT Artifacts（成果物）
### 3.1 推奨ディレクトリ（概念固定）
- `docs/contracts/`：方針/契約一覧/移行/運用/監査/教育/Runbook導線
- `contracts/`：機械向けSSOT
- `schemas/`
- `registry/registry.json`
- `rules/compatibility.yaml`
- `locks/consumers.lock`
- `fixtures/`（golden/edge/fuzz corpus）
- `replay/`（匿名化リプレイ）
- `normalization/`
- `policies/`
- `releases/`（lock + signature + sbom + changelog）
- `reports/`（compat_report/audit_pack/coverage_report/rollout_plan）
- `tests/`
- `retention/`（保持・削除規約：G-67）
- `tools/schema/`：lint/diff/impact/check/release/pack/simulate/remediate/attest/chaos/rollback CLI
- `.github/workflows/`：Gate + 定期検証 + backfill検証 + freeze enforcement

### 3.2 必須メタ（全イベント/レコード）
- `schema_id`, `schema_version`, `producer`
- `event_time`, `recv_time`, `persist_time`
- `trace_id`, `run_id`
- `seq`（可能なら必須）
- `event_id`, `dedup_key`
- `data_classification`（PUBLIC/INTERNAL/CONFIDENTIAL/SECRET）
- `source_ref`（主要フィールドは必須）
- `contract_set_id`（署名参照）
- （対象限定）`schema_digest` / `schema_locator`（自己記述：G-68）

---

## 4. Governance Rules（統治ルール）
### 4.1 禁止・強制・例外
- required追加、enum削除、意味変更は **MAJOR必須**
- 同一 `schema_id` で意味変更禁止
- SECRET/PIIは分類・決定的redaction・経路検査なしで流通禁止
- float禁止（例外はCCP）
- 性能予算超過は禁止（例外はCCP）
- 本番は canary＋SLO監視＋自動ロールバック
- LTS schema（任意）：変更制限強化
- Freeze window（凍結期間）：MAJOR禁止（例外は緊急手続き＋監査）

---

## 5. Capabilities（機能分解）
### 5.1 G-01〜G-60（既存要件を保持）
- G-01〜G-60は「v1.5の全要件を保持」として、以下の機能群を含む（要約列挙）。
  - ID体系、正本、SemVer/廃止、互換性判定、契約テスト、Registry、依存/影響、移行、CI Gate、実行時防衛、CCP、Release Train、Negotiation、Meaning/Time/Ordering/Perf、Replay、Provenance、Normalization、Error/Policy/Security/Dedup、Capability、Reports、SLO、Multi-repo、Drift、Firewall、Canary、Formal Proof、Evolution Simulation、Auto-Remediation、Lineage、Multi-env、Ownership、Test Coverage Gate、Attestation（署名）、Repro Build、SBOM、Disaster Mode、Audit Pack、Deterministic Redaction、UX、LTS、Changelog、Continuous Verification、Contract-first、Anti-entropy、Chaos、Invariants、Auto-rollout planner、Backfill guarantees、Cross-domain convergence、Deprecation enforcement、Incident linking、Multi-layer validation、Kill switch、AI-assisted review

**TODO:** G-01〜G-60 を「IDごとの個別定義（目的/入力/出力/ゲート/DoD/例外）」へ分解した一覧（本ファイルには要約しか存在しない）。

### 5.2 追加強化（G-61〜G-72）
#### G-61 Contract SLAs for Humans（人間向けSLA）
- 目的: 技術で止めても対応が遅い事故を防ぐ
- 要件:
  - CCP種別ごとにレビュー期限/移行期限/sunset期限をSLA化
  - 期限超過は自動エスカレーション（Issue化、Gate強化、Kill-switch推奨）
- DoD: 期限超過が放置されない

#### G-62 Two-Person Rule（重要契約の二重承認）
- 目的: 重大契約の単独ミスを防止
- 要件:
  - LTS/Execution/Portfolio/Audit等の重要schemaは2人承認必須
  - 1人運用時は形式だけ固定し、将来の運用に備える
- DoD: 重要変更は二重承認なしで通らない

#### G-63 Contract Freeze Windows（凍結期間）
- 目的: 重大イベント時に契約変更リスクを抑止
- 要件:
  - Freeze期間はMAJOR禁止、例外は緊急手続き＋監査必須
- DoD: Freeze中の破壊的変更は原則ゼロ

#### G-64 Blast Radius Budget（影響半径の予算）
- 目的: 影響範囲が大きすぎる変更を抑止
- 要件:
  - 影響consumer数/topic数/DBテーブル数に上限
  - 超過は分割を強制（CCPで段階計画）
- DoD: 巨大変更は段階化される

#### G-65 Multi-Version Runtime Compatibility（N世代同時受理）
- 目的: 長期移行や外部consumerでも破綻しない
- 要件:
  - ランタイムでN世代（例：2〜3）受理/変換
  - 受理範囲と期限を契約化（無制限併存は禁止）
- DoD: 併存は管理され、期限で収束する

#### G-66 Deterministic Contract Rollback Protocol（巻き戻し規格）
- 目的: ロールバックを人間芸にしない
- 要件:
  - rollback手順を定型化（チェックリスト/自動化）
  - rollback後の整合検証（サンプル検証・ドリフト確認）必須
- DoD: ロールバックが確実・再現可能

#### G-67 Contract Data Retention & Purge Rules（保持・削除契約）
- 目的: 規制/秘匿/容量の観点で保持が曖昧になるのを防ぐ
- 要件:
  - schemaごとに保持期間、匿名化要否、削除手順を契約化
  - replay/backfillが保持規約を破らないようゲート化
- DoD: 保持・削除が常に説明可能

#### G-68 Self-Describing Payload Mode（自己記述ペイロード）
- 目的: 長期保存・外部連携で仕様が見つからない問題を防ぐ
- 要件:
  - payloadに `schema_digest` / `schema_locator` / `contract_set_id` を含める（対象限定）
- DoD: 長期保存でも復元可能（対象限定でサイズ増は制御）

#### G-69 Contract Federation（契約の連邦）
- 目的: 別プロジェクト/別組織と連携しても統治を維持
- 要件:
  - 外部registryと相互参照/署名検証
  - 信頼境界（何を信じ、何を検証するか）を契約化
- DoD: 外部連携でもゼロトラストで成立

#### G-70 Breaking Change Containment（破壊変更の封じ込め）
- 目的: 破壊が必要でも被害を局所化
- 要件:
  - BREAKINGは新topic/新streamへ隔離し旧を維持
  - 旧→新ブリッジを用意し段階移行
- DoD: 破壊の被害が局所化される

#### G-71 Contract Education Pack（教育パック）
- 目的: ルール複雑化による運用ミスを防ぐ
- 要件:
  - チュートリアル/チェックリスト/典型事故集/レビュー観点をdocsに同梱
- DoD: 新規参加者でも運用が再現できる

#### G-72 Zero-Trust Contracts for Internal Boundaries（内部境界ゼロトラスト）
- 目的: 内部サービス間も信用せず必ず契約検証を通す
- 要件:
  - 内部RPC/Kafka/DB書き込みもFirewall/validate対象
  - 例外は極小化し、例外は必ず監査ログに残す
- DoD: 内部経路の“抜け道”がない

---

## 6. Implementation Plan（最小実装順序）
成立→最強の順に実装する。
1. G-01 + G-06
2. G-09
3. G-04
4. G-05
5. G-07
6. G-08
7. G-10
8. G-11〜G-16
9. G-29〜G-38
10. G-39〜G-48
11. G-49〜G-60
12. G-61〜G-72（運用ガバナンス最終層）

**TODO:** 各ステップの「前提/成果物/ゲート/最小DoD」詳細。

---

## 7. Definition of Done（完成条件：Gを“完全実装”扱いにする最終チェック）
以下が運用されていること。
- PRで compat判定＋根拠＋影響分析＋レポート＋changelog＋（必要なら）CCP/承認
- 未登録/必須メタ欠落は即fail
- Meaning/Time/Ordering/Dedup/Security/Policyが機械強制
- Runtime mismatch/invalid は隔離/縮退し観測・監査に残る
- Drift＋継続検証が稼働
- Canary＋環境別ゲート＋Freezeが運用される
- 署名＋再現性＋SBOMで供給網が強固
- Backfillは契約セット固定
- Anti-entropyで契約ズレを撲滅
- Chaos testingで未知入力耐性を保証
- Kill switchで緊急停止が監査付きで可能
- Blast radius budgetで巨大変更が抑止
- Rollbackプロトコルが定型化され、検証も必須
- 保持/削除契約が説明可能
- 内部境界もゼロトラストで抜け道がない

---

## 8. Operations（docs更新検知）
- docs変更通知は差分検知→通知を自動化（GitHub Actionsで `docs/**` 変更を検知し、PRコメント/Issue/Webhookで通知）
- G-09-06（docs追随チェック）と併用し、docs未更新の契約変更PRは落とす

**TODO:** 「G-09-06」の正式定義（本ファイルでは参照のみ）。

---

## 9. Capability Index（ID保持）
### 9.1 機能ID
- G-01〜G-60（既存要件一式：個別内訳はTODO）
- G-61 Contract SLAs for Humans
- G-62 Two-Person Rule
- G-63 Contract Freeze Windows
- G-64 Blast Radius Budget
- G-65 Multi-Version Runtime Compatibility
- G-66 Deterministic Contract Rollback Protocol
- G-67 Contract Data Retention & Purge Rules
- G-68 Self-Describing Payload Mode
- G-69 Contract Federation
- G-70 Breaking Change Containment
- G-71 Contract Education Pack
- G-72 Zero-Trust Contracts for Internal Boundaries
