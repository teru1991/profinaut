use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ucel_testkit::{run_ssot_integrity_gate, GateSeverity};

fn mk_temp_root(name: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ucel_ssot_gate_v2_{name}_{ts}"));
    fs::create_dir_all(&root).expect("create temp root");
    root
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdirs");
    }
    fs::write(path, content).expect("write file");
}

fn write_strict_venues(root: &Path, venues: &[&str]) {
    let body = format!(
        "{{\n  \"strict_ws_golden\": {},\n  \"strict_symbol_master\": []\n}}",
        serde_json::to_string(venues).expect("to json")
    );
    write_file(
        &root.join("ucel/coverage/coverage_v2/strict_venues.json"),
        &body,
    );
}

#[test]
fn v2_gate_fails_when_catalog_op_missing_in_coverage() {
    let root = mk_temp_root("missing_entry");

    write_file(
        &root.join("docs/exchanges/foo/catalog.json"),
        r#"{ "ws_channels": [ { "id": "crypto.public.ws.ticker" } ] }"#,
    );
    write_file(
        &root.join("ucel/coverage/coverage_v2/exchanges/foo.json"),
        r#"{ "exchange": "foo", "public": {"rest": true, "ws": true}, "private": {"enabled": false}, "ws_ops": ["crypto.public.ws.other"] }"#,
    );
    write_strict_venues(&root, &["foo"]);

    fs::create_dir_all(root.join("ucel/crates/ucel-cex-foo")).expect("mkdir crate");
    write_file(
        &root.join("ucel/crates/ucel-ws-rules/rules/foo.toml"),
        r#"name = "foo""#,
    );
    write_file(
        &root.join("ucel/examples/venue_smoke/foo.rs"),
        r#"fn main() {}"#,
    );

    let report = run_ssot_integrity_gate(&root).expect("run gate");
    assert!(report.has_failures(), "should fail");
    let s = report.format_human_readable();
    assert!(
        s.contains("COVERAGE_MISSING_ENTRY"),
        "expected missing entry failure, got:\n{s}"
    );
}

#[test]
fn v2_gate_fails_when_coverage_file_missing_for_catalog_venue() {
    let root = mk_temp_root("missing_file");

    write_file(
        &root.join("docs/exchanges/foo/catalog.json"),
        r#"{ "ws_channels": [ { "id": "crypto.public.ws.ticker" } ] }"#,
    );
    write_strict_venues(&root, &["foo"]);

    let report = run_ssot_integrity_gate(&root).expect("run gate");
    assert!(report.has_failures(), "should fail");
    let s = report.format_human_readable();
    assert!(
        s.contains("COVERAGE_MISSING_FILE"),
        "expected missing file failure, got:\n{s}"
    );
}

#[test]
fn v2_gate_warns_missing_rules_and_examples_when_non_strict() {
    let root = mk_temp_root("non_strict_warn");

    write_file(
        &root.join("docs/exchanges/foo/catalog.json"),
        r#"{ "ws_channels": [ { "id": "crypto.public.ws.ticker" } ] }"#,
    );
    write_file(
        &root.join("ucel/coverage/coverage_v2/exchanges/foo.json"),
        r#"{ "exchange": "foo", "public": {"rest": true, "ws": true}, "private": {"enabled": false}, "ws_ops": ["crypto.public.ws.ticker"] }"#,
    );
    write_strict_venues(&root, &[]);

    fs::create_dir_all(root.join("ucel/crates/ucel-cex-foo")).expect("mkdir crate");

    let report = run_ssot_integrity_gate(&root).expect("run gate");

    assert!(!report.has_failures(), "should not fail");
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.severity == GateSeverity::Warn && i.code == "RULES_MISSING"),
        "expected RULES_MISSING warn"
    );
    assert!(
        report
            .issues
            .iter()
            .any(|i| i.severity == GateSeverity::Warn && i.code == "EXAMPLE_MISSING"),
        "expected EXAMPLE_MISSING warn"
    );
}
