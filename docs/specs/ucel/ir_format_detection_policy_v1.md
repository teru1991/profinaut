# ir_format_detection_policy_v1
Detection precedence: magic bytes, content-type, extension/kind, container sniffing.
Unknown format MUST error and never silently fallback to plain text.
