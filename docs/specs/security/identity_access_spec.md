# Security / Identity / Access Core Spec v1.0（固定仕様）
Secrets・Identity・Access（秘密管理 / 認証認可 / 接続境界）

- Document ID: SEC-IDENTITY-ACCESS-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): B（Security / Identity / Access）
- Depends-on（Crosscut / Fixed）:
  - Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/startup_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/support_bundle_manifest.schema.json`
- Policy separation（本書で固定しない）:
  - TTL / rotation interval / retry/backoff / threshold → `docs/policy/**`
  - 手順（失効・漏洩対応・復旧） → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、システム全体に対して **秘密漏洩ゼロ、誤爆ゼロ、最小権限、監査可能、再現可能** を固定保証する。

必達要件（固定）：
1) **Secrets are never persisted in plaintext**（平文保存禁止）
2) **Secrets are never logged**（ログ出力禁止）
3) **Secrets are never included in Support Bundles**（サポートバンドル混入禁止）
4) **All privileged actions are auditable**（危険操作・権限操作は監査イベント必須）
5) **Identity is explicit**（誰が/何が行ったかを明示できる）
6) **Access is least-privileged and revocable**（最小権限・即時無効化可能）
7) **Environment isolation is hard**（dev/stage/prod、paper/shadow/liveは論理的に完全分離）
8) **Failure is safe**（秘密取得不能・検証不能は SAFE / ブロックへ）

---

## 1. 用語（本書での定義）
- Secret：APIキー、トークン、署名鍵、Cookie、認証ヘッダ、Private Key、DBパスワード等
- secret_ref：秘密の「参照」。値そのものではない（例：`vault:kv/exchange/gmo#api_key`）
- Identity：行為主体（human operator / service account / job）
- AuthN：認証（本人/主体の確認）
- AuthZ：認可（許可範囲）
- Break-glass：緊急時の特権経路（ただし監査・期限・最小範囲が固定必須）
- Boundary：権限が影響する境界（実行、設定適用、データ破壊、外部公開、鍵操作など）

---

## 2. 安全モデル（固定）
### 2.1 原則（固定）
- **Deny by default**：明示的許可が無い限り拒否
- **Least privilege**：必要最小の権限のみ付与
- **Short-lived credentials**：長期トークンを避け、短命トークン＋自動更新を基本とする
- **Separation of duties**：危険操作は単一主体で完結しない（challenge/confirm）
- **Compartmentalization**：取引所・bot・環境ごとに秘密と権限を分割
- **Revocability**：漏洩・疑義があれば即時無効化できる構造
- **Observability honesty**：監査/検知が死んでいるなら SAFE 側へ倒す

### 2.2 Crosscut連携（固定）
- 安全状態は `safety_state` 契約の 3 mode（NORMAL/SAFE/EMERGENCY_STOP）に従う
- 危険操作は `docs/specs/crosscut/safety_interlock_spec.md` の challenge/confirm を必須とする
- 監査は `docs/specs/crosscut/audit_replay_spec.md` の規約に従う
- サポート提出物は `docs/specs/crosscut/support_bundle_spec.md` に従う

---

## 3. Secrets（秘密）管理：固定仕様
### 3.1 保存と参照（固定）
- 秘密は **secret_ref でのみ扱う**（設定ファイル・コード・CIに平文を置かない）
- `.env` は「開発者ローカルの便宜」用途に限定し、プロダクションでは使用しない（Policyで例外を作っても “平文禁止” は破れない）
- リポジトリ内に秘密ファイルを作らない（例外：**公開鍵**や**テスト用ダミー**）
- secret_ref の解決（resolve）は **起動時/必要時に行い、メモリ保持は最小**（TTL後破棄）

### 3.2 禁止キー検知（固定）
以下は全経路で禁止：
- `api_key`, `secret`, `token`, `private_key`, `authorization`, `cookie`, `passphrase` など（実際のリストは Policy で拡張可能）
固定ルール：
- 設定読み込み時、ログ出力直前、サポートバンドル生成前に **forbidden-key scan** を実行
- 検知した場合：
  - 実行を拒否（または SAFE へ遷移）
  - `audit_event` を必ず出す（秘密値は絶対に含めない）

### 3.3 ローテーション（固定）
- 秘密はローテーション可能であること（仕組みの実装は自由）
- ローテーションイベントは監査対象：
  - `audit_event` に「どの secret_ref が、どの理由で、どの範囲で」更新/失効したかを記録
- ローテーション失敗は **安全側（SAFE / BLOCK）**へ倒す

### 3.4 取引所APIキー（固定）
- 取引所APIキーは用途別に分割できる前提（read-only / trade / withdraw等）
- 原則：
  - withdraw 相当権限はデフォルト無効（必要なら Break-glass 扱い）
  - trade 権限も bot/strategy 単位で分割できる
- いかなる場合も：
  - キーの平文をログ/監査/バンドルに入れない
  - 署名素材（secret）をそのまま永続化しない

---

## 4. Identity（主体）管理：固定仕様
### 4.1 主体の種類（固定）
- Human Operator（人間）
- Service Identity（サービス/ジョブ）
- External System（外部連携、ただし境界は必ず限定）

### 4.2 主体の識別（固定）
すべての監査イベント（audit_event）に、可能な範囲で以下を付与できること：
- actor_type（human/service/external）
- actor_id（ユーザID/サービスID）
- session_id（短命セッション識別子）
- run_id / trace_id（platform foundation の相関）

### 4.3 セッション（固定）
- セッションは短命であること（TTLはPolicy）
- 無効化できること（強制失効）
- セッション情報が欠落した特権操作は拒否（SAFE）

---

## 5. Access Control（認可）：固定仕様
### 5.1 権限モデル（固定）
権限は最低限以下の軸で分離される：
- 環境（dev/stage/prod）
- 実行モード（paper/shadow/live）
- ドメイン（collector/storage/execution/controlplane/observability）
- 対象（venue/bot/strategy/tenant）
- 操作種別（read/write/execute/admin/break-glass）

### 5.2 危険操作（固定）
危険操作は以下の条件を満たさない限り実行不可：
- System Safety Mode が許容（SAFE/EMERGENCY_STOP では原則拒否）
- dangerous-op challenge/confirm が完了
- 監査イベント（audit_event）が出せる（監査基盤が死んでいるなら SAFE）

危険操作の例（固定分類）：
- live 実行の開始/再開
- キルスイッチ解除
- gate を無効化/緩和
- 監視を無効化
- 秘密のローテーション/失効
- 破壊的データ操作（削除/リストア/強制コンパクション）

### 5.3 Break-glass（固定）
- Break-glass は **最終手段**であり常用禁止
- 必須：
  - 明示的理由
  - 時限（expiry）
  - 最小範囲（scope）
  - 監査イベント（開始/終了/結果）
- Break-glass でも秘密値そのものは露出しない

---

## 6. Environment Isolation（固定）
### 6.1 分離（固定）
- dev/stage/prod は秘密・設定・データ・権限が交差しない
- paper/shadow/live は誤爆防止のため、切替に強い制約を課す

### 6.2 Live誤爆防止（固定）
liveは暗黙に有効化されない。最低でも以下が必要：
- 明示 `mode=live`
- dangerous-op challenge/confirm
- Safety Mode が NORMAL
- gate_results が許容（Policy）

---

## 7. Network / Endpoint Boundary（固定）
### 7.1 公開境界（固定）
- 管理系エンドポイントは公開しない（公開するなら認証＋許可＋監査＋レート制限）
- 監視/運用アクセスも最小公開（ゼロトラスト前提）

### 7.2 内部通信（固定）
- 内部サービス間の通信は認証可能であること（方式は自由：mTLS/署名トークン等）
- 認証不能な内部通信は SAFE 側へ（少なくとも特権操作は拒否）

---

## 8. Audit / Evidence（固定）
### 8.1 必須監査イベント（固定）
最低限、以下は audit_event を必ず発行：
- run.start / run.end（startup_report参照）
- safety.transition（safety_state参照）
- dangerous_op.challenge / confirm / reject
- execution.killswitch.set
- secret.guard.triggered（禁止キー検知）
- secret.rotate / secret.revoke（秘密操作）
- access.denied（重要拒否：live/特権/破壊）

### 8.2 監査の秘密非含有（固定）
- audit_event.details に秘密値を入れない
- secret_ref は入れて良い（ただし “参照” のみ）

---

## 9. Support Bundle（固定）
- bundle は secret-free（crosscut準拠）
- 監査・安全・整合の証拠（startup_report/gate_results/integrity_report/safety_state）を含む
- 生成トリガはPolicyだが、生成時は必ず audit_event 参照を残す

---

## 10. 失敗モード（固定）と挙動
本ドメインの失敗は “安全側へ倒す” が大原則。

- secret resolve 失敗 → SAFE へ（少なくとも live/execute は BLOCK）
- forbidden-key 検知 → 実行拒否 + audit_event
- audit 出力不能 → SAFE へ（証明不能＝安全ではない）
- identity 不明 → 特権操作拒否
- 権限不足 → 明確に拒否（曖昧な成功はしない）

---

## 11. テスト/検証観点（DoD）
最低限、以下を満たすことを検証可能にする：

1) **Secret leak tests**
   - ログ/監査/バンドルに禁止キーが混入しない
2) **Forbidden-key scan tests**
   - ダミー秘密を設定に混ぜると確実に検知し拒否される
3) **Live misfire prevention**
   - mode=live が暗黙に有効化されない
   - challenge/confirm無しで live 実行が開始できない
4) **Least privilege**
   - 読み取り権限だけで実行系操作ができない
5) **Revocation**
   - セッション/秘密の失効後に操作ができない
6) **Audit completeness**
   - 危険操作・拒否・遷移が必ず audit_event を残す

---

## 12. Policy/Runbookへ逃がす点（明確な分離）
- TTL / rotation 間隔 / rate-limit backoff / allowed scopes / deny lists → Policy
- 漏洩時対応、失効手順、復旧フロー、権限発行フロー → Runbooks
- 導入順/移行計画 → Plans

---
End of document
