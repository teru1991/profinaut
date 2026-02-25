# Runtime Execution Safety Core Spec v1.0（固定仕様）
Execution Guard / Kill-Switch / Pre-trade Gate（実行安全）

- Document ID: EXEC-SAFETY-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): C（Runtime Execution Safety / Execution）
- Depends-on（Fixed）:
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/startup_report.schema.json`
- Policy separation（固定しない）:
  - レート制限、サイズ制限、価格乖離閾値、許可銘柄/市場、リトライ上限 → `docs/policy/**`
  - 復旧手順/緊急時オペレーション → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は「実行（注文/取消/変更/ポジション操作）」を **確実安全・堅牢・高速・安定**にするための不変条件を固定する。

必達要件（固定）：
1) **Pre-trade Gate is mandatory**：実行は必ずゲートを通る（例外なし）
2) **No unsafe execution**：System Safety Mode が SAFE/EMERGENCY_STOP のとき実行は禁止（例外は仕様で固定）
3) **Kill-Switch is coherent**：Execution Kill-Switch は明示された語彙で管理され、境界で強制される
4) **Idempotent execution**：重複送信・再試行で二重約定を発生させない（許容できない）
5) **Auditability**：実行の意図・決定・拒否・結果は監査可能（秘密ゼロ）
6) **Failure is safe**：不明・不整合・監査不能は安全側（拒否・SAFEへ）
7) **Performance**：ホットパスは軽量で、観測/監査は非同期で劣化しない（ただし証拠は落とさない）

---

## 1. 実行の責務境界（in / out）
### 1.1 本仕様の対象（in）
- 注文作成（place）
- 注文変更（amend）
- 注文取消（cancel）
- ポジション縮小/クローズ（close）
- 緊急フラット化（flatten intent）
- 実行前判定（pre-trade risk checks）
- Kill-Switch / Safety Mode の強制
- 実行の監査（intent/decision/outcome）

### 1.2 対象外（out）
- 具体的な取引所方言の詳細（UCEL execution connector spec が正）
- 取引戦略の中身（bot domain）
- ストレージ/データレイク詳細（storage domain）
- 監視基盤の具体（observability domain）

---

## 2. 固定語彙（Canonical vocabulary）
### 2.1 System Safety Mode（契約SSOT）
契約 `safety_state.mode` の3つが **唯一の正**：
- `NORMAL`
- `SAFE`
- `EMERGENCY_STOP`

### 2.2 Execution Kill-Switch（別概念・固定語彙）
Execution Kill-Switch Level（crosscutと一致）：
- `ALLOW`
- `CLOSE_ONLY`
- `FLATTEN`
- `BLOCK`

固定マッピング（必須）：
- safety_mode=`EMERGENCY_STOP` → killswitch=`BLOCK`
- safety_mode=`SAFE` → killswitch=`BLOCK`（例外として policy が `CLOSE_ONLY` を許可しうるが、必ず audit）
- safety_mode=`NORMAL` → killswitch は ALLOW/CLOSE_ONLY/FLATTEN/BLOCK のいずれもあり得る（局所 hazard）

### 2.3 Execution Intent（固定）
実行リクエストは “Intent” として正規化し、最低限以下を持つ：
- `intent_id`（idempotency key / event_uid 相当）
- `intent_type`（PLACE/AMEND/CANCEL/CLOSE/FLATTEN）
- `venue` / `market` / `symbol`
- `side` / `qty` / `price`（必要なもののみ）
- `reduce_only`（bool）
- `time_in_force`（必要なら）
- `origin`（bot/controlplane/manual）
- `actor`（identity_access_spec の actor fields）
- `trace_id` / `run_id` / `schema_version`

---

## 3. Pre-trade Gate（必須ゲート）
### 3.1 ゲートは「拒否がデフォルト」（固定）
ゲートは “許可される条件が揃ったときだけ” 通す。
不明/欠損/観測不能は拒否。

### 3.2 ゲート入力（固定）
ゲートは最低限、以下を参照する：
- System Safety Mode（safety_state）
- Execution Kill-Switch level
- gate_results（Runtime gate）
- identity / authorization（actor + scope）
- config hierarchy（secret_ref only / env isolation）
- venue health（接続状態、時刻整合、429/署名失敗率など）
- position/exposure snapshot（取得不能なら安全側）

### 3.3 ゲート出力（固定）
ゲートは必ず以下の結論を返す：
- `ALLOW`（実行してよい）
- `REJECT`（拒否）
- `ALLOW_WITH_CONSTRAINTS`（制約付き許可：例 reduce-only 強制、上限強制）

そして必ず理由を構造化して返す：
- `reason_codes[]`（例：SAFETY_SAFE_MODE / KILLSWITCH_BLOCK / AUTHZ_DENY / VENUE_UNHEALTHY / INTEGRITY_UNKNOWN）
- `evidence_refs`（gate_results_ref, integrity_report_ref 等）

---

## 4. 実行の安全ルール（固定）
### 4.1 Safety Mode と実行（固定）
- safety_mode=`SAFE` のとき：
  - 原則：すべて拒否
  - 例外：policy が “緊急リスク低減” を許可している場合のみ `CLOSE_ONLY` を許容
  - 例外でも必ず `reduce_only=true` であること
- safety_mode=`EMERGENCY_STOP` のとき：
  - 例外なし。すべて拒否（BLOCK）

### 4.2 Kill-Switch と実行（固定）
- killswitch=`BLOCK`：すべて拒否
- killswitch=`FLATTEN`：flatten/close intent のみ（reduce-only）
- killswitch=`CLOSE_ONLY`：新規建玉増は禁止。reduce-only のみ許可
- killswitch=`ALLOW`：通常通り（ただしゲート通過必須）

### 4.3 Live誤爆防止（固定）
- live 実行は “暗黙に” 有効化されない
- 以下が揃わなければ place/amend は拒否：
  - env=prod + mode=live が明示
  - challenge/confirm（dangerous op）完了
  - safety_mode=NORMAL
  - runtime gate が許容
- cancel/close も “危険操作” に該当しうるため、ポリシーに応じて challenge/confirm を適用可能（ただし固定仕様は「適用できる」ことを要求）

---

## 5. Idempotency / Dedup（固定）
### 5.1 intent_id が核（固定）
同じ `intent_id` は、何度送っても「同じ効果」になること。
- 同一 `intent_id` の place は二重発注しない
- 同一 `intent_id` の cancel は二重キャンセルで崩れない
- 同一 `intent_id` の amend は順序と整合を保つ

### 5.2 外部注文IDとの対応（固定）
- venue が `client_order_id` を受け付ける場合：
  - `intent_id` を安定的に埋め込む（衝突しない表現）
- venue が受け付けない場合：
  - 内部で `intent_id -> venue_order_id` の対応表を持つ（永続/再起動耐性の設計は storage domain に委譲するが、存在は必須）

---

## 6. Retry / Backoff（固定）
- Transient / RateLimited は再試行可能だが、二重効果を起こさない設計が前提
- 429 は必ず backoff + jitter（値はPolicy）
- retry storm を防ぐ（失敗が一定以上なら killswitch を tighten できること）

---

## 7. Execution Connector との接続（固定）
- 取引所方言は UCEL execution connector spec を正本とする：
  - `docs/specs/ucel/execution_connector_spec.md`
- 本仕様は、コネクタ呼び出し前後で：
  - **gate → intent → connector → result → audit**
  の並びを固定する

---

## 8. Audit（固定）
### 8.1 必須監査イベント（固定）
最低限、以下は必ず `audit_event` を出す（秘密値なし）：
- `execution.intent.created`：intent生成
- `execution.gate.decision`：ALLOW/REJECT/CONSTRAINTS + reason + evidence refs
- `execution.sent`：コネクタ送信（リクエスト要約のみ）
- `execution.result`：成功/失敗/部分成功（venue order id 等）
- `execution.rejected`：拒否（理由）
- `execution.killswitch.set`：Kill-Switch変更
- `safety.transition`：Safety Mode変化（crosscut）

### 8.2 audit不能＝安全でない（固定）
監査イベントが出せない場合：
- place/amend 等の実行は拒否（SAFEへ寄せる判断も可）
- 例外を作るなら break-glass（ただし本仕様では “例外を常用しない” を固定）

---

## 9. Observability（固定：要求のみ）
実行系は以下を観測できる（方式は自由）：
- gate decision counts（ALLOW/REJECT/CONSTRAINTS）
- killswitch level time series
- venue error rates（429/auth/protocol）
- end-to-end latency（intent→sent→result）
- idempotency collisions / duplicates prevented
- SAFE/EMERGENCY_STOP への遷移回数と原因

---

## 10. 失敗モード（固定）と縮退
- position snapshot 取得不能 → CLOSE_ONLY or BLOCK（Policy）だが、少なくとも “新規建玉増” は禁止
- venue auth failures 急増 → killswitch tighten（CLOSE_ONLY/BLOCK）
- integrity unknown（データ欠損/監視欠損） → SAFE（crosscut）
- cancel storm → rate limit and throttle（Policy） + audit

---

## 11. テスト/検証観点（DoD）
最低限これが検証できること：

1) SAFE/EMERGENCY_STOP で place が確実に拒否される
2) CLOSE_ONLY で exposure 増の注文が確実に拒否される
3) FLATTEN で flatten/close 以外が拒否される
4) 同じ intent_id の place が二重発注されない
5) 429 再試行でも二重効果が起きない
6) 監査イベントが intent→gate→sent→result で揃う
7) live 誤爆（暗黙live）が起きない（challenge必須）

---

## 12. Policy/Runbook へ逃がす点（明確な分離）
- 許容する緊急例外（SAFEでCLOSE_ONLYを許す条件）
- 価格乖離、最大注文サイズ、最大露出、バックオフ値
- “どの操作を dangerous op にするか” の具体設定
→ すべて `docs/policy/**` に置く（意味は変えない）

---
End of document
