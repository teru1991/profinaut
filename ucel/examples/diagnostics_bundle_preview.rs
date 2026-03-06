fn main() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let hashes = ucel_diagnostics_core::default_hash_set(&root).expect("hashes");
    println!("coverage_hash={}", hashes.coverage_hash);
    println!("coverage_v2_hash={}", hashes.coverage_v2_hash);
    println!("runtime_capability_hash={}", hashes.runtime_capability_hash);
}
