# IR Fetch Contract v1

Canonical fetch trait is `IrSourceAdapter`.

Required methods:
- `source_descriptor`
- `discover_issuers`
- `resolve_issuer`
- `list_documents`
- `fetch_document_detail`
- `list_artifacts`
- `fetch_artifact`

Fetch mode enum: `IrFetchMode::{Api,Feed,Html,Attachment}`.

The same contract must represent API/feed/HTML/attachment retrieval with metadata.
Raw passthrough is not the final surface.
