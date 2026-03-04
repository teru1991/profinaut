use crate::provider::{Contribution, ContributionKind, DiagnosticsProvider};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct RegistryLimits {
    pub max_providers: usize,
    pub max_contributions_total: usize,
    pub max_path_len: usize,
}

impl Default for RegistryLimits {
    fn default() -> Self {
        Self {
            max_providers: 128,
            max_contributions_total: 2048,
            max_path_len: 256,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("duplicate provider_id: {0}")]
    DuplicateProvider(String),
    #[error("too many providers: {0}")]
    TooManyProviders(usize),
    #[error("invalid contribution path: {0}")]
    InvalidPath(String),
    #[error("path too long: {0}")]
    PathTooLong(String),
    #[error("too many contributions: {0}")]
    TooManyContributions(usize),
}

pub struct DiagnosticsRegistry {
    limits: RegistryLimits,
    providers: BTreeMap<String, Box<dyn DiagnosticsProvider>>,
}

impl DiagnosticsRegistry {
    pub fn new(limits: RegistryLimits) -> Self {
        Self {
            limits,
            providers: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, p: Box<dyn DiagnosticsProvider>) -> Result<(), RegistryError> {
        if self.providers.len() >= self.limits.max_providers {
            return Err(RegistryError::TooManyProviders(self.providers.len() + 1));
        }
        let id = p.meta().provider_id;
        if self.providers.contains_key(&id) {
            return Err(RegistryError::DuplicateProvider(id));
        }
        self.providers.insert(id, p);
        Ok(())
    }

    pub fn provider_ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Collect contributions from all providers, enforce:
    /// - provider_id must match contribution.provider_id
    /// - path safety & normalization constraints (no absolute, no '..')
    /// - deterministic sort
    /// - global count limit
    pub fn collect(&self, req: &crate::provider::DiagnosticsRequest) -> Result<Vec<Contribution>, RegistryError> {
        let mut out: Vec<Contribution> = Vec::new();
        let mut seen_paths: BTreeSet<(String, String, u8)> = BTreeSet::new();

        for (pid, p) in self.providers.iter() {
            let mut cs = p.contributions(req);
            // Enforce provider_id match and path safety early
            for c in cs.iter_mut() {
                c.provider_id = pid.clone(); // enforce canonical
                validate_path(&c.path, self.limits.max_path_len)?;
                // Reject binary by default? Not here (policy). Registry just keeps kind.
                let key = {
                    let k = match c.kind {
                        ContributionKind::Json => 0,
                        ContributionKind::Text => 1,
                        ContributionKind::Meta => 2,
                        ContributionKind::Binary => 3,
                    };
                    (pid.clone(), c.path.clone(), k)
                };
                seen_paths.insert(key);
            }
            out.append(&mut cs);
        }

        if out.len() > self.limits.max_contributions_total {
            return Err(RegistryError::TooManyContributions(out.len()));
        }

        out.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        Ok(out)
    }
}

fn validate_path(p: &str, max_len: usize) -> Result<(), RegistryError> {
    if p.is_empty() {
        return Err(RegistryError::InvalidPath(p.to_string()));
    }
    if p.len() > max_len {
        return Err(RegistryError::PathTooLong(p.to_string()));
    }
    // deny absolute
    if p.starts_with('/') || p.starts_with('\\') {
        return Err(RegistryError::InvalidPath(p.to_string()));
    }
    // deny traversal
    if p.contains("..") {
        return Err(RegistryError::InvalidPath(p.to_string()));
    }
    // very small allowlist of chars (future: tighten if needed)
    if !p.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/')) {
        return Err(RegistryError::InvalidPath(p.to_string()));
    }
    Ok(())
}
