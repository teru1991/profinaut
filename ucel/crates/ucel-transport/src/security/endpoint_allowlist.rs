use std::collections::HashSet;
use ucel_core::{ErrorCode, UcelError};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubdomainPolicy {
    /// Only exact host matches.
    Exact,
    /// Allow subdomains of allowlisted hosts (e.g. allow "api.example.com" if "example.com" allowlisted).
    AllowSubdomains,
}

#[derive(Debug, Clone)]
pub struct EndpointAllowlist {
    allowed_hosts: HashSet<String>,
    subdomain_policy: SubdomainPolicy,
}

impl EndpointAllowlist {
    pub fn new<I, S>(hosts: I, subdomain_policy: SubdomainPolicy) -> Result<Self, UcelError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut allowed_hosts = HashSet::new();
        for h in hosts {
            let hs = h.into().trim().to_ascii_lowercase();
            if hs.is_empty() {
                continue;
            }
            if hs.contains("://") || hs.contains('/') {
                return Err(UcelError::new(
                    ErrorCode::CatalogInvalid,
                    format!("invalid allowlist host entry: {hs}"),
                ));
            }
            allowed_hosts.insert(hs);
        }

        if allowed_hosts.is_empty() {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                "endpoint allowlist must not be empty",
            ));
        }

        Ok(Self {
            allowed_hosts,
            subdomain_policy,
        })
    }

    pub fn validate_https_wss(&self, raw: &str) -> Result<Url, UcelError> {
        let url = Url::parse(raw).map_err(|e| {
            UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("invalid endpoint url: {e}"),
            )
        })?;

        let scheme = url.scheme().to_ascii_lowercase();
        if scheme != "https" && scheme != "wss" {
            return Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("endpoint scheme must be https or wss (got: {scheme})"),
            ));
        }

        let host = url.host_str().ok_or_else(|| {
            UcelError::new(ErrorCode::CatalogInvalid, "endpoint url must have host")
        })?;
        let host = host.to_ascii_lowercase();

        if self.is_allowed_host(&host) {
            Ok(url)
        } else {
            Err(UcelError::new(
                ErrorCode::CatalogInvalid,
                format!("endpoint host is not allowlisted: {host}"),
            ))
        }
    }

    fn is_allowed_host(&self, host: &str) -> bool {
        if self.allowed_hosts.contains(host) {
            return true;
        }
        if self.subdomain_policy == SubdomainPolicy::AllowSubdomains {
            for allowed in &self.allowed_hosts {
                let suffix = format!(".{allowed}");
                if host.ends_with(&suffix) {
                    return true;
                }
            }
        }
        false
    }
}
