# UCEL Domestic Public Extension REST Schema Policy v1

## Versioning
各 operation は `major.minor.patch` の schema version を持つ。
- breaking: major up
- additive: minor up
- typo/fix: patch up

## Category enum
- `vendor_public_status`
- `vendor_public_reference`
- `vendor_public_network`
- `vendor_public_instrument_rule`
- `vendor_public_funding_like`
- `vendor_public_misc`

## Payload type enum
- `object`
- `array`
- `enum_like_object`
- `time_series`

## Compatibility baseline (009D)
- 現在の vendor public REST extension operation は全件 `1.0.0`。
- unknown operation_id は fail-fast。
- additive field は typed builder で許容し、breaking mismatch は schema update 前提で fail 可能にする。
