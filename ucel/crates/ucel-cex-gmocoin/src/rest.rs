use hmac::{Hmac, Mac};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;

const PUBLIC_BASE: &str = "https://api.coin.z.com/public";
const PRIVATE_BASE: &str = "https://api.coin.z.com/private";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
struct WsAuthResp {
    status: u16,
    data: String, // token
}

#[derive(Clone, Debug)]
pub struct GmoCredentials {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Clone, Debug)]
pub struct GmoRest {
    client: reqwest::Client,
    cred: Option<GmoCredentials>,
}

#[derive(Debug, Serialize)]
struct PrivateHeaders<'a> {
    #[serde(rename = "API-KEY")]
    api_key: &'a str,
    #[serde(rename = "API-TIMESTAMP")]
    api_timestamp: &'a str,
    #[serde(rename = "API-SIGN")]
    api_sign: &'a str,
}

impl GmoRest {
    pub fn new_public() -> Self {
        Self {
            client: reqwest::Client::new(),
            cred: None,
        }
    }

    pub fn new_with_credentials(cred: GmoCredentials) -> Self {
        Self {
            client: reqwest::Client::new(),
            cred: Some(cred),
        }
    }

    fn sign(&self, timestamp: &str, method: &Method, path: &str, body: &str) -> Result<String, String> {
        let cred = self
            .cred
            .as_ref()
            .ok_or_else(|| "missing credentials".to_string())?;

        // sign payload: timestamp + method + path + body
        let payload = format!("{}{}{}{}", timestamp, method.as_str(), path, body);

        let mut mac = HmacSha256::new_from_slice(cred.api_secret.as_bytes())
            .map_err(|e| format!("hmac init failed: {e}"))?;
        mac.update(payload.as_bytes());
        let sig = mac.finalize().into_bytes();
        Ok(hex::encode(sig))
    }

    async fn request_public_any(
        &self,
        method: Method,
        path: &str,
        query: Option<BTreeMap<String, String>>,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{PUBLIC_BASE}{path}");
        let mut req = self.client.request(method, &url);

        if let Some(q) = query {
            req = req.query(&q);
        }
        if let Some(b) = body {
            req = req.json(&b);
        }

        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("gmo public error status={}", resp.status()));
        }
        resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())
    }

    async fn request_private_any<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        query: Option<BTreeMap<String, String>>,
        body: Option<serde_json::Value>,
    ) -> Result<T, String> {
        let cred = self
            .cred
            .as_ref()
            .ok_or_else(|| "missing credentials".to_string())?;

        let url = format!("{PRIVATE_BASE}{path}");

        let timestamp = format!("{}", chrono::Utc::now().timestamp_millis());
        let body_str = body
            .as_ref()
            .map(|b| serde_json::to_string(b).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "".to_string());

        let sign = self.sign(&timestamp, &method, path, &body_str)?;

        let mut req = self.client.request(method, &url);

        if let Some(q) = query {
            req = req.query(&q);
        }
        if body.is_some() {
            req = req.body(body_str.clone());
            req = req.header("Content-Type", "application/json");
        }

        req = req
            .header("API-KEY", &cred.api_key)
            .header("API-TIMESTAMP", &timestamp)
            .header("API-SIGN", &sign);

        let resp = req.send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("gmo private error status={}", resp.status()));
        }
        resp.json::<T>().await.map_err(|e| e.to_string())
    }

    // ----------------------------
    // Public endpoints
    // ----------------------------

    /// GET /public/v1/status  [oai_citation:9‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_status(&self) -> Result<serde_json::Value, String> {
        self.request_public_any(Method::GET, "/v1/status", None, None)
            .await
    }

    /// GET /public/v1/ticker  [oai_citation:10‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_ticker(&self, symbol: &str) -> Result<serde_json::Value, String> {
        let mut q = BTreeMap::new();
        q.insert("symbol".to_string(), symbol.to_string());
        self.request_public_any(Method::GET, "/v1/ticker", Some(q), None)
            .await
    }

    /// GET /public/v1/orderbooks  [oai_citation:11‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_orderbooks(
        &self,
        symbol: &str,
        page: Option<u32>,
        count: Option<u32>,
    ) -> Result<serde_json::Value, String> {
        let mut q = BTreeMap::new();
        q.insert("symbol".to_string(), symbol.to_string());
        if let Some(p) = page {
            q.insert("page".to_string(), p.to_string());
        }
        if let Some(c) = count {
            q.insert("count".to_string(), c.to_string());
        }
        self.request_public_any(Method::GET, "/v1/orderbooks", Some(q), None)
            .await
    }

    /// GET /public/v1/trades  [oai_citation:12‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_trades(
        &self,
        symbol: &str,
        page: Option<u32>,
        count: Option<u32>,
    ) -> Result<serde_json::Value, String> {
        let mut q = BTreeMap::new();
        q.insert("symbol".to_string(), symbol.to_string());
        if let Some(p) = page {
            q.insert("page".to_string(), p.to_string());
        }
        if let Some(c) = count {
            q.insert("count".to_string(), c.to_string());
        }
        self.request_public_any(Method::GET, "/v1/trades", Some(q), None)
            .await
    }

    /// GET /public/v1/symbols  [oai_citation:13‡Coin API](https://api.coin.z.com/docs/)
    pub async fn public_symbols(&self) -> Result<serde_json::Value, String> {
        self.request_public_any(Method::GET, "/v1/symbols", None, None)
            .await
    }

    // ----------------------------
    // Private WS token (ws-auth)
    // ----------------------------

    /// POST /private/v1/ws-auth -> token  [oai_citation:14‡Coin API](https://api.coin.z.com/docs/)
    pub async fn ws_auth_create(&self) -> Result<String, String> {
        let r: WsAuthResp = self
            .request_private_any(Method::POST, "/v1/ws-auth", None, Some(serde_json::json!({})))
            .await?;
        Ok(r.data)
    }

    /// DELETE /private/v1/ws-auth (token) – docs show token deletion is supported (examples exist)
    pub async fn ws_auth_delete(&self, token: &str) -> Result<serde_json::Value, String> {
        self.request_private_any(
            Method::DELETE,
            "/v1/ws-auth",
            None,
            Some(serde_json::json!({ "token": token })),
        )
            .await
    }
}
