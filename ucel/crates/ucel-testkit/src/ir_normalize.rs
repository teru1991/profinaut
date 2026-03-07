use serde::Deserialize;
use std::{fs, io::Write, path::Path};

#[derive(Debug, Deserialize)]
pub struct ZipSpecEntry {
    pub path: String,
    pub kind: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ZipSpec {
    pub entries: Vec<ZipSpecEntry>,
}

pub fn load_text_fixture(path: &str) -> String {
    fs::read_to_string(path).expect("fixture must be readable")
}

pub fn build_pdf_bytes_from_text_fixture(path: &str) -> Vec<u8> {
    load_text_fixture(path).into_bytes()
}

pub fn build_zip_bytes_from_spec(path: &str) -> Vec<u8> {
    let spec: ZipSpec = serde_json::from_str(&load_text_fixture(path)).expect("valid zip spec json");
    let mut out = Vec::<u8>::new();
    {
        let cursor = std::io::Cursor::new(&mut out);
        let mut writer = zip::ZipWriter::new(cursor);
        let opts = zip::write::FileOptions::default();
        for e in spec.entries {
            writer.start_file(e.path, opts).unwrap();
            writer.write_all(e.text.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
    }
    out
}

pub fn write_temp_artifact_with_extension(ext: &str, bytes: &[u8]) -> tempfile::NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(&format!(".{ext}")).tempfile().unwrap();
    f.write_all(bytes).unwrap();
    f
}

pub fn fixture_path(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ir_normalize")
        .join(rel)
        .to_string_lossy()
        .to_string()
}
