use serde_json::Value;
use ucel_testkit::fuzz::{json_depth, ws_guard_entry};
use ucel_testkit::fuzz_corpus::{load_json_corpus, CorpusConfig};

const MAX_DEPTH: usize = 64;

fn deeply_nested_json(depth: usize) -> Vec<u8> {
    let mut s = String::new();
    for _ in 0..depth {
        s.push_str("{\"a\":");
    }
    s.push('0');
    for _ in 0..depth {
        s.push('}');
    }
    s.into_bytes()
}

#[test]
fn fuzz_json_depth_limit_returns_err_without_panicking() {
    let corpus = load_json_corpus(CorpusConfig::default()).expect("json corpus must load");
    for seed in corpus {
        let parsed: Value = serde_json::from_slice(&seed.bytes).expect("json seed must parse");
        let _ = json_depth(&parsed, MAX_DEPTH);
        let _ = ws_guard_entry(&seed.bytes);
    }

    let over_depth = deeply_nested_json(MAX_DEPTH + 8);
    assert!(ws_guard_entry(&over_depth).is_err());

    let parsed_over: Value = serde_json::from_slice(&over_depth).expect("generated deep json parses");
    assert!(json_depth(&parsed_over, MAX_DEPTH).is_err());
}
