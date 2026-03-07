# IR Identity Contract v1

Canonical identity contract:
- `IrIssuerResolver`
- `IrIssuerResolutionInput`
- `IrIssuerResolutionResult`
- `IrIssuerIdentityProvenance`
- `IrIssuerConfidence`

Rules:
- market-scoped canonical key and source-scoped aliases are distinct.
- ticker/cik/edinet-like/site-slug/url-like values are alias forms.
- provenance is mandatory; missing provenance must fail.
