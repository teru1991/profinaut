pub fn normalize_text(raw: &str) -> String { raw.replace("\r\n", "\n").trim().to_string() }
