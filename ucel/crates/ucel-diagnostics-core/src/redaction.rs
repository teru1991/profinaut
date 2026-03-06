pub fn deny_patterns() -> Vec<&'static str> {
    vec![
        "AKIA",
        "BEGIN PRIVATE KEY",
        "Authorization: Bearer ",
        "X-Api-Key",
        "SECRET",
        "TOKEN",
        "API_KEY",
        "api_key",
        "api_secret",
    ]
}

pub fn contains_denied_pattern(text: &str) -> Option<&'static str> {
    deny_patterns().into_iter().find(|pat| text.contains(pat))
}
