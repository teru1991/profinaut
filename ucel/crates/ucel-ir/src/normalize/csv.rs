pub fn normalize_csv(raw: &str) -> String { raw.replace("\r\n", "\n").trim().to_string() }
