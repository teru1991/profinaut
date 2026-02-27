use bytes::Bytes;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use ucel_cex_coincheck::CoincheckRestAdapter;
use ucel_core::{ErrorCode, UcelError};
use ucel_transport::{
    HttpRequest, HttpResponse, RequestContext, Transport, WsConnectRequest, WsStream,
};

#[derive(Debug, Deserialize)]
struct Catalog {
    rest_endpoints: Vec<CatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct CatalogEntry {
    id: String,
    path: String,
    auth: CatalogAuth,
}

#[derive(Debug, Deserialize)]
struct CatalogAuth {
    #[serde(rename = "type")]
    auth_type: String,
}

#[derive(Default)]
struct FixtureTransport {
    calls: AtomicUsize,
    fixtures: Mutex<HashMap<String, Bytes>>,
}

impl FixtureTransport {
    fn new(fixtures: HashMap<String, Bytes>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            fixtures: Mutex::new(fixtures),
        }
    }
}

impl Transport for FixtureTransport {
    async fn send_http(
        &self,
        req: HttpRequest,
        _ctx: RequestContext,
    ) -> Result<HttpResponse, UcelError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let path = req
            .path
            .split("coincheck.test")
            .last()
            .unwrap_or(&req.path)
            .to_string();
        let body = self
            .fixtures
            .lock()
            .unwrap()
            .get(&path)
            .cloned()
            .ok_or_else(|| UcelError::new(ErrorCode::Internal, "missing fixture"))?;
        Ok(HttpResponse { status: 200, body })
    }

    async fn connect_ws(
        &self,
        _req: WsConnectRequest,
        _ctx: RequestContext,
    ) -> Result<WsStream, UcelError> {
        Ok(WsStream::default())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn rest_catalog_all_rows_are_tested_with_typed_parse() {
    let raw = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../docs/exchanges/coincheck/catalog.json"),
    )
    .unwrap();
    let catalog: Catalog = serde_json::from_str(&raw).unwrap();

    let mut fixtures = HashMap::new();
    for e in &catalog.rest_endpoints {
        let filename = format!("{}.json", e.id);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(filename);
        fixtures.insert(e.path.clone(), Bytes::from(std::fs::read(&path).unwrap()));
        fixtures.insert(
            e.path.replace("{id}", "1").replace("{pair}", "btc_jpy"),
            Bytes::from(std::fs::read(path).unwrap()),
        );
    }

    let adapter = CoincheckRestAdapter::new("http://coincheck.test");
    let transport = Arc::new(FixtureTransport::new(fixtures));

    for e in &catalog.rest_endpoints {
        let mut params = HashMap::new();
        if e.path.contains("{id}") {
            params.insert("id".to_string(), "1".to_string());
        }
        if e.path.contains("{pair}") {
            params.insert("pair".to_string(), "btc_jpy".to_string());
        }
        let key = if e.auth.auth_type == "signature" {
            Some("k".to_string())
        } else {
            None
        };
        let out = adapter
            .execute_rest(&*transport, &e.id, &params, None, key)
            .await;
        assert!(out.is_ok(), "id={} should parse typed fixture", e.id);
    }

    assert_eq!(
        transport.calls.load(Ordering::SeqCst),
        catalog.rest_endpoints.len()
    );
}
