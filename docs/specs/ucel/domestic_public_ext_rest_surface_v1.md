# UCEL Domestic Public Extension REST Surface v1

## Scope
`ucel/coverage_v2/domestic_public/jp_public_inventory.json` のうち:
- `api_kind = rest`
- `surface_class = vendor_public_extension`

## Extension surface
SDK/Registry は以下の typed extension surface を提供する。
- `vendor_public_call_typed`
- `vendor_public_reference_typed`
- `vendor_public_status_typed`

## Typed envelope contract
`VendorPublicRestTypedEnvelope` は以下を必須とする。
- `category`
- `schema_version`
- `payload_type`
- `metadata`
- `typed_payload`

`metadata` 必須:
- `venue`
- `operation_id`
- `source_endpoint`
- `inventory_public_id`

## Fail rules
- raw passthrough (`raw_payload` 相当) を返してはならない。
- schema_version 無しは禁止。
- inventory 上の vendor extension REST entry が route/spec/docs に無い場合は fail。
- canonical 化対象を extension へ逃がしてはならない（inventory SSOT 準拠）。
