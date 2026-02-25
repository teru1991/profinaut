# Reporting / Dashboard Analytics / Explainability Core Spec v1.0（固定仕様）
Truthful reporting / Evidence-linked dashboards / Explainability / Access control

- Document ID: RPT-DASH-EXPLAIN-SPEC
- Status: Canonical / Fixed Contract
- Belongs-to (Domains): L（Reporting / Dashboard）
- Depends-on（Fixed）:
  - Platform Foundation: `docs/specs/platform_foundation_spec.md`
  - Identity/Access: `docs/specs/security/identity_access_spec.md`
  - Crosscut Safety: `docs/specs/crosscut/safety_interlock_spec.md`
  - Crosscut Audit/Replay: `docs/specs/crosscut/audit_replay_spec.md`
  - Crosscut Support Bundle: `docs/specs/crosscut/support_bundle_spec.md`
  - Observability: `docs/specs/observability/observability_slo_diagnostics_spec.md`
  - Storage/Integrity: `docs/specs/storage/storage_integrity_spec.md`
  - Risk/Portfolio: `docs/specs/risk/portfolio_risk_management_spec.md`
  - Accounting/PnL: `docs/specs/accounting/order_trade_ledger_pnl_spec.md`
  - Control Plane: `docs/specs/control_plane/control_plane_bot_manager_spec.md`
