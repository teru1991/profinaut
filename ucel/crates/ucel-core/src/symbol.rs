/// Symbol normalization utilities (SSOT)
///
/// Canonical form:
/// - base/quote (e.g. BTC/JPY, BTC/USDT)
///
/// This module avoids per-exchange copy/paste bugs.
pub fn normalize_pair(raw: &str) -> String {
    // common delimiters: "_", "-", "/", ""
    if raw.contains('/') {
        raw.to_string()
    } else if raw.contains('_') {
        raw.replace('_', "/")
    } else if raw.contains('-') {
        raw.replace('-', "/")
    } else {
        // no delimiter: return as-is (some venues use BTCUSDT); caller may need mapping table
        raw.to_string()
    }
}

pub fn to_delim(raw_canonical_or_raw: &str, delim: char) -> String {
    let s = normalize_pair(raw_canonical_or_raw);
    if s.contains('/') {
        s.replace('/', &delim.to_string())
    } else {
        s
    }
}
