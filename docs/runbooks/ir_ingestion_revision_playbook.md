# IR Ingestion / Revision Playbook v1.0

## 目的
IR/開示は訂正・差替えがある前提で、provenanceと版管理を壊さずに取り込む。
取得失敗や重複、差替え検知を安全に処理する。

## 参照（SSOT）
- 固定仕様: `docs/specs/ucel/ir_connector_spec.md`
- 契約:
  - `docs/contracts/audit_event.schema.json`
  - `docs/contracts/replay_pointers.schema.json`（任意）

---

## 0. 原則
- 最新だけ保持禁止（履歴を消さない）
- hash変化はrevision（差替え）として扱う
- provenance（source_ref/source_type/content_ref）を必ず残す

---

## 1. 入口：事象タイプ
- [ ] fetch_fail（取得失敗継続）
- [ ] parse_unknown（形状不明）
- [ ] duplicate（重複）
- [ ] revision_detected（差替え/訂正）

---

## 2. 取得失敗（fetch_fail）
対応：
- [ ] エラーカテゴリ分類（Network/RateLimit/Protocol）
- [ ] 再試行（policy上限）
- [ ] 継続するなら監査イベント＋通知（ノイズ抑制はPolicy）

---

## 3. parse_unknown（形状不明）
対応：
- [ ] raw/docを保存参照（content_ref）
- [ ] 推測で抽出しない（品質を落とさない）
- [ ] 実装更新（adapter改善）で再取り込み可能にする

---

## 4. 重複（duplicate）
対応：
- [ ] hash + source_ref で同一判定
- [ ] event_uidは決定的に同一へ
- [ ] 重複として監査（必要なら）

---

## 5. revision_detected（差替え/訂正）
対応：
- [ ] document_versionを更新
- [ ] supersedesで前版参照
- [ ] revisionイベントを発火（kind=revision）
- [ ] 重要ならアラート（Policy）

---

## 6. 証明
- [ ] provenanceが追える（source_ref/content_ref）
- [ ] revision履歴が残っている（supersedes）
- [ ] 必要ならreplay pointersを生成
