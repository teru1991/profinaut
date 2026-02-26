# Level 1 SSOT Outline — J（Risk / Policy Gate）SSOT

## 0. メタ情報
- ドメイン文字: **J**
- 元文書: **J. Risk / Policy Gate — 実装・運用・品質保証・納品 SSOT（清書・最終）**
- 位置づけ（原文要約）
  - **Jの機能要件は網羅済み**で、追加の“機能”は不要。残るリスク（実装ミス/更新で破壊/運用ブレ/証跡不足）を潰すための**固定ルール（ガバナンス）**を定義する。

---

## 1. SSOT（Single Source of Truth）
### 1.1 SSOTパッケージ構成（Jの正本）
- Jは「本文1本」＋「固定SSOTファイル群」で成立する。

#### 1.1.1 正本（本文）
- `docs/specs/domains/J_risk_policy_gate.md`
  - Jの全体仕様（3層ゲート、例外、優先順位、状態機械、I/F、監査、運用導線）
  - TODO: 3層ゲートの具体名/入力/出力/優先順位の詳細（本ドキュメントには列挙のみ）

#### 1.1.2 固定SSOTファイル群（Jが参照する“動かせない正本”）
- 配置: `docs/specs/domains/J/` 配下（参照は必ずここから）
1) 境界値: `boundaries.yml`（境界値カタログ）
2) 理由コード: `reason_codes.yml`（Reason Code Registry）
3) 状態機械: `mode_machine.yml`（Mode & Transition Table）
4) 例外テンプレ: `exception_templates.yml`（Exception Template Library）
5) 観測契約: `observability_contract.yml`（Metrics/Logs/Traces Contract）
6) 権限: `rbac_matrix.yml`（RBAC Matrix SSOT）
7) 無人運用帯: `quiet_hours.yml`（Quiet Hours / Autopilot Policy）
8) 禁止操作: `forbidden_ops.yml`（Forbidden Operations List）
9) 失敗モード: `failure_modes.md`（FMEA / フォールトツリー）
10) 依存先RTO/RPO: `dependency_slo.yml`（Dependency RTO/RPO）
11) デグレード段階: `degraded_levels.yml`（Degraded Mode Levels）
12) 保持/マスキング: `retention_redaction.md`（Retention & Redaction）
13) ブートストラップ: `bootstrap.md`（初回起動の最小セット）

- 重要（変更統制）
  - これらSSOTは **change request workflow（申請→レビュー→回帰→canary→本番）必須**、**直接変更は禁止**。

---

## 2. ガバナンス（Non-negotiable / Fixed Rules）
### 2.1 変更管理（Change Workflow SSOT）
- 対象: policy/例外/境界値/理由コード/状態機械/権限/禁止操作/無人帯/観測/保持 等、**J関連の変更はすべて**対象。

#### 2.1.1 必須ステップ（固定）
1) 申請（Change Request）
2) SSOT Linter（整合Lint）合格
3) Safety Invariant Suite（安全不変条件テスト）合格
4) 回帰（Golden + Adversarial + Fire Drill）合格
5) Shadow → Canary → Enforce（段階導入）
6) SLO監視（逸脱なら自動ロールバック：blast radius付き部分ロールバック）

#### 2.1.2 互換性ゲート（固定）
- Policy DSL / Metrics schema の**破壊的変更は投入前に必ず検出して停止**。
- 下流（I/L/M/K 等）の能力差は **capability negotiation** で安全に縮退表現へ落とす。
- TODO: capability negotiation の具体I/F、ネゴシエーション失敗時のデフォルト

### 2.2 運用固定（人間事故を潰す）
#### 2.2.1 RBAC（誰が何をできるか）
- 重大操作（解除、treasury解除、compliance override 等）は強く制限。
- break-glass は timebox + 監査強制 + 使用中は自動厳格化。
- TODO: 重大操作の定義一覧、役割→権限の最小権限設計

#### 2.2.2 Quiet Hours（無人帯）
- 無人帯は攻めない：新規を寄せる（REDUCED/CLOSE_ONLY）、高リスク戦略隔離、treasury禁止。
- quiet hours 終了後も即NORMAL禁止（Re-Arm段階復帰）。
- TODO: Quiet Hours の定義（曜日/時間帯/例外条件）、Re-Arm 段階の仕様

#### 2.2.3 Forbidden Ops（最悪手順を禁止）
- HALT中の例外で新規許可、監査不健全なのに解除、隔離解除即NORMAL 等を仕様で禁止。
- 未遂も監査ログ化。
- TODO: forbidden_ops の具体リストと検出ポイント

---

## 3. Canonical Model / Contracts（正準モデル・契約）
### 3.1 主要SSOT（契約）一覧
- 1.1.2 の固定SSOTファイル群が **正準契約**（境界値/理由コード/状態遷移/例外/観測/RBAC/無人帯/禁止/失敗モード/依存SLO/デグレード/保持/ブートストラップ）。

### 3.2 監査・再現性・保持（契約）
- `mode_source` 必須、証跡（Explain最小スキーマ）必須、監査ハッシュチェーン必須。
- Repro Pack / コーパス（採用する場合）の保持・マスキングを `retention_redaction.md` で固定。
- 監査（O）不健全時の扱いは `dependency_slo.yml` に固定（厳格運用なら停止も許容）。
- TODO: Explain最小スキーマ（フィールド一覧）、hash chain の計算方式、Repro Pack の構成

