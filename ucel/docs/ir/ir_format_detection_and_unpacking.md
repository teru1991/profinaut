# IR Format Detection and Unpacking
Detection uses content-type, magic bytes, extension/kind, and XML markers.
ZIP unpack policy enforces entry limits, byte limits, no path traversal, no nested archives by default.
ZIP fixtures are generated from zip_spec.json at runtime.
