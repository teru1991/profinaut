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
        path: "/v1/me/getbalance".into(),
        query: BTreeMap::new(),
        body: String::new(),
        headers: BTreeMap::from([
            ("ACCESS-KEY".into(), api_key.into()),
            ("ACCESS-TIMESTAMP".into(), timestamp.into()),
            ("ACCESS-SIGN".into(), signature.into()),
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
        path: "/v1/me/sendchildorder".into(),
        query: BTreeMap::new(),
        body: body.into(),
        headers: BTreeMap::from([
            ("ACCESS-KEY".into(), api_key.into()),
            ("ACCESS-TIMESTAMP".into(), timestamp.into()),
            ("ACCESS-SIGN".into(), signature.into()),
            ("Content-Type".into(), "application/json".into()),
        ]),
    }
}