- Contracts SSOT（唯一の正）:
  - `docs/contracts/integrity_report.schema.json`
  - `docs/contracts/gate_results.schema.json`
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/replay_pointers.schema.json`
  - `docs/contracts/support_bundle_manifest.schema.json`
- Policy separation（固定しない）:
  - 表示の閾値/集計粒度/保持期間/キャッシュTTL → `docs/policy/**`
  - 運用手順/問い合わせ手順/監査提出フロー → `docs/runbooks/**`

---

## 0. 目的と到達点（Non-negotiable）
Reporting/Dashboard は “人間の意思決定” を支える最前線であり、誤表示は損失に直結する。
本仕様は、ダッシュボード/レポートが **真実を隠さず、証拠にリンクし、説明可能**であることを固定保証する。

必達要件（固定）：
1) **Truthfulness**：UNKNOWN/欠損/監視欠損を「健康」に見せない（Observability honesty）
2) **Evidence-linked**：重要数値は必ず根拠（integrity_report / audit_event / replay_pointers）へ辿れる
3) **Safe-by-design UI**：live操作や危険操作は “固定の安全導線” を外せない
4) **Access control**：見せる/操作できる範囲は最小権限で制御（Identity/Access準拠）
5) **No secret leakage**：表示・エクスポート・共有で秘密が漏れない
6) **Stable semantics**：同じ指標名が意味を変えない（Policyで値は変えても意味は固定）
7) **Performance isolation**：重い集計で本番を壊さない（キャッシュ/非同期/段階化）

---

## 1. 範囲（in / out）
### 1.1 In
- ダッシュボードの “真実性ルール”（UNKNOWN表現、欠損表現）
- 指標の出典リンク（evidence refs）
- レポート生成の再現性（replay pointers）
- 表示/操作の認可
- エクスポート（CSV/JSON/PDF等）の安全要件（秘密・PII）
- Control Plane 操作の安全導線（dangerous op）

### 1.2 Out
- 具体UIフレームワーク選定
- 具体の画面デザイン（ただし安全導線は固定要求）
- BI専用の高度可視化（将来拡張）

---

## 2. Truthfulness（固定：嘘をつかない表示）
### 2.1 UNKNOWN の固定扱い
以下は “良い状態” として表示してはいけない：
- 監視欠損（targets down）
- integrity_report UNKNOWN/FAIL
- gate_results UNKNOWN/FAIL
- quality UNKNOWN（価格/ポジション/データ入力）

固定ルール：
- 画面上で明示（例：UNKNOWNバッジ、灰色、注意）
- 重要な数値（PnL/Exposure/Fill率等）には quality を添付して表示できること

### 2.2 欠損の固定扱い
- 欠損は “0” や “変化なし” に埋めて表示しない
- 欠損は “missing intervals / gaps” として明示し、根拠へリンク可能

---

## 3. Evidence-linked Metrics（固定：証拠参照）
### 3.1 重要指標は evidence を持つ（固定）
最低限、以下のカテゴリは evidence refs を提示できる：
- Safety mode / transitions（safety_state + audit）
- Kill-Switch level（audit + control plane）
- Gate results（gate_results_ref）
- Integrity report（integrity_report_ref）
- Collector quarantine（audit + integrity）
- Storage backlog/backpressure（audit + integrity）
- Execution outcomes（audit_event ids）
- Risk snapshot / limit breaches（audit + risk snapshot refs）
- PnL report（replay_pointers_ref + ledger window）

### 3.2 Evidence参照形式（固定）
UI/レポートは少なくとも以下を提供できる：
- `audit_event` の参照（id もしくは時刻+検索キー）
- `integrity_report` の参照（window+ref）
- `gate_results` の参照（window+ref）
- `replay_pointers` の参照（dataset/ledger window ref）
- `support_bundle_manifest` の参照（生成時）

---

## 4. Dashboard Safety Coupling（固定）
### 4.1 Safety 状態の常時可視（固定）
ダッシュボードは常に以下を見える形で表示できる：
- System Safety Mode（NORMAL/SAFE/EMERGENCY_STOP）
- Execution Kill-Switch（ALLOW/CLOSE_ONLY/FLATTEN/BLOCK）
- Observability health（OK/UNKNOWN）

固定ルール：
- SAFE/EMERGENCY_STOP を “目立たない” 表示にしない
- Unknown を “緑” にしない

### 4.2 操作の制約（固定）
- SAFE/EMERGENCY_STOP のとき、live系操作はUIから実行不可（APIも拒否）
- “Kill-Switch緩和” は dangerous op（challenge/confirm必須）
- “Kill-Switch強化” は即時可能（権限があれば）

---

## 5. Reporting / Export（固定）
### 5.1 再現可能なレポート（固定）
定期/オンデマンドのレポート生成は、必ず replay pointers を生成し、以下を固定できる：
- 対象期間（window）
- 入力範囲参照（ledger window / dataset_ref）
- policy snapshot ref（丸め/手数料/評価通貨等）
- binary_hash / config_hash（可能な範囲）

### 5.2 エクスポート安全（固定）
- エクスポートは secret-free（禁止キー検知）
- 個人情報/機微情報が入る可能性がある場合は、赤塗り/マスキングの仕組みがある
- エクスポート操作は監査対象（audit_event）

---

## 6. Access Control（固定）
### 6.1 認可（固定）
- 表示・操作は actor の権限で制限される（Identity/Access準拠）
- 例：
  - view-only：閲覧のみ
  - operator：停止/診断など限定操作
  - admin：dangerous op challenge 発行可能
  - break-glass：緊急時のみ（監査+期限+範囲）

### 6.2 UI最小権限（固定）
- 権限がない操作はUI上でも表示/実行できない（“押したら403” を常態化させない）
- dangerous op の confirm は actor と reason が必須

---

## 7. Explainability（固定：説明可能性）
### 7.1 “なぜそうなった” を固定で説明できる
以下は少なくとも説明可能（説明の根拠リンク付き）：
- なぜ bot が停止/隔離されたか（quarantine・gate・safety）
- なぜ注文が拒否されたか（execution.gate.decision reason_codes）
- なぜPnLが変化したか（ledger entries / restatement / price input quality）
- なぜ露出が上がったか（positions/open orders/risk snapshot）

### 7.2 最低限の説明フォーマット（固定）
- “結論” + “理由コード” + “証拠リンク” + “推奨次アクション（runbook link）”

---

## 8. Performance Isolation（固定）
### 8.1 重い集計は本番を壊さない（固定要求）
- UI表示のための集計は、collector/execution のホットパスを阻害しない
- 大きい集計は非同期（バッチ/キャッシュ）で提供できる
- キャッシュの古さは明示される（freshness表示）

### 8.2 Degraded表示（固定）
- バックエンド遅延/DB負荷で “最新が出せない” 場合、古い値を “最新” として見せない
- stale/unknown を明示

---

## 9. Audit（固定）
最低限、以下を audit_event に残す：
- `report.generate.start/end`（window + replay_pointers_ref）
- `report.exported`（形式/対象/actor）
- `dashboard.dangerous_op.challenge/confirm/reject`（control plane連携）
- `dashboard.access.denied`（重要拒否）
- `dashboard.view.critical`（P0画面の閲覧ログ：必要ならPolicyで有効化）

秘密値は含めない。

---

## 10. テスト/検証観点（DoD）
最低限これが検証できること：

1) UNKNOWN/欠損が “健康” と表示されない
2) 主要指標が evidence refs に辿れる
3) SAFE/EMERGENCY_STOP で live操作がUI/APIとも拒否される
4) エクスポートが secret-free（禁止キー検知）
5) レポート生成で replay_pointers が生成される
6) 権限無しで危険操作が見えない/できない
7) stale 表示が “最新” と誤認されない

---

## 11. Policy/Runbookへ逃がす点
- 表示閾値、集計粒度、キャッシュTTL、通知・エクスポート制限
- 問い合わせ・監査提出フロー
→ Policy/Runbookへ（意味は変えない）

---
End of document
