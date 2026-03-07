# IR Canonical Surface v1

This contract defines canonical source/issuer/document/artifact surface for IR.

- Source abstraction: `IrSourceDescriptor` with market/family/kind/access policy/access patterns.
- Issuer abstraction: `IrIssuerKey` + source-scoped aliases with provenance.
- Document abstraction: `IrDocumentDescriptor` keyed by source-document identity.
- Artifact abstraction: `IrArtifactDescriptor` keyed by document+artifact id.

Fail rules:
- source without access policy class is invalid.
- provenance-less issuer resolution is invalid.
- unknown artifact/document taxonomy is invalid.
