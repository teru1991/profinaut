use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use ucel_core::{
    AuthMaterial, AuthRequestMeta, IdempotencyKey, NonceScope, SecretRef, SignContext, UcelError,
};
use ucel_transport::auth::{
    AuthSigner, IdempotencyProvider, NonceProvider, SecretResolver, SignedAuthParts, TimeProvider,
};

#[derive(Default)]
pub struct StaticSecretResolver {
    pub material_by_key: HashMap<String, AuthMaterial>,
}

impl SecretResolver for StaticSecretResolver {
    fn resolve(
        &self,
        secret_ref: &SecretRef,
        _meta: &AuthRequestMeta,
    ) -> Result<AuthMaterial, UcelError> {
        let key = secret_ref.key_id.as_ref().ok_or_else(|| {
            ucel_core::UcelError::new(
                ucel_core::ErrorCode::MissingAuth,
                "missing key_id in secret ref",
            )
        })?;
        self.material_by_key.get(key).cloned().ok_or_else(|| {
            ucel_core::UcelError::new(ucel_core::ErrorCode::MissingAuth, "secret not found")
        })
    }
}

pub struct FixedTimeProvider {
    pub now_ms_value: i64,
}

impl TimeProvider for FixedTimeProvider {
    fn now_ms(&self) -> i64 {
        self.now_ms_value
    }
}

#[derive(Default)]
pub struct MonotonicNonceProvider {
    counters: Mutex<HashMap<NonceScope, u64>>,
}

impl NonceProvider for MonotonicNonceProvider {
    fn next_nonce(&self, scope: &NonceScope) -> Result<u64, UcelError> {
        let mut guard = self.counters.lock().expect("nonce mutex poisoned");
        let next = guard.get(scope).copied().unwrap_or(0) + 1;
        guard.insert(scope.clone(), next);
        Ok(next)
    }
}

pub struct SequenceIdempotencyProvider {
    seq: AtomicU64,
}

impl Default for SequenceIdempotencyProvider {
    fn default() -> Self {
        Self {
            seq: AtomicU64::new(1),
        }
    }
}

impl IdempotencyProvider for SequenceIdempotencyProvider {
    fn next_idempotency_key(&self, meta: &AuthRequestMeta) -> Result<IdempotencyKey, UcelError> {
        let n = self.seq.fetch_add(1, Ordering::Relaxed);
        Ok(IdempotencyKey {
            raw: format!("{}:{}:{}", meta.venue, meta.request_name, n),
        })
    }
}

#[derive(Default)]
pub struct EchoSigner;

impl AuthSigner for EchoSigner {
    fn sign(
        &self,
        ctx: &SignContext,
        _material: &AuthMaterial,
    ) -> Result<SignedAuthParts, UcelError> {
        let mut headers = BTreeMap::new();
        headers.insert("x-auth-preview".into(), ctx.redacted_preview());
        if let Some(nonce) = ctx.nonce {
            headers.insert("x-nonce".into(), nonce.to_string());
        }
        if let Some(id) = &ctx.idempotency_key {
            headers.insert("x-idempotency-key".into(), id.clone());
        }
        Ok(SignedAuthParts {
            headers,
            ..Default::default()
        })
    }
}
