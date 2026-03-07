pub fn html_to_text(raw: &str) -> String {
    let mut s = raw.replace("\r\n", "\n");
    for tag in ["script", "style"] {
        while let Some(start) = s.to_ascii_lowercase().find(&format!("<{}", tag)) {
            if let Some(end) = s.to_ascii_lowercase()[start..].find(&format!("</{}>", tag)) {
                s.replace_range(start..start + end + tag.len() + 3, "");
            } else { break; }
        }
    }
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => { in_tag = false; out.push(' '); }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
