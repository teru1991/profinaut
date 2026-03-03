use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedCase {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusConfig {
    pub max_seed_bytes: usize,
}

impl Default for CorpusConfig {
    fn default() -> Self {
        Self {
            max_seed_bytes: 256 * 1024,
        }
    }
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/fuzz")
        .canonicalize()
        .expect("fixtures/fuzz path must exist")
}

fn load_with_ext(
    section: &str,
    ext: &str,
    cfg: CorpusConfig,
) -> Result<Vec<SeedCase>, Box<dyn std::error::Error>> {
    let dir = fixture_root().join(section);
    let mut out = Vec::new();

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some(ext) {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("invalid fixture filename: {}", path.display()))?
            .to_string();
        let bytes = fs::read(&path)?;
        if bytes.len() > cfg.max_seed_bytes {
            return Err(format!("seed too large: {name} is {} bytes", bytes.len()).into());
        }

        out.push(SeedCase { name, bytes });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    if out.is_empty() {
        return Err(format!("corpus is empty: {}", dir.display()).into());
    }
    Ok(out)
}

pub fn load_ws_frame_corpus(
    cfg: CorpusConfig,
) -> Result<Vec<SeedCase>, Box<dyn std::error::Error>> {
    load_with_ext("ws_frames", "txt", cfg)
}

pub fn load_json_corpus(cfg: CorpusConfig) -> Result<Vec<SeedCase>, Box<dyn std::error::Error>> {
    load_with_ext("json", "json", cfg)
}
