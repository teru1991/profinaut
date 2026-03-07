# UCEL Domestic Public API Inventory v1

## Purpose
JP domestic exchange crates の public API（REST + WebSocket）を repo 内証拠のみで完全棚卸しし、後続実装タスク(009B〜009F)の SSOT とする。

## Domestic scope decision
以下のいずれかに証拠がある venue を domestic scope とする。
- `ucel/Cargo.toml` workspace members
- `docs/exchanges/<venue>/catalog.json`
- `ucel/coverage/<venue>.yaml`
- `ucel/coverage/coverage_v2/exchanges/<venue>.json`
- `ucel/crates/ucel-registry/src/hub/registry.rs` registration table

v1 固定 domestic venues:
- bitbank
- bitflyer
- coincheck
- gmocoin
- bittrade
- sbivc

## Inventory target
含める対象:
- public REST endpoint
- public WS channel
- public system/status/maintenance
- public symbols/market metadata/reference

除外対象:
- private REST/WS
- repo 証拠がない endpoint/channel
- 海外 venue

## Inventory record contract
`ucel/coverage_v2/domestic_public/jp_public_inventory.json` の `entries[]` は以下を必須とする。
- venue
- venue_family
- api_kind (`rest|ws`)
- public_id
- path_or_channel
- method_or_subscribe_kind
- auth (`public`)
- category
- canonical_surface
- surface_class
- current_repo_status
- evidence_files
- evidence_kinds
- notes

## Fail rules
Gate は以下で fail する。
1. catalog / docs / coverage / ws_rules に public evidence があるのに inventory に無い。
2. 同一 `venue + public_id` が重複。
3. `auth != public` が混入。
4. evidence_files が空、または参照ファイルが repo に存在しない。
5. 同一 entry が複数 class として扱われる（JSON の単一値制約 + test gate）。

## Status semantics
`current_repo_status` は `ucel/coverage/<venue>.yaml` の entry 状態から算出する。
- implemented: implemented=true かつ tested=true
- partial: implemented/tested の片方のみ true
- not_implemented: implemented=false かつ tested=false
- unknown_for_repo: coverage entry が存在しない
