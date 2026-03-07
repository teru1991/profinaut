# ir_content_normalization_v1
Defines canonical normalized content output with deterministic pipeline: detect -> normalize -> assemble.
Fields: document_key, artifact_key, normalization_schema_version, normalized_format, normalized_text, sections, tables, extracted_attachments, language_hints, charset, provenance, support_level.
Fail-fast: unknown format, invalid charset, malformed content, invalid archive.
