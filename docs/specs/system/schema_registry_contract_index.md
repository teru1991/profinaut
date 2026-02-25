# Schema Registry & Contract Index v1.0（固定）
Document ID: SYS-SCHEMA-REGISTRY-CONTRACT-INDEX
Status: Canonical / Fixed Contract
Scope: docs/contracts（JSON Schema）を “一覧・責務・互換・検証” で辿れるようにする索引SSOT

## 0. 目的（Non-negotiable）
契約（schema）が増えるほど「どれが正か」「誰が作り誰が読むか」「互換は守れているか」が壊れやすい。
本書は、contracts を索引化し、設計・実装・運用を一気通貫にする。

---

## 1. 参照（正本）
- Contract SSOT: `docs/contracts/*.schema.json`（唯一の正）
- Contracts README: `docs/contracts/README.md`
- Versioning policy: `docs/specs/system/versioning_policy.md`
- Domains SSOT: `docs/specs/system/domains_ssot.md`
- Gate spec: `docs/specs/system/docs_gate_spec.md`

---

## 2. 契約（Contract）の固定ルール（要約）
- 契約は docs/contracts/ にしか置かない（唯一の正）
- 互換性は versioning_policy に従う
- schema_version（整数）で “破壊変更” を表現する（必要時）
- Policy/Plan/Runbook は契約を “再定義” しない（参照のみ）

---

## 3. Contract Index（v1.0時点の一覧）
このリポ内の contracts は “証拠（Evidence）” 系が中心。
（追加された場合は、この表に追記していく）

| schema file | primary purpose | belongs-to domain | produced by（概念） | consumed by（概念） | notes |
|---|---|---|---|---|---|
| docs/contracts/audit_event.schema.json | 監査イベント（append-only） | O / Y（横断） | 全コンポーネント | D / Y / O / S | secret-free |
| docs/contracts/gate_results.schema.json | Gate結果（CI/Runtime/Daily） | C / G / Y | CI / runtime gate | S / D / Y | UNKNOWNは合格ではない |
| docs/contracts/integrity_report.schema.json | データ整合性レポート | H / C / Y | data platform | S / D / Y | gap/dup/stale |
| docs/contracts/replay_pointers.schema.json | 再現範囲ポインタ | O / N / Y | replay system | N / Y / D | dataset_ref導線 |
| docs/contracts/safety_state.schema.json | Safety状態 | E（横断） | safety controller | I / L / S | SAFE/EMERGENCY等 |
| docs/contracts/startup_report.schema.json | 起動時レポート | A / C / Y | 各サービス | S / D / Y | capabilities含む |
| docs/contracts/support_bundle_manifest.schema.json | サポート提出物目録 | Y（横断） | support bundle builder | D / Y | secret-free |

---

## 4. 互換性（固定）
### 4.1 破壊変更（MAJOR相当）
- フィールド削除/リネーム
- enum の意味変更
- 制約の強化（許容していた入力を弾く）
→ 新 schema_version（または新schema）を導入する

### 4.2 後方互換（MINOR相当）
- optional field 追加
- 制約の安全な緩和
→ 既存 consumer が壊れない

---

## 5. 契約テスト（Contract Tests）の固定要件
### 5.1 最低限の検証（MUST）
- JSON としてパースできる（CI）
- schema が自己矛盾しない（CI）
- 代表的な出力が schema に合致する（runtime/CIどちらでも可）

### 5.2 推奨の検証（SHOULD）
- Roundtrip（生成→検証→再読込）が可能
- 互換性チェック（旧consumerの入力が受理される）

---

## 6. “Schema Registry” の範囲（注意：ここは contracts ではない）
contracts は “証拠/メタ” が中心だが、データ本体の canonical schema（H/I/K 等）は Core Spec に定義される。
- Market data canonical: docs/specs/market_data/**
- Execution canonical: docs/specs/execution/**
- Ledger/PnL canonical: docs/specs/accounting/**

本書は contracts の索引SSOTであり、canonical data schema の全文定義を置かない（SSOT分裂防止）。

---

## 7. DoD（索引が機能している条件）
1) schema追加時に “この表” が更新される
2) belongs-to domain が明記される
3) 互換性ルール（versioning_policy）へ導線がある
4) Gateで “契約が壊れた状態” を main に入れない

---
End of document
