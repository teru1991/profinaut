# Domain Core Spec Template v1.0（固定）
Document ID: SYS-DOMAIN-SPEC-TEMPLATE
Status: Canonical / Fixed Contract
Scope: A〜Y 各ドメインの「Core Spec（固定仕様）」を UCEL Golden 並みの粒度で統一するためのテンプレ

## 0. 目的（Non-negotiable）
本テンプレは、ドメインごとの仕様品質を揃え、SSOT分裂と設計の抜け漏れを防ぐ。
- どのドメインでも「同じ型」で読める
- Core（不変）に可変の閾値を混ぜない
- crosscut（Safety/Audit/Bundle）と contracts（JSON Schema）へ必ず接続する
- 実装・運用・サポートまで “導線” が通っている（trace/runbook/evidence）

---

## 1. 使い方（必須ルール）
### 1.1 ドキュメント種別（SSOTレイヤ）
- Core Spec（固定）: `docs/specs/**`
- Policy（運用値）: `docs/policy/**`
- Plan（計画）: `docs/plans/**`
- Runbook（手順）: `docs/runbooks/**`
- Contract（機械契約）: `docs/contracts/*.schema.json`（唯一の正）

**Core Specに書いてはいけないもの**
- 閾値（ms/%/回数/保持期間/接続数/上限値）
- フェーズ、進捗、TODO、担当
- 手順（復旧/運用の段取り）
→ これらは Policy / Plan / Runbook に外出しする（必須）

### 1.2 ドメイン所属（必須）
各 Core Spec は、必ず `docs/specs/system/domains_ssot.md` の A〜Y のどれかに所属する（Belongs-to を明記）。

