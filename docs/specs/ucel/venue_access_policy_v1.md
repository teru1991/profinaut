# UCEL Venue Access Policy v1 (JP Resident)

## Purpose
本仕様は、UCEL における venue access 可否（public/private）を**機械可読 SSOT**として定義する。
ここでいう可否は法的真偽ではなく、**このリポジトリが有効化を許す範囲**を示す。

## Residency
- 正式サポート residency class: `jp_resident`

## Access scope
- `blocked`: 全 surface を拒否
- `public_only`: public REST / public WS のみ許可
- `public_private`: public/private REST/WS と execution を許可

## Access surface
- `public_rest`
- `public_ws`
- `private_rest`
- `private_ws`
- `execution`

## Default and fail-closed rules
- policy entry が無い venue は `default_scope` を適用する。
- v1 の `default_scope` は `public_only`。
- 判定が曖昧な場合は private とみなし拒否（fail closed）。

## JP resident v1 baseline
- `public_private`:
  - bitbank
  - bitflyer
  - coincheck
  - gmocoin
- `public_only` (documented exception):
  - sbivc
- 上記以外は `default_scope=public_only`

## Enforcement points
- Hub REST/WS の resolve 後、送信前に policy gate を適用する。
- Invoker REST/WS の resolve 後、送信前に policy gate を適用する。
- Capabilities は venue access scope を明示し、上位が private を誤有効化できないこと。

## SSOT files
- Human policy: `ucel/docs/policies/venue_access_policy.md`
- Machine policy: `ucel/coverage/coverage_v2/jurisdictions/jp_resident_access.json`
