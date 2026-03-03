use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=../../../docs/ssot/market_meta_catalog.json");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let ssot_path = manifest_dir.join("../../../docs/ssot/market_meta_catalog.json");
    let content =
        fs::read_to_string(&ssot_path).expect("failed to read SSOT market_meta_catalog.json");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let out_path = out_dir.join("embedded_catalog.rs");
    let rust = format!("pub const EMBEDDED_CATALOG_JSON: &str = r###\"{content}\"###;");
    fs::write(out_path, rust).expect("failed to write embedded_catalog.rs");
}
