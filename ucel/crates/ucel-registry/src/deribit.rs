use crate::{CatalogAuth, CatalogEntry, ExchangeCatalog};
use serde::Deserialize;
use std::path::Path;
use ucel_core::{ErrorCode, OpMeta, OpName, UcelError};

#[derive(Debug, Clone, Deserialize)]
struct DeribitCatalog {
    exchange: String,
    rpc_http_methods: Vec<DeribitRpcMethod>,
    rpc_ws_methods: Vec<DeribitRpcMethod>,
    ws_subscriptions: Vec<DeribitWsSubscription>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeribitRpcMethod {
    id: String,
    base_url: String,
    method: String,
    operation: Option<String>,
    auth: CatalogAuth,
}

#[derive(Debug, Clone, Deserialize)]
struct DeribitWsSubscription {
    id: String,
    ws_url: String,
    channel: String,
    auth: CatalogAuth,
}

pub fn load_deribit_catalog_from_path(path: &Path) -> Result<ExchangeCatalog, UcelError> {
    let raw = std::fs::read_to_string(path).map_err(|err| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("failed to read {}: {err}", path.display()),
        )
    })?;
    let catalog: DeribitCatalog = serde_json::from_str(&raw).map_err(|err| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("failed to parse {}: {err}", path.display()),
        )
    })?;

    let mut rest_endpoints = Vec::with_capacity(catalog.rpc_http_methods.len());
    for row in catalog.rpc_http_methods {
        validate_non_empty(&row.id, "id")?;
        validate_non_empty(&row.base_url, "base_url")?;
        validate_non_empty(&row.method, "method")?;
        if !row.base_url.starts_with("https://") && !row.base_url.starts_with("http://") {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("invalid base_url for id={}: {}", row.id, row.base_url),
            ));
        }

        let visibility = derive_visibility_from_id(&row.id)?;
        rest_endpoints.push(CatalogEntry {
            id: row.id,
            visibility: Some(visibility),
            access: String::new(),
            operation: row.operation,
            method: Some("POST".to_string()),
            base_url: Some(row.base_url),
            path: Some(format!("/{}", row.method)),
            ws_url: None,
            ws: None,
            auth: row.auth,
            requires_auth: None,
        });
    }

    let mut ws_channels =
        Vec::with_capacity(catalog.rpc_ws_methods.len() + catalog.ws_subscriptions.len());

    for row in catalog.rpc_ws_methods {
        validate_non_empty(&row.id, "id")?;
        validate_non_empty(&row.base_url, "base_url")?;
        validate_non_empty(&row.method, "method")?;
        if !row.base_url.starts_with("wss://") && !row.base_url.starts_with("ws://") {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("invalid ws_url for id={}: {}", row.id, row.base_url),
            ));
        }

        let visibility = derive_visibility_from_id(&row.id)?;
        ws_channels.push(CatalogEntry {
            id: row.id,
            visibility: Some(visibility),
            access: String::new(),
            operation: row.operation,
            method: None,
            base_url: None,
            path: None,
            ws_url: Some(row.base_url),
            ws: None,
            auth: row.auth,
            requires_auth: None,
        });
    }

    for row in catalog.ws_subscriptions {
        validate_non_empty(&row.id, "id")?;
        validate_non_empty(&row.ws_url, "ws_url")?;
        validate_non_empty(&row.channel, "channel")?;
        if !row.ws_url.starts_with("wss://") && !row.ws_url.starts_with("ws://") {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("invalid ws_url for id={}: {}", row.id, row.ws_url),
            ));
        }

        let visibility = Some(derive_visibility_from_id(&row.id)?);
        ws_channels.push(CatalogEntry {
            id: row.id,
            visibility,
            access: String::new(),
            operation: Some(row.channel.clone()),
            method: None,
            base_url: None,
            path: None,
            ws_url: Some(row.ws_url),
            ws: None,
            auth: row.auth,
            requires_auth: None,
        });
    }

    let normalized = ExchangeCatalog {
        exchange: catalog.exchange,
        rest_endpoints,
        ws_channels,
    };

    crate::validate_catalog(&normalized)?;
    Ok(normalized)
}

pub fn map_deribit_operation(entry: &CatalogEntry) -> Result<OpMeta, UcelError> {
    let op = map_deribit_op_name(&entry.id);
    let requires_auth = derive_visibility_from_id(&entry.id)? == "private";
    Ok(OpMeta { op, requires_auth })
}

pub fn map_deribit_op_name(id: &str) -> OpName {
    if id.contains("ticker") {
        OpName::FetchTicker
    } else if id.contains("order_book") || id.contains("book.") {
        OpName::FetchOrderbookSnapshot
    } else if id.contains("trades") {
        OpName::FetchTrades
    } else if id.contains("chart") || id.contains("tradingview") {
        OpName::FetchKlines
    } else if id.contains("buy") || id.contains("sell") {
        OpName::PlaceOrder
    } else if id.contains("cancel") {
        OpName::CancelOrder
    } else if id.contains("account_summary") || id.contains("portfolio") {
        OpName::FetchBalances
    } else if id.contains("auth") {
        OpName::CreateWsAuthToken
    } else if id.contains("user.orders") {
        OpName::SubscribeOrderEvents
    } else if id.contains("user.trades") || id.contains("user.changes") {
        OpName::SubscribeExecutionEvents
    } else if id.contains("set_heartbeat") {
        OpName::FetchStatus
    } else {
        OpName::FetchStatus
    }
}

fn derive_visibility_from_id(id: &str) -> Result<String, UcelError> {
    if id.contains(".private.") {
        Ok("private".to_string())
    } else if id.contains(".public.") {
        Ok("public".to_string())
    } else {
        Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("id does not encode visibility: {id}"),
        ))
    }
}

fn validate_non_empty(value: &str, field: &str) -> Result<(), UcelError> {
    if value.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("{field} must not be empty"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn loads_deribit_catalog_and_counts_all_rest_and_ws_rows() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let path = repo_root.join("docs/exchanges/deribit/catalog.json");
        let catalog = load_deribit_catalog_from_path(&path).unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 9);
        assert_eq!(catalog.ws_channels.len(), 19);
    }

    #[test]
    fn derives_requires_auth_from_visibility_for_deribit() {
        let entry = CatalogEntry {
            id: "jsonrpc.ws.private.trading.private_buy".to_string(),
            visibility: Some("private".to_string()),
            access: String::new(),
            operation: Some("place buy order".to_string()),
            method: None,
            base_url: None,
            path: None,
            ws_url: Some("wss://www.deribit.com/ws/api/v2".to_string()),
            ws: None,
            auth: CatalogAuth {
                auth_type: "token".to_string(),
            },
            requires_auth: Some(false),
        };
        let meta = map_deribit_operation(&entry).unwrap();
        assert_eq!(meta.op, OpName::PlaceOrder);
        assert!(meta.requires_auth);
    }
}
