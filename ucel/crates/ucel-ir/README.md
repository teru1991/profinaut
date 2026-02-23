# ucel-ir

`ucel-ir` provides statutory filings ingestion scaffolding and recorded providers for:
- **JP EDINET** (optional API key; missing key degrades gracefully)
- **US SEC EDGAR** (required `user_agent`)

## Configuration sketch

```rust
use std::path::PathBuf;
use ucel_ir::{EdinetConfig, SecEdgarConfig};

let edinet = EdinetConfig {
    api_key: None, // optional: provider still runs in degraded mode when absent
    fixtures_dir: Some(PathBuf::from("testdata/edinet")),
    list_url: "https://api.edinet-fsa.go.jp/api/v2/documents.json".to_string(),
};

let mut sec = SecEdgarConfig::with_defaults(
    "my-app/1.0 contact@example.com",
    PathBuf::from("testdata/sec"),
);
sec.tickers = vec!["AAPL".to_string()];
```

Use `FsRawSink`/`FsCheckpointStore` for filesystem persistence paths in production.

## Offline tests

```bash
cd ucel
cargo test -p ucel-ir
```
