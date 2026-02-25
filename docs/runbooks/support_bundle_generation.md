# Support Bundle Generation Runbook v1.0

## 目的
Support Bundle（診断パッケージ）を、秘密非漏洩（fail-closed）で生成し、
障害解析・問い合わせ・自己復旧を最短化する。

## 参照（SSOT）
- 固定仕様: `docs/specs/crosscut/support_bundle_spec.md`
- 契約:
  - `docs/contracts/support_bundle_manifest.schema.json`
  - `docs/contracts/safety_state.schema.json`
  - `docs/contracts/audit_event.schema.json`
- Policy:
  - `docs/policy/forbidden_keys.toml`
  - `docs/policy/retention.toml`

---

## 0. 原則
- **秘密を絶対に入れない**
- 検知したら **fail-closed**（生成中断＋監査＋必要ならHALT）
- bundleは "提出物" になり得るため、構造は契約SSOTに従う

---

## 1. 生成のトリガ
- 手動：運用者が生成
- 自動：重大イベント（HALT、外部混入、WAL復旧、長時間観測欠損 等）

---

## 2. 最小構成（必須）
- manifest.json
- safety_state.json
- audit_tail.ndjson
- logs_tail（赤塗り済み）
- gate_results（ある場合）
- startup_report（ある場合）
- integrity_report（ある場合）

※何を入れたかの正は `manifest.json`。

---

## 3. 赤塗り（fail-closed）
1) 生成前スキャン  
   - forbidden_keys/regex に一致しないか
2) 生成後スキャン  
   - 出力ファイル全体に対して再スキャン
3) 検知したら：
   - bundle生成を中断（部分ファイルは破棄推奨）
   - AuditEvent：`SECRET_LEAK_GUARD_TRIGGERED` を発火
   - 実弾系がある場合は `HALT` を推奨（運用判断）

---

## 4. 典型的な提出
- 問い合わせ・自己解析の際は、bundleを添付し
  - いつ（期間）
  - どのスコープ（account/venue/bot）
  - 何が起きたか（SafetyStateのtrigger）
  を合わせて提出する

---

## 5. 保持・削除
- 保持数/サイズ上限は Policy（retention.toml）
- 削除する場合は（将来）削除操作自体を監査イベントとして残す設計を推奨
