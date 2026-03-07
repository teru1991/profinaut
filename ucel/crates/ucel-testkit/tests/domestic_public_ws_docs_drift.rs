use ucel_testkit::domestic_public_ws::{load_ws_fixture_bundle, repo_root};

#[test]
fn runtime_matrix_contains_fixture_channels() {
    let root = repo_root();
    let runtime_matrix = std::fs::read_to_string(
        root.join("ucel/docs/exchanges/domestic_public_ws_runtime_matrix.md"),
    )
    .expect("read runtime matrix");

    let fixtures = load_ws_fixture_bundle(&root).expect("load fixture bundle");
    for channel in fixtures.channels {
        assert!(
            runtime_matrix.contains(&channel.public_id),
            "runtime matrix missing {}",
            channel.public_id
        );
    }
}

#[test]
fn integrity_policy_mentions_all_modes() {
    let root = repo_root();
    let policy = std::fs::read_to_string(
        root.join("ucel/docs/exchanges/domestic_public_ws_integrity_policy.md"),
    )
    .expect("read integrity policy");

    for mode in [
        "none",
        "snapshot_only",
        "sequence_only",
        "checksum_only",
        "sequence_and_checksum",
    ] {
        assert!(policy.contains(mode), "missing mode in policy: {mode}");
    }
}
