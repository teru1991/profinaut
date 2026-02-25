# Cross-Domain Dependency Matrix v1.0（固定）
Document ID: SYS-CROSS-DOMAIN-DEP-MATRIX
Status: Canonical / Fixed Map
Scope: A〜Y の責務境界と “実装依存（Depends-on）” を SSOT として固定する

## 0. 目的（Non-negotiable）
ドメインが増えると、設計の衝突点は「境界の曖昧さ」と「暗黙依存」に集約される。
本書は以下を固定する：
- A〜Y の “依存関係” を明文化し、設計の抜け漏れを防ぐ
- 「どこまでがそのドメインの責任か」を迷わせない
- 実装順序（Plan）を固定するものではない（Planは可変）

---

## 1. 用語（固定）
### 1.1 Depends-on（依存）の定義
- MUST dependency: それが無いと “安全に正しく” 実装できない依存
- SHOULD dependency: 無くても作れるが “品質（安全/運用/再現性）” が落ちる依存

### 1.2 境界の原則（固定）
- Core Spec は意味を固定し、運用値は Policy へ外出しする
- crosscut（Safety/Audit/Bundle）は上書きせず参照する
- contracts（JSON Schema）は `docs/contracts/` が唯一の正

---

## 2. 参照（正本）
- Domains SSOT: `docs/specs/system/domains_ssot.md`
- Platform Foundation（A）: `docs/specs/platform_foundation_spec.md`
- Environment/Mode: `docs/specs/system/environment_mode_matrix.md`
- Versioning: `docs/specs/system/versioning_policy.md`

---

## 3. 依存マップ（A〜Y：初期SSOT）
注記：
- ここにある依存は “代表的な最小集合”。
- 各ドメインの Core Spec は、本書と矛盾してはならない。
- 追加/削除は SemVer（意味追加＝MINOR、統合/削除＝MAJOR）。

### 3.1 A〜G（基盤・統治・観測）
| Domain | MUST Depends-on | SHOULD Depends-on | Inputs (概念) | Outputs / Evidence (概念) |
|---|---|---|---|---|
| A Platform Foundation | — | C（観測）, B（鍵）, G（契約） | system context | 共通ID/エラー/設定階層/Capabilities |
| B Secrets/Identity/Access | A | D, Y, U | secret refs / auth context | RBAC/強操作/鍵用途分離/ローテ方針 |
| C Observability/SRE | A | D, Y | logs/metrics/traces | SLO/SLI/alerts/UNKNOWN扱い |
| D Incident/Runbooks | A, C | B, E, Y | alerts / symptoms | runbooks / postmortem / recovery導線 |
| E Safety Controller | A, B, C | D, J, I | safety inputs / signals | SAFE_MODE / CLOSE_ONLY / FLATTEN / HALT |
| F Time/Clock Discipline | A | C | ntp/drift/exchange time | event_time/recv_time/persist_time 統一 |
| G Data Contracts/Schema | A | C, Y, U | schema proposals | contracts SSOT / Gate / compatibility |

### 3.2 H〜O（データ・実行・統合台帳・検証・再現）
| Domain | MUST Depends-on | SHOULD Depends-on | Inputs (概念) | Outputs / Evidence (概念) |
|---|---|---|---|---|
| H Market Data Platform | A, F, G, B, C | D, O, V | WS/REST feeds | canonical data / quality / lake layout |
| I Execution Platform | A, F, G, B, C, E | J, K, D, O | intents / venue APIs | orders/fills/balances / reconciliation |
| J Risk/Policy Gate | A, C, E | K, D, O | positions/pnl/exposure | pre/in/post-trade gate decisions |
| K Portfolio/Treasury | A, G, B, C | O, D | fills/balances/orders | single book / pnl / exposure / transfers |
| L Bot Control Plane | A, B, C, E | J, K, G, D, Y | bot registry / ops | deploy/start/pause/stop / audit trail |
| M Strategy Runtime/Plugin | A, L, C | H, I, J, K, O | market data / signals | intents / state persistence |
| N Experiment/Backtest/Forward | A, G, C | H, I, M, O, T, V | dataset/code/params | experiment ledger / comparisons |
| O Deterministic Replay/Audit | A, G, C | H, I, K, D, Y | event logs / pointers | replay pointers / append-only audit log |

### 3.3 P〜Y（AI/オンチェーン/株/ダッシュボード/品質/リリース/運用/製品化/診断）
| Domain | MUST Depends-on | SHOULD Depends-on | Inputs (概念) | Outputs / Evidence (概念) |
|---|---|---|---|---|
| P AI/ML Platform | A, N, O, C | J, M, T, V | datasets / runs | models / hpo / safe-RL constraints |
| Q On-chain Trading/Arb | A, B, F, G, C | H, I, J, K, O, D | RPC/DEX/chain events | trades/arbs / finality/reorg handling |
| R Equity/IR Analytics | A, G, C | H, D, Y | disclosures/IR | scores/screeners/alerts |
| S Dashboard Product | A, C | D, E, L, J, K, H, Y | status/evidence | layout/widgets/explain UI/notifications |
| T Testing/Simulation/Chaos | A, G, C | H, I, N, O, E | emulators/faults | golden E2E / chaos validation |
| U Release/SupplyChain/Hardening | A, B, G, C | D, Y | builds/SBOM | signed releases / scans / rollout safety |
| V FinOps/Storage Lifecycle | A, C | H, N, O, D | storage metrics | retention/rebuild/failover playbook |
| W Reporting/Accounting/Export | A, K, O, C | R, G, D | trades/pnl/events | tax/export/reporting packages |
| X Productization/Distribution | A, U, B, C | Y, D | release artifacts | stable/beta/rollback/license SSOT |
| Y Supportability/Diagnostics | A, C, O, G | D, B, U | support bundle inputs | support_bundle / error catalog / repro path |

---

## 4. 境界衝突が起きやすい “要注意ペア”（固定チェック）
- H（収集） ↔ G（契約）: “フォーマット決定” は G、 “取り込み運用” は H
- I（実行） ↔ J（リスク）: “最終可否” は J、 “唯一の発注出口” は I
- K（台帳） ↔ O（監査/再現）: “改竄不能の証跡” は O、 “残高/ポジの真実” は K
- L（運用） ↔ S（UI）: “操作と監査” は L、 “表示と説明” は S
- T（テスト） ↔ 全ドメイン: “最悪条件の再現” を T が担保、意味の固定は各 Core Spec

---

## 5. DoD（本マップが機能している条件）
1) 各ドメイン Core Spec の Depends-on が本書と矛盾しない
2) 依存理由が “暗黙” ではなく文章で説明されている
3) 新規ドメイン/統合が SemVer で管理される
4) Plan は本書を参照するが、本書が Plan を固定しない

---
End of document
