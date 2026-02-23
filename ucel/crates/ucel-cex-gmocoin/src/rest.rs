use hmac::{Hmac, Mac};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;

const PUBLIC_BASE: &str = "https://api.coin.z.com/public";
const PRIVATE_BASE: &str = "https://api.coin.z.com/private";

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug)]
pub struct GmoCredentials {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Clone)]
pub struct GmoRest {
    client: reqwest::Client,
    creds: Option<GmoCredentials>,
}

impl GmoRest {
    pub fn new_public_only() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self { client, creds: None })
    }

    pub fn new_with_credentials(creds: GmoCredentials) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self { client, creds: Some(creds) })
    }

    fn timestamp_ms() -> String {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        ms.to_string()
    }

    fn sign(secret: &str, timestamp: &str, method: &str, path: &str, body: &str) -> String {
        let text = format!("{timestamp}{method}{path}{body}");
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac key");
        mac.update(text.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn build_query_string(params: &BTreeMap<String, String>) -> String {
        // deterministic & urlencoded
        let mut pairs: Vec<String> = Vec::new();
        for (k, v) in params {
            let k = urlencoding::encode(k);
            let v = urlencoding::encode(v);
            pairs.push(format!("{k}={v}"));
        }
        pairs.join("&")
    }

    pub async fn request_public_any<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str, // must start with "/v1/..."
        query: Option<BTreeMap<String, String>>,
        body_json: Option<serde_json::Value>,
    ) -> Result<T, String> {
        let mut url = format!("{PUBLIC_BASE}{path}");
        if let Some(q) = query {
            let qs = Self::build_query_string(&q);
            if !qs.is_empty() {
                url.push('?');
                url.push_str(&qs);
            }
        }
        let mut req = self.client.request(method, url);

        if let Some(b) = body_json {
            req = req.json(&b);
        }

        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(format!("public http status={} body={txt}", resp.status()));
        }
        resp.json::<T>().await.map_err(|e| e.to_string())
    }

    pub async fn request_private_any<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str, // must start with "/v1/..."
        query: Option<BTreeMap<String, String>>,
        body_json: Option<serde_json::Value>,
    ) -> Result<T, String> {
        let creds = self.creds.as_ref().ok_or_else(|| "missing creds".to_string())?;

        let mut url = format!("{PRIVATE_BASE}{path}");
        if let Some(q) = query.clone() {
            let qs = Self::build_query_string(&q);
            if !qs.is_empty() {
                url.push('?');
                url.push_str(&qs);
            }
        }

        let body_str = match &body_json {
            Some(v) => v.to_string(),
            None => "{}".to_string(),
        };

        let ts = Self::timestamp_ms();
        let sign = Self::sign(&creds.api_secret, &ts, method.as_str(), path, &body_str);

        let mut req = self
            .client
            .request(method, url)
            .header("API-KEY", &creds.api_key)
            .header("API-TIMESTAMP", ts)
            .header("API-SIGN", sign);

        if let Some(b) = body_json {
            req = req.json(&b);
        }

        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(format!("private http status={} body={txt}", resp.status()));
        }
        resp.json::<T>().await.map_err(|e| e.to_string())
    }

    // ----------------------------
    // Typed helpers (core)
    // ----------------------------

    /// GET /public/v1/status  [oai_citation:9‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_status(&self) -> Result<serde_json::Value, String> {
        self.request_public_any(Method::GET, "/v1/status", None, None).await
    }

    /// GET /public/v1/ticker (symbol optional)  [oai_citation:10‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_ticker(&self, symbol: Option<&str>) -> Result<serde_json::Value, String> {
        let mut q = BTreeMap::new();
        if let Some(s) = symbol {
            q.insert("symbol".to_string(), s.to_string());
        }
        let q = if q.is_empty() { None } else { Some(q) };
        self.request_public_any(Method::GET, "/v1/ticker", q, None).await
    }

    /// GET /public/v1/orderbooks (symbol required)  [oai_citation:11‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_orderbooks(&self, symbol: &str) -> Result<serde_json::Value, String> {
        let mut q = BTreeMap::new();
        q.insert("symbol".to_string(), symbol.to_string());
        self.request_public_any(Method::GET, "/v1/orderbooks", Some(q), None).await
    }

    /// GET /public/v1/trades (symbol required, page/count optional)  [oai_citation:12‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_trades(&self, symbol: &str, page: Option<u32>, count: Option<u32>) -> Result<serde_json::Value, String> {
        let mut q = BTreeMap::new();
        q.insert("symbol".to_string(), symbol.to_string());
        if let Some(p) = page { q.insert("page".to_string(), p.to_string()); }
        if let Some(c) = count { q.insert("count".to_string(), c.to_string()); }
        self.request_public_any(Method::GET, "/v1/trades", Some(q), None).await
    }

    /// GET /public/v1/symbols  [oai_citation:13‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_symbols(&self) -> Result<serde_json::Value, String> {
        self.request_public_any(Method::GET, "/v1/symbols", None, None).await
    }

    // ----------------------------
    // Private WS token (ws-auth)
    // ----------------------------

    #[derive(Debug, Deserialize)]
    pub struct WsAuthResp {
        pub status: u16,
        pub data: String, // token
    }

    /// POST /private/v1/ws-auth -> token  [oai_citation:14‡Coin API](https://api.coin.z.com/docs/)
    pub async fn ws_auth_create(&self) -> Result<String, String> {
        let r: WsAuthResp = self.request_private_any(Method::POST, "/v1/ws-auth", None, Some(serde_json::json!({}))).await?;
        Ok(r.data)
    }

    /// DELETE /private/v1/ws-auth (token) – docs show token deletion is supported (examples exist)
    pub async fn ws_auth_delete(&self, token: &str) -> Result<serde_json::Value, String> {
        self.request_private_any(Method::DELETE, "/v1/ws-auth", None, Some(serde_json::json!({ "token": token }))).await
    }
}