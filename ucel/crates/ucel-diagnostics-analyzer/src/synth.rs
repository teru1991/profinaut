use crate::read::{ManifestFile, ParsedManifest};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Write;

pub struct SynthBundle {
    pub manifest: ParsedManifest,
    pub tar_zst_bytes: Vec<u8>,
}

pub fn build_minimal_bundle_v1() -> SynthBundle {
    build_bundle_with_diag_semver("1.0.0")
}

pub fn build_minimal_bundle_v2_major2() -> SynthBundle {
    build_bundle_with_diag_semver("2.0.0")
}

fn build_bundle_with_diag_semver(diag_semver: &str) -> SynthBundle {
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    files.insert("logs/tail.txt".into(), b"hello\n".to_vec());
    files.insert("meta/diag_semver.txt".into(), format!("{diag_semver}\n").into_bytes());
    files.insert("meta/info.json".into(), br#"{"k":"v"}"#.to_vec());

    let manifest = ParsedManifest {
        bundle_id: "bundle_minimal".to_string(),
        created_at_rfc3339: "2026-01-01T00:00:00Z".to_string(),
        diag_semver: diag_semver.to_string(),
        files: files
            .iter()
            .map(|(path, bytes)| ManifestFile {
                path: path.clone(),
                size_bytes: bytes.len() as u64,
                sha256_hex: sha256_hex(bytes),
            })
            .collect(),
    };

    let mut tar_buf = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_buf);

        let manifest_json = serde_json::to_vec(&manifest).expect("serialize manifest");
        add_tar_file(&mut builder, "manifest.json", &manifest_json);

        for (path, bytes) in &files {
            add_tar_file(&mut builder, path, bytes);
        }

        builder.finish().expect("finish tar archive");
    }

    let mut encoder = zstd::Encoder::new(Vec::new(), 3).expect("create zstd encoder");
    encoder.write_all(&tar_buf).expect("compress tar bytes");
    let tar_zst_bytes = encoder.finish().expect("finish zstd encoding");

    SynthBundle {
        manifest,
        tar_zst_bytes,
    }
}

fn add_tar_file(builder: &mut tar::Builder<&mut Vec<u8>>, path: &str, bytes: &[u8]) {
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, bytes)
        .expect("append tar entry");
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
