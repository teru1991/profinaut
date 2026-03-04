use crate::diagnostics::limits::{BundleBuildError, BundleLimits};

pub fn normalize_and_validate(rel: &str, limits: &BundleLimits) -> Result<String, BundleBuildError> {
    if rel.is_empty() {
        return Err(BundleBuildError::InvalidPath(rel.to_string()));
    }
    if rel.len() > limits.max_path_len {
        return Err(BundleBuildError::PathTooLong(rel.to_string()));
    }
    if rel.starts_with('/') || rel.starts_with('\\') || rel.contains(':') {
        return Err(BundleBuildError::InvalidPath(rel.to_string()));
    }
    if rel.contains("..") {
        return Err(BundleBuildError::InvalidPath(rel.to_string()));
    }
    if !rel
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
    {
        return Err(BundleBuildError::InvalidPath(rel.to_string()));
    }

    let mut out = String::with_capacity(rel.len());
    let mut last_slash = false;
    for ch in rel.chars() {
        if ch == '/' {
            if last_slash {
                continue;
            }
            last_slash = true;
            out.push(ch);
        } else {
            last_slash = false;
            out.push(ch);
        }
    }

    while out.starts_with("./") {
        out = out.trim_start_matches("./").to_string();
    }
    if out.is_empty() || out.ends_with('/') {
        return Err(BundleBuildError::InvalidPath(rel.to_string()));
    }
    Ok(out)
}
