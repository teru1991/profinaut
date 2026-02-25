# Platform Foundation Core Spec v1.0（固定仕様）
Platform Foundation（基盤・共通）

- Document ID: PF-CORE-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): A（Platform Foundation）
- Crosscut dependencies（固定）:
  - Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/*.schema.json`（audit_event / safety_state / startup_report / gate_results / integrity_report / replay_pointers / support_bundle_manifest）
- Non-goals（本書で固定しない）:
  - 閾値（ms/%/容量/回数/保持期間）→ `docs/policy/**`
  - 運用手順（復旧/移行/手動介入）→ `docs/runbooks/**`
  - 計画/ロードマップ → `docs/plans/**`

---

## 0. 目的と到達点（Non-negotiable）
本仕様は、全ドメインに共通する「基盤の不変条件」を固定し、以下を保証する：

1) **相関可能性（Traceability）**：どの事象も `trace_id/run_id/schema_version` 等で追跡可能  
2) **再現・証明（Replay/Audit）**：監査・再現に必要な最小情報が常に残る  
3) **安全（Safety-by-design）**：誤爆（live誤操作）、秘密漏洩、危険操作の暴走を設計で封じる  
4) **堅牢（Resilience）**：部分故障で全体崩壊しない。失敗は分類され、縮退（degrade）する  
5) **高速（Performance）**：共通処理は軽量で、ホットパスを阻害しない（重い処理は分離）  
6) **安定（Stability）**：仕様が揺れない。Policyで値は動かせるが意味は変えない

---

## 1. “基盤”の範囲（責務境界）
### 1.1 Platform Foundation が必ず定義するもの（固定）
- 共通ID体系（trace_id / run_id / schema_version / event_uid 等）
- 共通エラーモデル（Standard Error Model）
- 冪等性・重複排除の原則（idempotency / dedupe contract）
- 設定階層（base/env/tenant/bot/secret）と適用順序
- Feature Flags / Capabilities（機能宣言・縮退）
- 環境隔離（dev/stage/prod、paper/shadow/live）と誤爆防止
- 監査/再現/サポート提出物の最小要求（crosscut 参照）
- Redaction（禁止キー検知・赤塗り）の不変要件

### 1.2 Platform Foundation が“実装を固定しない”もの（可変）
- 具体的ストレージ（S3/MinIO/ローカル）・DB種類
- 具体的観測基盤（Prometheus/Loki/Grafana 等）
- 具体的配布方式（バイナリ/コンテナ/サービス管理）
→ ただし「境界とインタフェース」「証拠の形」は固定する

---

## 2. 共通ID体系（SSOT：全ドメイン必須）
### 2.1 必須ID（すべての監査対象イベントに付与できること）
- `trace_id`：1つの処理連鎖（外部入力→内部判断→出力）を相関するID
- `run_id`：プロセス起動単位の識別（再現・監査の起点）
- `schema_version`：契約の版（contracts SSOT の版）
- `event_uid`：同一イベント判定（重複排除・再現の核）
- `component`：サービス名/モジュール名（例：collector/execution/controlplane）
- `instance_id`：ホスト/コンテナ/プロセスの識別（省略可だが推奨）

### 2.2 生成・伝播ルール（固定）
- `run_id` は起動時に生成し、プロセス内で不変
- `trace_id` は外部入力起点で生成し、可能な限り下流へ伝播
- `schema_version` は「出力する契約」に必ず一致
- `event_uid` は「同じ外部事実を二重計上しない」ために determinism-friendly に設計する
  - 原則：外部IDがあればそれを使う
  - 無ければ：`(source, venue, stream, event_time, seq, payload_hash)` のような安定キーで構成
- これらは **ログ・メトリクス・監査イベント・サポートバンドル**に共通で現れること

---

## 3. Standard Error Model（標準エラーモデル：固定）
目的：障害を「分類→対応→再現」に繋げる。曖昧な例外は許容しない。

### 3.1 エラーの分類（最低限）
- `Transient`：一時的（再試行で回復しうる）
- `RateLimited`：429等（バックオフ・ジッタ必須）
- `Auth`：認証・署名・権限
- `InvalidRequest`：入力不正（再試行しても治らない）
- `Protocol`：外部仕様違反/互換性破綻
- `Integrity`：整合性破綻（データ欠損/重複/時刻異常等）
- `ResourceExhausted`：CPU/メモリ/ディスク/FD/帯域
- `Safety`：危険操作ブロック/安全状態移行
- `Unknown`：最終手段（ここに落ち続けるのは設計欠陥）

### 3.2 必須フィールド（監査・再現に必要）
各エラーは少なくとも以下を持つ（ログ/監査で出せる形で）：
- `kind`（上記分類）
- `code`（説明可能な一意ID：将来Yドメインで体系化）
- `message`（人間向け）
- `retryable`（bool）
- `context`（trace_id/run_id、対象venue、対象stream、リクエスト種別など）
- `source`（外部/内部、HTTP/WS/DB等）

### 3.3 エラー時の固定ルール
- **秘密を含む可能性があるデータは必ず赤塗り**（詳細は crosscut と contracts の赤塗り規約に従う）
- `Unknown` が一定以上発生したら、それ自体を危険信号として扱う（観測と安全の連動は crosscut）

---

## 4. 冪等性（Idempotency）と重複排除（Dedupe）
### 4.1 冪等性の固定原則
- 同じ入力（同じ `event_uid`）は、何度処理しても **同じ結果**になること
- 再試行は「副作用が二重に起きない」ように設計されること

### 4.2 Dedupeの固定原則
- “重複が起きうる” のは正常（WS再送/再接続/リプレイ）
- システムは **重複を許容して吸収する**（落とすのではなく抑制する）
- Dedupe は Raw層ではなく、少なくとも canonical 化の層で保証できること（設計はH/Gドメインに委譲）

---

## 5. 設定階層（Config Hierarchy：固定）
### 5.1 階層と優先順位（固定）
設定は論理的に以下を持つ（実装形式は自由）：

1) `base`（全環境共通の不変設定）
2) `env`（dev/stage/prod 等）
3) `tenant`（将来の分離単位。1人運用でも“論理区画”として存在）
4) `bot`（ボット単位の設定）
5) `secret`（秘密：参照（secret_ref）のみ。値の直書き禁止）

**適用順序（固定）**：base → env → tenant → bot → secret_ref解決

### 5.2 Secretの固定ルール
- Secretは「保存しない」「ログに出さない」「バンドルに入れない」
- 設定中の secret は **secret_ref**（参照）としてのみ登場
- 禁止キー検知（forbidden-key scan）は必須（crosscut / contracts / policyで運用）

---

## 6. Feature Flags / Capabilities（固定）
目的：実装差・未実装・障害時縮退を“宣言”して安全に運用する。

### 6.1 Capabilities（固定）
各コンポーネント（またはコネクタ/アダプタ）は、
- “できること/できないこと”
- “保証できること/できないこと”
を **機械可読に宣言**できなければならない。

必須概念：
- `capabilities_version`
- `component`
- `features`（例：marketdata.ws.depth, execution.place_order, replay.typeB）
- `constraints`（例：max_symbols, max_subscriptions）

### 6.2 Safetyとの固定連携
- capability が不明/欠損なら、安全側に倒す（SAFE/ブロック）
- feature flag で “危険な近道” を許可する場合、それは dangerous-op 扱い（監査・確認が必要）

---

## 7. 環境隔離（dev/stage/prod & paper/shadow/live：固定）
### 7.1 不変要件
- dev/stage/prod は **論理的に完全分離**される（設定・秘密・データ・実行権限）
- paper/shadow/live は **同一経路の切替**であり、誤爆防止が最優先

### 7.2 誤爆防止（固定）
以下のどれかが欠けた状態で live 実行に入ってはならない：
- 明示的な `mode=live` 設定（暗黙のlive禁止）
- dangerous-op challenge/confirm を通過（crosscut）
- safety_state が NORMAL（かつ policy gate が許可）

---

## 8. Crosscutとの関係（固定）
Platform Foundation は crosscut を“上書きしない”。必ず参照し、全ドメインへ伝播させる。

- Safety state / dangerous ops / kill-switch は `docs/specs/crosscut/safety_interlock_spec.md`
- Audit & Replay は `docs/specs/crosscut/audit_replay_spec.md`
- Support bundle は `docs/specs/crosscut/support_bundle_spec.md`

---

## 9. テスト可能性（固定：検証観点）
本仕様が満たされているかを、各ドメインは最低限以下で検証できること：

- ID：監査イベントに trace_id/run_id/schema_version が常に存在
- Error：Unknownが継続発生しない（分類が実装されている）
- Config：secret直書きが検知される（禁止キー検知）
- Isolation：paper/shadow/live の誤爆防止が確認できる（challenge必須）
- Capability：未実装機能が「黙って成功」しない（宣言と拒否）

---

## 10. 次のドメイン仕様への共通テンプレ（固定）
以降の各ドメイン（B〜Y）の Core Spec は最低限この章立てを踏襲する：

1) 目的と到達点（Non-negotiable）
2) 責務境界（in / out）
3) 固定アーキテクチャ（概念層）
4) データ/契約（schema_version, id, event）
5) 安全（crosscut参照・境界）
6) 観測（metrics/logs/trace）
7) 失敗モード（標準エラーモデルへのマップ）
8) テスト/検証観点
9) Policy/Runbook/Planへの分離点

---
End of document