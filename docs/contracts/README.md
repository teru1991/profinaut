# docs/contracts/ (Contract SSOT)
ここは **機械可読な契約（JSON Schema）の唯一の正（SSOT）** です。

## ルール
- `*.schema.json` 以外に「契約」を置かない
- 実装（Rust/Python/TS等）はここに追随する
- 仕様書（Core Spec）は `docs/specs/` で意味を定義する（契約の背景/責務境界）

## 収録される契約
- audit_event
- gate_results
- integrity_report
- replay_pointers
- safety_state
- startup_report
- support_bundle_manifest

## 互換性
互換性ルールは `docs/specs/system/versioning_policy.md` に従う。
