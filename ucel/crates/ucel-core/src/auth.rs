use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{ErrorCode, UcelError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    None,
    HmacHeader,
    HmacQuery,
    JwtBearer,
    SessionToken,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthSurface {
    PrivateRest,
    PrivateWs,
    Execution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SecretRef {
    pub key_id: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthMaterial {
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub passphrase: Option<String>,
    pub session_token: Option<String>,
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthRequestMeta {
    pub venue: String,
    pub surface: AuthSurface,
    pub auth_mode: AuthMode,
    pub requires_auth: bool,
    pub request_name: String,
    pub key_id: Option<String>,
}

impl AuthRequestMeta {
    pub fn requires_material(&self) -> bool {
        self.requires_auth && !matches!(self.auth_mode, AuthMode::None)
    }

    pub fn validate(&self) -> Result<(), UcelError> {
        if self.requires_auth && self.key_id.is_none() {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "auth request requires key_id",
            ));
        }
        if self.requires_auth && matches!(self.auth_mode, AuthMode::None) {
            return Err(UcelError::new(
                ErrorCode::MissingAuth,
                "auth request requires non-none auth_mode",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerTimeOffset {
    pub offset_ms: i64,
    pub observed_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NonceScope {
    pub venue: String,
    pub key_id: String,
    pub surface: AuthSurface,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyKey {
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignContext {
    pub method: String,
    pub path: String,
    pub query_canonical: String,
    pub body_canonical: String,
    pub timestamp_ms: i64,
    pub nonce: Option<u64>,
    pub idempotency_key: Option<String>,
    pub auth_mode: AuthMode,
    pub key_id: Option<String>,
}

impl SignContext {
    pub fn redacted_preview(&self) -> String {
        let key = if self.key_id.is_some() {
            "***redacted***"
        } else {
            "none"
        };
        format!(
            "method={} path={} query={} body={} ts={} nonce={} idempotency={} mode={:?} key_id={}",
            self.method,
            self.path,
            self.query_canonical,
            self.body_canonical,
            self.timestamp_ms,
            self.nonce
                .map(|_| "***redacted***".to_string())
                .unwrap_or_else(|| "none".to_string()),
            self.idempotency_key
                .as_ref()
                .map(|_| "***redacted***".to_string())
                .unwrap_or_else(|| "none".to_string()),
            self.auth_mode,
            key
        )
    }
}

pub fn validate_auth_material(
    meta: &AuthRequestMeta,
    material: &AuthMaterial,
) -> Result<(), UcelError> {
    if !meta.requires_material() {
        return Ok(());
    }

    match meta.auth_mode {
        AuthMode::HmacHeader | AuthMode::HmacQuery => {
            if material.api_key.is_none() || material.api_secret.is_none() {
                return Err(UcelError::new(
                    ErrorCode::MissingAuth,
                    "hmac auth requires api_key and api_secret",
                ));
            }
        }
        AuthMode::JwtBearer => {
            if material.api_secret.is_none() {
                return Err(UcelError::new(
                    ErrorCode::MissingAuth,
                    "jwt auth requires api_secret",
                ));
            }
        }
        AuthMode::SessionToken => {
            if material.session_token.is_none() {
                return Err(UcelError::new(
                    ErrorCode::MissingAuth,
                    "session token auth requires session_token",
                ));
            }
        }
        AuthMode::Custom | AuthMode::None => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_material_matches_mode_and_flag() {
        let m = AuthRequestMeta {
            venue: "v".into(),
            surface: AuthSurface::PrivateRest,
            auth_mode: AuthMode::HmacHeader,
            requires_auth: true,
            request_name: "place_order".into(),
            key_id: Some("k".into()),
        };
        assert!(m.requires_material());

        let n = AuthRequestMeta {
            auth_mode: AuthMode::None,
            ..m
        };
        assert!(!n.requires_material());
    }

    #[test]
    fn redacted_preview_masks_sensitive_values() {
        let c = SignContext {
            method: "POST".into(),
            path: "/private".into(),
            query_canonical: "a=1".into(),
            body_canonical: "{}".into(),
            timestamp_ms: 1,
            nonce: Some(100),
            idempotency_key: Some("idem-1".into()),
            auth_mode: AuthMode::HmacHeader,
            key_id: Some("kid".into()),
        };
        let p = c.redacted_preview();
        assert!(!p.contains("idem-1"));
        assert!(!p.contains("kid"));
        assert!(p.contains("***redacted***"));
    }

    #[test]
    fn serde_round_trip_auth_surface_and_mode() {
        let mode = AuthMode::JwtBearer;
        let mode_json = serde_json::to_string(&mode).unwrap();
        assert_eq!(mode_json, "\"jwt_bearer\"");
        assert_eq!(serde_json::from_str::<AuthMode>(&mode_json).unwrap(), mode);

        let surface = AuthSurface::Execution;
        let surface_json = serde_json::to_string(&surface).unwrap();
        assert_eq!(surface_json, "\"execution\"");
        assert_eq!(
            serde_json::from_str::<AuthSurface>(&surface_json).unwrap(),
            surface
        );
    }
}
