use serde_json::Value;
use std::path::Path;

pub use crate::fixtures::{discover_ws_cases, repo_root_from_manifest_dir, GoldenWsCase};
use crate::normalize::{canonicalize_json, first_diff_path};

#[derive(Debug, Clone)]
pub struct GoldenWsFixture {
    pub venue: String,
    pub name: String,
    pub raw: String,
    pub expected: Value,
}

impl GoldenWsFixture {
    pub fn load(repo_root: &Path, venue: &str, name: &str) -> Result<Self, String> {
        let base = repo_root
            .join("ucel")
            .join("fixtures")
            .join("golden")
            .join("ws")
            .join(venue);
        let raw_path = base.join("raw.json");
        let expected_path = base.join("expected.normalized.json");

        let raw = std::fs::read_to_string(&raw_path)
            .map_err(|e| format!("failed to read raw fixture {}: {}", raw_path.display(), e))?;
        let expected_raw = std::fs::read_to_string(&expected_path).map_err(|e| {
            format!(
                "failed to read expected fixture {}: {}",
                expected_path.display(),
                e
            )
        })?;
        let expected: Value = serde_json::from_str(&expected_raw).map_err(|e| {
            format!(
                "failed to parse expected json {}: {}",
                expected_path.display(),
                e
            )
        })?;

        Ok(Self {
            venue: venue.to_string(),
            name: name.to_string(),
            raw,
            expected,
        })
    }
}

pub fn assert_json_eq(actual: &Value, expected: &Value, context: &str) {
    let a = canonicalize_json(actual);
    let e = canonicalize_json(expected);
    if a == e {
        return;
    }

    let diff_path = first_diff_path(&a, &e).unwrap_or_else(|| "$".to_string());
    panic!(
        "golden mismatch ({context})\nfirst_diff_path: {diff_path}\nactual:\n{}\nexpected:\n{}\n",
        serde_json::to_string_pretty(&a).unwrap_or_else(|_| "<json>".into()),
        serde_json::to_string_pretty(&e).unwrap_or_else(|_| "<json>".into()),
    );
}

pub fn run_ws_case(case: &GoldenWsCase) -> Result<Value, String> {
    match case.venue.as_str() {
        "bithumb" => {
            let evt = ucel_cex_bithumb::normalize_ws_event(&case.endpoint_id, &case.raw_payload)
                .map_err(|e| {
                    format!(
                        "venue={} case={} endpoint={} decode failed raw={} err={}",
                        case.venue,
                        case.case_name,
                        case.endpoint_id,
                        case.raw_path.display(),
                        e
                    )
                })?;
            serde_json::to_value(evt).map_err(|e| {
                format!(
                    "venue={} case={} endpoint={} serialize failed expected={} err={}",
                    case.venue,
                    case.case_name,
                    case.endpoint_id,
                    case.expected_path.display(),
                    e
                )
            })
        }
        "bybit" => {
            let evt = ucel_cex_bybit::normalize_ws_event(&case.endpoint_id, &case.raw_payload)
                .map_err(|e| {
                    format!(
                        "venue={} case={} endpoint={} decode failed raw={} err={}",
                        case.venue,
                        case.case_name,
                        case.endpoint_id,
                        case.raw_path.display(),
                        e
                    )
                })?;
            serde_json::to_value(evt).map_err(|e| {
                format!(
                    "venue={} case={} endpoint={} serialize failed expected={} err={}",
                    case.venue,
                    case.case_name,
                    case.endpoint_id,
                    case.expected_path.display(),
                    e
                )
            })
        }
        other => Err(format!(
            "unsupported golden venue={other} case={}",
            case.case_name
        )),
    }
}

pub fn run_ws_venue(repo_root: &Path, venue: &str) -> Result<usize, String> {
    let cases = discover_ws_cases(repo_root, venue)?;
    if cases.is_empty() {
        return Err(format!(
            "no ws golden fixtures found for strict venue={venue}"
        ));
    }

    for case in &cases {
        let actual = run_ws_case(case)?;
        assert_json_eq(
            &actual,
            &case.expected,
            &format!("venue={} case={}", case.venue, case.case_name),
        );
    }

    Ok(cases.len())
}
