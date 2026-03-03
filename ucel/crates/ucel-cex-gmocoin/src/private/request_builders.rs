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
    timestamp: &str,
    signature: &str,
) -> PrivateRequestShape {
    PrivateRequestShape {
        method: "GET",
        path: "/v1/account/assets".into(),
        query: BTreeMap::new(),
        body: String::new(),
        headers: BTreeMap::from([
            ("API-KEY".into(), api_key.into()),
            ("API-TIMESTAMP".into(), timestamp.into()),
            ("API-SIGN".into(), signature.into()),
        ]),
    }
}

pub fn build_post_order_request(
    api_key: &str,
    timestamp: &str,
    signature: &str,
    body: &str,
) -> PrivateRequestShape {
    PrivateRequestShape {
        method: "POST",
        path: "/v1/order".into(),
        query: BTreeMap::new(),
        body: body.into(),
        headers: BTreeMap::from([
            ("API-KEY".into(), api_key.into()),
            ("API-TIMESTAMP".into(), timestamp.into()),
            ("API-SIGN".into(), signature.into()),
            ("Content-Type".into(), "application/json".into()),
        ]),
    }
}
