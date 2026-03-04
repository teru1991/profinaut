use regex::Regex;

#[derive(Debug, Clone)]
pub struct ResidualScanner {
    deny: Vec<Regex>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResidualScanError {
    #[error("regex compile error: {0}")]
    Regex(String),
}

#[derive(Debug, Clone)]
pub struct ResidualFinding {
    pub pattern_index: usize,
    pub snippet_hash: String,
}

impl ResidualScanner {
    pub fn new(patterns: &[String]) -> Result<Self, ResidualScanError> {
        let mut deny = Vec::with_capacity(patterns.len());
        for p in patterns {
            deny.push(Regex::new(p).map_err(|e| ResidualScanError::Regex(e.to_string()))?);
        }
        Ok(Self { deny })
    }

    pub fn scan(&self, bytes: &[u8]) -> Vec<ResidualFinding> {
        let s = String::from_utf8_lossy(bytes);
        let mut out = Vec::new();
        for (i, re) in self.deny.iter().enumerate() {
            if let Some(m) = re.find(&s) {
                let snippet = &s[m.start()..m.end().min(m.start() + 64)];
                let h = sha256_hex(snippet.as_bytes());
                out.push(ResidualFinding {
                    pattern_index: i,
                    snippet_hash: h,
                });
            }
        }
        out
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}