### 3.3 性能と可用性（契約）
- pre-trade のレイテンシ予算（p99）を boundary で固定。
- タイムアウト時は安全側（DENY/CLOSE_ONLY）。
- 負荷時は Degraded Levels に従い段階縮退（Level1→2→3）。
- HA（failover）時も状態（mode/例外/隔離/水位）を引継ぎ、split-brain防止。
- TODO: latency budget の単位/測定点、degraded level の定義、状態引継ぎの一貫性モデル

---

## 4. Behavior / Tests（品質保証・動作検証）
### 4.1 品質維持（壊れないことを自動保証）
#### 4.1.1 Safety Invariant Suite（CI必須）
Jが破ってはいけない不変条件（例）
- Safety Envelope 超過は必ず拒否
- EがHALTならJは必ずDENY
- unknown metric / schema mismatch は fail-close（CLOSE_ONLY以上）
- break-glass は期限で必ず失効
- quarantine解除は規約条件を満たさない限り不可
- 同一snapshotで同一decision（決定論封印）
- TODO: 各Invariantの形式仕様（入力→期待結果→例外）とテストケース最小セット

#### 4.1.2 SSOT Linter（CI必須）
投入前にSSOT矛盾を止める（例）
- reason_code に runbook_ref が無い、などを禁止
- mode遷移表の到達不能/解除不能/禁止ループを禁止
- boundaries の単位欠落・範囲外を禁止
- exceptionテンプレ必須項目欠落を禁止
- observability契約と実装の不一致を禁止
- TODO: Lintルールのエラーコード、重大度、CIのfail条件

#### 4.1.3 Continuous Self-Check（本番常時）
運用中も定期自己診断し、失敗時は安全側へ
- 決定論チェック（同一snapshot再評価一致）
- 依存先健全性（K/H/I/Oなど）
- 監査ハッシュチェーンの連続性
- SLO/予兆（preincident連携）
- SSOTロード整合（lint相当の軽量確認）
- TODO: self-check の周期、失敗時のmode遷移、通知/自動復旧の仕様

### 4.2 回帰・導入・ロールバック（段階運用テスト）
- 回帰は Golden + Adversarial + Fire Drill を必須とする。
- 導入は Shadow → Canary → Enforce。
- 逸脱時は自動ロールバック（blast radius付き部分ロールバック）。
- TODO: Canary判定条件、SLO逸脱の閾値、部分ロールバックの単位

---

## 5. 納品（DoD・証跡）
### 5.1 J Domain DoD（完了定義）
- 必須機能チェック（pre/in/post、例外、E優先、determinism、quarantine、treasury等）
- 必須テスト（Invariant/Lint/Regression/Adversarial/FireDrill）
- 必須観測（observability契約準拠）
- 必須監査（hash chain / mode_source / explain最小スキーマ）

- TODO: “必須機能チェック”のチェックリスト（項目の正確な列挙）

### 5.2 Verification Evidence Pack（検証証跡）
- CIログ（invariant/lint/regression）
- 代表Repro（SEV想定）
- Fire Drillログ
- 依存先故障シミュレーション
- 負荷試験（degraded→safe-mode）
- 監査ハッシュチェーン検証結果
- TODO: Evidence Pack の格納先、命名規則、最低保持期間

---

## 6. 実装計画（推奨フェーズ）
固定の推奨フェーズ（安全側から広げる）
1) CLOSE_ONLY固定で枠作成
2) pre-trade（ALLOW/DENY）＋Safety Envelope＋監査
3) in-run段階縮退＋Explain最小
4) post-trade違反分類＋自動処置
5) RBAC/禁止操作/Quiet Hours/例外テンプレ
6) 互換性ゲート＋SSOT Linter＋Invariant suite
7) Quarantine/Blast radius＋依存障害＋self-protection
8) Treasury/Onchain/Oracle/Cross-venue＋Cost/Compliance
9) HA/failover＋SLO最適化

- 注: 参照評価器/差分検査/再現パック/コーパスは必要に応じて追加。
- TODO: 各フェーズの受入条件（exit criteria）、成果物一覧

---

## 7. Capability Index（ID保持）
> 文書内に A-xxx / Sxx / T-xx / F-xx / Y-xx 等の明示IDは無し。  
> よって、本ドメイン識別子 **J** と、固定SSOTの“正本ファイル名”を索引として保持する。

- J（Domain）
- `docs/specs/domains/J_risk_policy_gate.md`
- `docs/specs/domains/J/boundaries.yml`
- `docs/specs/domains/J/reason_codes.yml`
- `docs/specs/domains/J/mode_machine.yml`
- `docs/specs/domains/J/exception_templates.yml`
- `docs/specs/domains/J/observability_contract.yml`
- `docs/specs/domains/J/rbac_matrix.yml`
- `docs/specs/domains/J/quiet_hours.yml`
- `docs/specs/domains/J/forbidden_ops.yml`
- `docs/specs/domains/J/failure_modes.md`
- `docs/specs/domains/J/dependency_slo.yml`
- `docs/specs/domains/J/degraded_levels.yml`
- `docs/specs/domains/J/retention_redaction.md`
- `docs/specs/domains/J/bootstrap.md`

---

## 8. 未完了（Jとして残っているもの）
- 機能面: **なし（網羅済み）**
- 残タスク:
  - SSOTファイル群を実ファイルとして作成
  - CIに Invariant / Lint / Regression / Fire Drill を配線
