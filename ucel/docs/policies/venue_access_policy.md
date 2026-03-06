# UCEL Venue Access Policy (JP Resident SSOT)

## Scope
このポリシーは、JP resident 前提で UCEL が有効化を許可する venue access scope を定義する。
本ポリシーは実世界法規の宣言ではなく、repo 内証拠に基づく実装可否の SSOT である。

## Policy ID
- `jp-resident-v1`

## Defaults
- `default_scope`: `public_only`
- entry 未指定 venue は `public_only`

## Explicit entries
- `bitbank`: `public_private`（domestic policy）
- `bitflyer`: `public_private`（domestic policy）
- `coincheck`: `public_private`（domestic policy）
- `gmocoin`: `public_private`（domestic policy）
- `sbivc`: `public_only`（documented exception）

## Enforcement
- private REST / private WS / execution surface は scope で fail-fast 判定する。
- `public_only` / `blocked` venue では private surface を送信前に拒否する。

## Source linkage
- Human-readable SSOT: `ucel/docs/policies/coverage_policy.md`
- Machine-readable SSOT: `ucel/coverage/coverage_v2/jurisdictions/jp_resident_access.json`
- Spec: `docs/specs/ucel/venue_access_policy_v1.md`
