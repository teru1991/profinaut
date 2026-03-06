use std::collections::BTreeMap;

use ucel_core::{
    validate_auth_material, AuthMaterial, AuthRequestMeta, ErrorCode, IdempotencyKey, NonceScope,
    SecretRef, ServerTimeOffset, SignContext, UcelError,
};

pub trait SecretResolver {
    fn resolve(
        &self,
        secret_ref: &SecretRef,
        meta: &AuthRequestMeta,
    ) -> Result<AuthMaterial, UcelError>;
}

pub trait TimeProvider {
    fn now_ms(&self) -> i64;
}

pub trait NonceProvider {
    fn next_nonce(&self, scope: &NonceScope) -> Result<u64, UcelError>;
}

pub trait IdempotencyProvider {
    fn next_idempotency_key(&self, meta: &AuthRequestMeta) -> Result<IdempotencyKey, UcelError>;
}

pub trait AuthSigner {
    fn sign(
        &self,
        ctx: &SignContext,
        material: &AuthMaterial,
    ) -> Result<SignedAuthParts, UcelError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SignedAuthParts {
    pub headers: BTreeMap<String, String>,
    pub query: BTreeMap<String, String>,
    pub body_fields: BTreeMap<String, String>,
    pub ws_login_fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthRuntime {
    pub max_clock_skew_ms: i64,
}

impl Default for AuthRuntime {
    fn default() -> Self {
        Self {
            max_clock_skew_ms: 60_000,
        }
    }
}

impl AuthRuntime {
    pub fn prepare_sign_context(
        &self,
        meta: &AuthRequestMeta,
        method: &str,
        path: &str,
        query_canonical: &str,
        body_canonical: &str,
        now_ms: i64,
        nonce: Option<u64>,
        idempotency_key: Option<String>,
    ) -> Result<SignContext, UcelError> {
        meta.validate()?;
        Ok(SignContext {
            method: method.to_string(),
            path: path.to_string(),
            query_canonical: query_canonical.to_string(),
            body_canonical: body_canonical.to_string(),
            timestamp_ms: now_ms,
            nonce,
            idempotency_key,
            auth_mode: meta.auth_mode,
            key_id: meta.key_id.clone(),
        })
    }

    pub fn enforce_auth_material_presence(
        &self,
        meta: &AuthRequestMeta,
        material: &AuthMaterial,
    ) -> Result<(), UcelError> {
        validate_auth_material(meta, material)
    }

    pub fn resolve_and_sign<R, N, I, S>(
        &self,
        resolver: &R,
        nonce_provider: &N,
        idempotency_provider: &I,
        signer: &S,
        secret_ref: &SecretRef,
        meta: &AuthRequestMeta,
        method: &str,
        path: &str,
        query_canonical: &str,
        body_canonical: &str,
        now_ms: i64,
    ) -> Result<SignedAuthParts, UcelError>
    where
        R: SecretResolver,
        N: NonceProvider,
        I: IdempotencyProvider,
        S: AuthSigner,
    {
        meta.validate()?;
        let material = resolver.resolve(secret_ref, meta)?;
        self.enforce_auth_material_presence(meta, &material)?;

        let nonce = if meta.requires_material() {
            Some(nonce_provider.next_nonce(&NonceScope {
                venue: meta.venue.clone(),
                key_id: meta.key_id.clone().unwrap_or_default(),
                surface: meta.surface,
            })?)
        } else {
            None
        };

        let idempotency_key = if meta.requires_material() {
            Some(idempotency_provider.next_idempotency_key(meta)?.raw)
        } else {
            None
        };

        let ctx = self.prepare_sign_context(
            meta,
            method,
            path,
            query_canonical,
            body_canonical,
            now_ms,
            nonce,
            idempotency_key,
        )?;

        signer.sign(&ctx, &material)
    }

    pub fn update_server_time_offset(
        &self,
        current: Option<ServerTimeOffset>,
        server_ms: i64,
        local_ms: i64,
        observed_at_ms: i64,
    ) -> Result<ServerTimeOffset, UcelError> {
        let offset_ms = server_ms.saturating_sub(local_ms);
        if offset_ms.abs() > self.max_clock_skew_ms {
            return Err(UcelError::new(
                ErrorCode::InvalidOrder,
                format!(
                    "clock skew exceeds threshold: abs(offset_ms)={} max={}",
                    offset_ms.abs(),
                    self.max_clock_skew_ms
                ),
            ));
        }

        if let Some(previous) = current {
            if observed_at_ms < previous.observed_at_ms {
                return Err(UcelError::new(
                    ErrorCode::Desync,
                    "server time observation moved backwards",
                ));
            }
        }

        Ok(ServerTimeOffset {
            offset_ms,
            observed_at_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucel_core::{AuthMode, AuthSurface};

    struct StubResolver;
    impl SecretResolver for StubResolver {
        fn resolve(
            &self,
            _secret_ref: &SecretRef,
            _meta: &AuthRequestMeta,
        ) -> Result<AuthMaterial, UcelError> {
            Ok(AuthMaterial {
                api_key: Some("k".into()),
                api_secret: Some("s".into()),
                ..Default::default()
            })
        }
    }

    struct StubNonce;
    impl NonceProvider for StubNonce {
        fn next_nonce(&self, _scope: &NonceScope) -> Result<u64, UcelError> {
            Ok(7)
        }
    }

    struct StubIdem;
    impl IdempotencyProvider for StubIdem {
        fn next_idempotency_key(
            &self,
            _meta: &AuthRequestMeta,
        ) -> Result<IdempotencyKey, UcelError> {
            Ok(IdempotencyKey { raw: "idem".into() })
        }
    }

    struct StubSigner;
    impl AuthSigner for StubSigner {
        fn sign(
            &self,
            ctx: &SignContext,
            _material: &AuthMaterial,
        ) -> Result<SignedAuthParts, UcelError> {
            let mut headers = BTreeMap::new();
            headers.insert("x-nonce".into(), ctx.nonce.unwrap_or_default().to_string());
            Ok(SignedAuthParts {
                headers,
                ..Default::default()
            })
        }
    }

    #[test]
    fn resolve_and_sign_happy_path() {
        let runtime = AuthRuntime::default();
        let meta = AuthRequestMeta {
            venue: "bitbank".into(),
            surface: AuthSurface::PrivateRest,
            auth_mode: AuthMode::HmacHeader,
            requires_auth: true,
            request_name: "place_order".into(),
            key_id: Some("k1".into()),
        };
        let parts = runtime
            .resolve_and_sign(
                &StubResolver,
                &StubNonce,
                &StubIdem,
                &StubSigner,
                &SecretRef {
                    key_id: Some("k1".into()),
                    alias: None,
                },
                &meta,
                "POST",
                "/private",
                "",
                "{}",
                10,
            )
            .unwrap();
        assert_eq!(parts.headers.get("x-nonce"), Some(&"7".to_string()));
    }
}
