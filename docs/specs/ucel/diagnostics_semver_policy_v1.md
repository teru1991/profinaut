# UCEL Diagnostics SemVer Policy v1

- **major**: backward-incompatible manifest/analyzer change (old bundle unreadable)
- **minor**: backward-compatible field additions/check enhancements
- **patch**: output formatting/fixes with compatibility preserved

Analyzer compatibility rule:
- old bundle -> new analyzer must work inside supported major range.
- unsupported major mismatch must fail explicitly.
