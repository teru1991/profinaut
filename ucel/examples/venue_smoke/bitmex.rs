use serde_yaml::Value;
use std::fs;

fn main() {
    let venue = "bitmex";
    let path = format!("ucel/coverage/{}.yaml", venue);
    let yaml = fs::read_to_string(&path).expect("read coverage yaml");
    let v: Value = serde_yaml::from_str(&yaml).expect("parse yaml");

    let strict = v.get("strict").and_then(|x| x.as_bool()).unwrap_or(false);
    let entries = v
        .get("entries")
        .and_then(|x| x.as_sequence())
        .cloned()
        .unwrap_or_default();

    println!("venue={} strict={}", venue, strict);

    for e in entries {
        let id = e.get("id").and_then(|x| x.as_str()).unwrap_or("");
        let support = e
            .get("support")
            .and_then(|x| x.as_str())
            .unwrap_or("supported");
        if support == "not_supported" {
            continue;
        }
        if id.starts_with("crypto.public.ws.") || id.starts_with("crypto.private.ws.") {
            println!("{}", id);
        }
    }
}
