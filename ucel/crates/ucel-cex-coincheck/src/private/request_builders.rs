use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateRequestShape {
    pub method: &'static str,
    pub path: String,
    pub query: BTreeMap<String, String>,
    pub body: String,
    pub headers: BTreeMap<String, String>,
}

pub fn build_get_balance_request(
    api_key: &str,
    nonce: &str,
    signature: &str,
) -> PrivateRequestShape {
    PrivateRequestShape {
        method: "GET",
        path: "/api/accounts/balance".into(),
        query: BTreeMap::new(),
        body: String::new(),
        headers: BTreeMap::from([
            ("ACCESS-KEY".into(), api_key.into()),
            ("ACCESS-NONCE".into(), nonce.into()),
            ("ACCESS-SIGNATURE".into(), signature.into()),
        ]),
    }
}

pub fn build_post_order_request(
    api_key: &str,
    nonce: &str,
    signature: &str,
    body: &str,
) -> PrivateRequestShape {
    PrivateRequestShape {
        method: "POST",
        path: "/api/exchange/orders".into(),
        query: BTreeMap::new(),
        body: body.into(),
        headers: BTreeMap::from([
            ("ACCESS-KEY".into(), api_key.into()),
            ("ACCESS-NONCE".into(), nonce.into()),
            ("ACCESS-SIGNATURE".into(), signature.into()),
            ("Content-Type".into(), "application/json".into()),
        ]),
    }
}
