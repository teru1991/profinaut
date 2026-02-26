# Level 2 Deep Spec — J. Risk Policy Gate

> 整理のみ / 新規仕様追加なし。未記載は TODO。

## 1. Non-negotiable（固定ルール）カタログ
### 1.1 変更は必ずChange Workflowを通す
- 直編集禁止（SSOTファイル群は change request workflow 必須）。
- 必須ステップ: CR → Lint → Invariant → Regression(含FireDrill) → Shadow/Canary/Enforce → SLO監視/自動ロールバック。

### 1.2 互換性破壊は投入前に停止
- Policy DSL / Metrics schema の破壊的変更は検出して停止。
- capability negotiation で縮退表現へ。
- TODO: capability negotiation の具体I/F、ネゴシエーション失敗時のデフォルト

### 1.3 安全側デフォルト
- タイムアウトは安全側（DENY/CLOSE_ONLY）。
- unknown metric / schema mismatch は fail-close（CLOSE_ONLY以上）。

### 1.4 重大操作の強制制約
- break-glass は timebox + 監査強制 + 使用中は自動厳格化。
- Forbidden Ops は仕様で禁止、未遂も監査ログ化。

---

## 2. Canonical Model / Contract（SSOT構造の正準）
### 2.1 正本と参照SSOT
- 正本本文: `docs/specs/domains/J_risk_policy_gate.md`（全体仕様の唯一正）。
- 参照SSOT: `docs/specs/domains/J/*` の固定ファイル群（境界/理由/状態/例外/観測/RBAC/無人帯/禁止/失敗モード/依存SLO/縮退/保持/ブートストラップ）。

### 2.2 観測・監査・再現性
- 観測契約: `observability_contract.yml`（Metrics/Logs/Traces Contract）。
- 監査要件: mode_source / explain最小スキーマ / 監査ハッシュチェーン必須。
- 保持/マスキング: `retention_redaction.md` で固定。
- TODO: Explain最小スキーマ（フィールド一覧）、hash chain の計算方式、Repro Pack の構成

---

## 3. Behavior / Tests（保証手段の正準）
### 3.1 CIゲート（必須）
- SSOT Linter（整合Lint）
- Safety Invariant Suite（安全不変条件）
- Regression（Golden + Adversarial + Fire Drill）

### 3.2 本番ゲート（常時）
- Continuous Self-Check（決定論/依存健全/監査連続/SLO予兆/SSOTロード整合）。

---

## 4. Delivery（DoD & Evidence）
### 4.1 DoD（完了定義の構造）
- 機能チェック + 必須テスト + 必須観測 + 必須監査。

### 4.2 Evidence Pack（証跡の構造）
- CIログ、代表Repro（SEV想定）、Fire Drillログ、依存故障シミュレーション、負荷試験、監査ハッシュチェーン検証結果。

---

## 5. TODO（元文書に無い“埋めるべき空欄”）
> 追加仕様は作らず、「元文書が参照しているが詳細未記載」な箇所のみ列挙。
- TODO: 3層ゲートの詳細（名称/入力/出力/優先順位/状態機械との結合点）
- TODO: capability negotiation のI/Fと失敗時デフォルト
- TODO: Quiet Hours の定義と Re-Arm 段階仕様
- TODO: Forbidden Ops の具体リスト
- TODO: Explain最小スキーマ、監査ハッシュチェーン方式、Repro Pack 構成
- TODO: latency budget の測定点、degraded levels の定義、状態引継ぎ一貫性モデル
- TODO: Invariant/Lint ルールのコード化（エラーコード・重大度・fail条件）
- TODO: Canary判定条件、SLO逸脱閾値、部分ロールバック単位
- TODO: 実装フェーズごとの受入条件（exit criteria）と成果物一覧
