# IR Artifact Contract v1

Document contract:
- `IrDocumentListRequest/Response`
- `IrDocumentDetailRequest/Response`

Artifact contract:
- `IrArtifactListRequest/Response`
- `IrArtifactFetchRequest/Response`

Artifact fetch response may contain:
- bytes (binary)
- text candidate
- metadata

Responsibility split is mandatory; source raw IDs remain in provenance metadata.
