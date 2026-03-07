# domestic_public_ext_ws_schema_policy_v1

Schema policy for vendor public WS extension:
- breaking frame-shape change => major bump
- additive field change => minor bump
- typo/non-breaking fix => patch bump

`VendorPublicWsSchemaVersion` is mandatory for every operation.
`VendorPublicWsPayloadType` must match emitted payload shape.
`VendorPublicWsMetadata` must include venue/operation/source_channel/inventory_public_id.
