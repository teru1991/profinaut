# IR JP Official Surface v1

This spec fixes JP official source implementation scope on canonical IR surface.

In-scope source families:
- `jp_statutory_disclosure` (`edinet_api_documents_v2`)
- `jp_timely_disclosure` (`jp_tdnet_timely_html`)

Required canonical operations:
- issuer lookup / resolve
- list_documents / fetch_document_detail
- list_artifacts / fetch_artifact
- source metadata + provenance retention
