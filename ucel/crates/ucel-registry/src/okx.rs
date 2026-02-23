use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};
use ucel_core::{ErrorCode, OpMeta, OpName, UcelError};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OkxCatalog {
    pub exchange: String,
    pub rest_endpoints: Vec<OkxRestEntry>,
    pub ws_channels: Vec<OkxWsEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OkxRestEntry {
    pub id: String,
    pub visibility: String,
    pub method: String,
    pub path_or_doc: String,
    pub source_url: String,
    pub auth: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OkxWsEntry {
    pub id: String,
    pub visibility: String,
    pub channel_or_doc: String,
    pub source_url: String,
    pub auth: String,
}

pub fn load_okx_catalog_from_repo_root(repo_root: &Path) -> Result<OkxCatalog, UcelError> {
    let path = repo_root
        .join("docs")
        .join("exchanges")
        .join("okx")
        .join("catalog.json");
    load_okx_catalog_from_path(&path)
}

pub fn load_okx_catalog_from_path(path: &Path) -> Result<OkxCatalog, UcelError> {
    let raw = fs::read_to_string(path).map_err(|e| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("failed to read {}: {e}", path.display()),
        )
    })?;

    let catalog: OkxCatalog = serde_json::from_str(&raw).map_err(|e| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("failed to parse {}: {e}", path.display()),
        )
    })?;

    validate_okx_catalog(&catalog)?;
    Ok(catalog)
}

pub fn validate_okx_catalog(catalog: &OkxCatalog) -> Result<(), UcelError> {
    if catalog.exchange.trim() != "okx" {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("unexpected exchange value: {}", catalog.exchange),
        ));
    }

    let mut seen = HashSet::new();
    for row in &catalog.rest_endpoints {
        validate_okx_rest_row(row)?;
        if !seen.insert(row.id.clone()) {
            return Err(UcelError::new(
                ErrorCode::CatalogDuplicateId,
                format!("duplicate id found: {}", row.id),
            ));
        }
    }

    for row in &catalog.ws_channels {
        validate_okx_ws_row(row)?;
        if !seen.insert(row.id.clone()) {
            return Err(UcelError::new(
                ErrorCode::CatalogDuplicateId,
                format!("duplicate id found: {}", row.id),
            ));
        }
    }

    Ok(())
}

pub fn validate_okx_rest_row(row: &OkxRestEntry) -> Result<(), UcelError> {
    validate_required_field("id", &row.id)?;
    validate_required_field("visibility", &row.visibility)?;
    validate_required_field("method", &row.method)?;
    validate_required_field("path_or_doc", &row.path_or_doc)?;
    validate_required_field("source_url", &row.source_url)?;
    validate_required_field("auth", &row.auth)?;
    validate_visibility_auth_consistency(&row.id, &row.visibility, &row.auth)?;

    if !matches_http_method(&row.method) {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("invalid method for id={}: {}", row.id, row.method),
        ));
    }

    validate_url_like(&row.id, "source_url", &row.source_url)
}

pub fn validate_okx_ws_row(row: &OkxWsEntry) -> Result<(), UcelError> {
    validate_required_field("id", &row.id)?;
    validate_required_field("visibility", &row.visibility)?;
    validate_required_field("channel_or_doc", &row.channel_or_doc)?;
    validate_required_field("source_url", &row.source_url)?;
    validate_required_field("auth", &row.auth)?;
    validate_visibility_auth_consistency(&row.id, &row.visibility, &row.auth)?;
    validate_url_like(&row.id, "source_url", &row.source_url)
}

pub fn okx_requires_auth(visibility: &str) -> Result<bool, UcelError> {
    match visibility.trim().to_ascii_lowercase().as_str() {
        "public" => Ok(false),
        "private" => Ok(true),
        other => Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("invalid visibility={other}"),
        )),
    }
}

pub fn map_okx_op_name(id: &str) -> Result<OpName, UcelError> {
    if !id.starts_with("okx.") {
        return Err(UcelError::new(
            ErrorCode::NotSupported,
            format!("unsupported okx operation id={id}"),
        ));
    }

    if id.contains(".ws.") {
        if id.contains(".private") {
            Ok(OpName::SubscribeExecutionEvents)
        } else {
            Ok(OpName::SubscribeTicker)
        }
    } else if id.contains(".private") {
        Ok(OpName::PlaceOrder)
    } else {
        Ok(OpName::FetchStatus)
    }
}

pub fn okx_op_meta_from_id(id: &str, visibility: &str) -> Result<OpMeta, UcelError> {
    let op = map_okx_op_name(id)?;
    let requires_auth = okx_requires_auth(visibility)?;
    Ok(OpMeta { op, requires_auth })
}

fn validate_required_field(name: &str, value: &str) -> Result<(), UcelError> {
    if value.trim().is_empty() {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            format!("missing required field: {name}"),
        ));
    }
    Ok(())
}

fn validate_visibility_auth_consistency(
    id: &str,
    visibility: &str,
    auth: &str,
) -> Result<(), UcelError> {
    let requires_auth = okx_requires_auth(visibility)?;
    let auth_lower = auth.trim().to_ascii_lowercase();
    if requires_auth && auth_lower == "none" {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("visibility/private conflicts with auth=none for id={id}"),
        ));
    }
    if !requires_auth && auth_lower != "none" {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("visibility/public conflicts with auth={auth} for id={id}"),
        ));
    }
    Ok(())
}

fn validate_url_like(id: &str, field: &str, value: &str) -> Result<(), UcelError> {
    if !(value.starts_with("https://")
        || value.starts_with("http://")
        || value.starts_with("wss://")
        || value.starts_with("ws://"))
    {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("invalid {field} for id={id}: {value}"),
        ));
    }
    Ok(())
}

fn matches_http_method(method: &str) -> bool {
    method
        .split('/')
        .all(|part| matches!(part.trim(), "GET" | "POST" | "PUT" | "DELETE" | "PATCH"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_okx_catalog_and_counts_rows() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let catalog = load_okx_catalog_from_repo_root(&repo_root).unwrap();
        assert_eq!(catalog.rest_endpoints.len(), 4);
        assert_eq!(catalog.ws_channels.len(), 3);
    }

    #[test]
    fn requires_auth_is_derived_only_from_visibility() {
        let private_meta = okx_op_meta_from_id("okx.rest.private", "private").unwrap();
        assert!(private_meta.requires_auth);

        let public_meta = okx_op_meta_from_id("okx.ws.public", "public").unwrap();
        assert!(!public_meta.requires_auth);
    }

    #[test]
    fn rejects_visibility_auth_conflict() {
        let row = OkxRestEntry {
            id: "okx.rest.public".into(),
            visibility: "public".into(),
            method: "GET".into(),
            path_or_doc: "doc-ref".into(),
            source_url: "https://www.okx.com/docs-v5/en/".into(),
            auth: "apiKey/sign".into(),
        };
        let err = validate_okx_rest_row(&row).unwrap_err();
        assert_eq!(err.code, ErrorCode::CatalogInvalid);
    }
}
