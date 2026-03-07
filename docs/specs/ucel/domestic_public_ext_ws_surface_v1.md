# domestic_public_ext_ws_surface_v1

Task: UCEL-DOMESTIC-PUBLIC-EXT-WS-009E

Vendor public WS extension surface methods:
- `vendor_public_subscribe_typed`
- `vendor_public_reference_subscribe_typed`
- `vendor_public_status_subscribe_typed`

Typed envelope required fields:
- venue
- operation_id
- category
- schema_version
- payload_type
- normalized_metadata
- typed_payload
- source_channel
- readiness_mode
- integrity_mode
- resume_mode

Fail conditions:
- raw frame passthrough
- missing schema_version
- missing typed_payload
- missing readiness/integrity/resume mode
