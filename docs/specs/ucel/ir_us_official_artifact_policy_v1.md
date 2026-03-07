# IR US Official Artifact Policy v1

Canonical supported artifact kinds:
- html, pdf, xbrl, ixbrl, xml, txt, zip, json

Success conditions:
- artifact kind resolved
- content_type provided
- size_bytes provided
- checksum provided
- source_url retained in metadata

Fail-fast:
- oversized artifact
- invalid content type
- unsupported artifact kind