### 1.3 crosscut 参照（必須）
各ドメインは “上書き” せず、必ず参照して統一する：
- Safety: `docs/specs/crosscut/safety_interlock_spec.md`
- Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
- Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`

### 1.4 Environment/Mode 参照（必須）
誤爆防止の意味は固定（上書き禁止）：
- `docs/specs/system/environment_mode_matrix.md`

### 1.5 Gate/Trace（必須）
- docs品質ゲート要件: `docs/specs/system/docs_gate_spec.md`
- 正本リンクSSOT: `docs/status/trace-index.json`
  - 新規Core Spec追加時は、trace-index に “正本導線” を追加する（推奨ではなく必須）

---

## 2. ファイル命名（推奨）
- Core Spec: `<domain_dir>/<topic>_spec.md` または `<domain_dir>/<domain>_core_spec.md`
- Directory（推奨）:
  - `docs/specs/<domain_name>/...`
  - ただし SSOT 境界は “domains_ssot” が正。ディレクトリは都合でよい。

---

## 3. Core Spec のテンプレ（貼り付け用）
以下を各ドメインの Core Spec 冒頭に貼り、内容を埋める。

⸻

# <DOMAIN_NAME> Core Spec v<MAJOR>.<MINOR>（固定仕様）
<短い副題（例：OMS/EMS：唯一の発注出口）>

- Document ID: <UNIQUE_ID>
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): <A〜Y のいずれか（複数可）>
- Crosscut dependencies（固定）:
  - Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
- System dependencies（固定）:
  - Domains SSOT: `docs/specs/system/domains_ssot.md`
  - Environment/Mode: `docs/specs/system/environment_mode_matrix.md`
  - Versioning: `docs/specs/system/versioning_policy.md`
  - Terminology: `docs/specs/system/terminology.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/*.schema.json`（該当するものを列挙）
- Non-goals（本書で固定しない）:
  - 閾値（ms/%/保持期間/上限等）→ `docs/policy/**`
  - 手順（復旧/運用）→ `docs/runbooks/**`
  - 計画（フェーズ/進捗）→ `docs/plans/**`

---

## 0. 目的と到達点（Non-negotiable）
- <必達要件を箇条書きで>
- <「壊れると何が終わるか」＝Fail時の破局点を明示>

---

## 1. 責務境界（In / Out）
### 1.1 In Scope（このドメインが責任を持つ）
- <箇条書き>

### 1.2 Out of Scope（隣接ドメインに委譲する）
- <箇条書き（委譲先ドメインも書く）>

### 1.3 依存関係（Depends-on）
- MUST dependencies（必須）: <例：A/B/C/G/E...>
- SHOULD dependencies（推奨）: <例：D/Y...>
- 依存理由（短文）: <なぜ必要か>

---

## 2. カノニカル概念（固定語彙）
- <このドメイン固有の用語・状態名・イベント名>
- 用語が system/terminology と衝突する場合は、terminology を正として合わせる

---

## 3. アーキテクチャ（固定：概念層）
### 3.1 コンポーネント境界
- <構成要素の責務と依存方向（矢印の向き）>

### 3.2 状態機械（必要なら必須）
- <状態一覧>
- <遷移条件>
- <禁止遷移>
- <安全連動（SAFE/CLOSE_ONLY/FLATTEN/HALT等）>

### 3.3 シーケンス（正常系 / 異常系）
- 正常系: <入力→処理→出力>
- 異常系: <429 / reconnect / partial fill / reorg / IO stall 等>

---

## 4. データ / イベント / 契約（固定）
### 4.1 入力（Input）
- sources: <WS/REST/DB/Queue/Files>
- key fields: <trace_id/run_id/schema_version/event_uid 等>

### 4.2 出力（Output）
- outputs: <streams/tables/files/events>
- contracts: <docs/contracts の該当 schema>

### 4.3 互換性（SemVer）
- breaking / additive / patch の定義
- schema_version を変える条件（contractsの規約に従う）

---

## 5. 安全（crosscut連動：固定）
- Safety interlock に従う（上書き禁止）
- 危険操作（dangerous op）の定義と challenge/confirm の前提
- secret-free の保証（ログ/監査/バンドル）

---

## 6. 観測（固定：何を出すか）
- metrics（必須）: <SLO/SLI につながるもの>
- logs（必須）: <構造化・相関ID>
- traces（推奨）: <span境界>
- healthz/capabilities（必須）: <未知＝UNKNOWN扱い>

---

## 7. 失敗モード（固定：標準エラーモデルへのマップ）
- Failure Mode → Error Kind → retryable → safety impact → runbook
- Unknown を常態化させない（Unknown増加は設計欠陥）

---

## 8. テスト / 検証観点（固定）
- Contract tests（schema一致）
- E2E golden（paper→shadow→live 安全保証）
- Chaos / Fault injection（429/遅延/欠損/reorg 等）
- Deterministic replay（同入力→同出力）

---

## 9. Policy / Plan / Runbook への分離点（固定）
- Policyへ外出しする値: <一覧>
- Planへ外出しする事項: <一覧>
- Runbookへ外出しする手順: <一覧>
- 参照リンク: <該当ファイルパス>

---

## 10. DoD（このCore Specが“完成”とみなされる条件）
1) Non-negotiable が明確
2) In/Out が隣接ドメインと矛盾しない
3) 契約（contracts）と crosscut に接続している
4) 失敗モードが標準エラーモデルへマップされている
5) テスト観点が書かれている
6) 可変情報がCoreに混入していない

⸻

---

## 4. 付録：Policy / Plan / Runbook の最小テンプレ（参考）
### 4.1 Policy（推奨テンプレ）
- Document ID / Status / Scope
- Tunables（閾値/上限/保持/通知感度）
- Change log（Policy内に短く記録）

### 4.2 Plan（推奨テンプレ）
- 目的 / フェーズ / マイルストーン / 依存 / 完了条件

### 4.3 Runbook（推奨テンプレ）
- Symptoms / Observations / Auto-recovery / Manual actions / Evidence / Rollback / Postmortem
- runbook index: `docs/runbooks/README.md`

---
End of document
