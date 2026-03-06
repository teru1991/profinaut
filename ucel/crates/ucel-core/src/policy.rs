use serde::{Deserialize, Serialize};

use crate::{ErrorCode, UcelError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidencyClass {
    JpResident,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VenueAccessScope {
    Blocked,
    PublicOnly,
    PublicPrivate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessSurface {
    PublicRest,
    PublicWs,
    PrivateRest,
    PrivateWs,
    Execution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenueAccessEntry {
    pub venue: String,
    pub scope: VenueAccessScope,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenueAccessPolicy {
    pub policy_id: String,
    pub residency: ResidencyClass,
    pub default_scope: VenueAccessScope,
    #[serde(default)]
    pub entries: Vec<VenueAccessEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenueAccessCapabilities {
    pub scope: VenueAccessScope,
    pub private_rest: bool,
    pub private_ws: bool,
    pub execution: bool,
}

impl VenueAccessScope {
    pub fn allows(self, surface: AccessSurface) -> bool {
        match self {
            VenueAccessScope::Blocked => false,
            VenueAccessScope::PublicOnly => {
                matches!(surface, AccessSurface::PublicRest | AccessSurface::PublicWs)
            }
            VenueAccessScope::PublicPrivate => true,
        }
    }
}

impl VenueAccessPolicy {
    pub fn scope_for_venue(&self, venue: &str) -> VenueAccessScope {
        let venue = venue.to_ascii_lowercase();
        self.entries
            .iter()
            .find(|entry| entry.venue.eq_ignore_ascii_case(&venue))
            .map(|entry| entry.scope)
            .unwrap_or(self.default_scope)
    }
}

pub fn enforce_access(
    policy: &VenueAccessPolicy,
    venue: &str,
    surface: AccessSurface,
) -> Result<(), UcelError> {
    let scope = policy.scope_for_venue(venue);
    if scope.allows(surface) {
        return Ok(());
    }

    Err(UcelError::new(
        ErrorCode::PermissionDenied,
        format!(
            "venue_access denied venue={venue} surface={surface:?} policy_id={} scope={scope:?}",
            policy.policy_id
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_policy() -> VenueAccessPolicy {
        VenueAccessPolicy {
            policy_id: "jp-resident-v1".into(),
            residency: ResidencyClass::JpResident,
            default_scope: VenueAccessScope::PublicOnly,
            entries: vec![VenueAccessEntry {
                venue: "bitbank".into(),
                scope: VenueAccessScope::PublicPrivate,
                reason: "domestic policy".into(),
            }],
        }
    }

    #[test]
    fn public_private_allows_all_surfaces() {
        let s = VenueAccessScope::PublicPrivate;
        assert!(s.allows(AccessSurface::PublicRest));
        assert!(s.allows(AccessSurface::PublicWs));
        assert!(s.allows(AccessSurface::PrivateRest));
        assert!(s.allows(AccessSurface::PrivateWs));
        assert!(s.allows(AccessSurface::Execution));
    }

    #[test]
    fn public_only_blocks_private_and_execution() {
        let s = VenueAccessScope::PublicOnly;
        assert!(s.allows(AccessSurface::PublicRest));
        assert!(s.allows(AccessSurface::PublicWs));
        assert!(!s.allows(AccessSurface::PrivateRest));
        assert!(!s.allows(AccessSurface::PrivateWs));
        assert!(!s.allows(AccessSurface::Execution));
    }

    #[test]
    fn blocked_rejects_all_surfaces() {
        let s = VenueAccessScope::Blocked;
        assert!(!s.allows(AccessSurface::PublicRest));
        assert!(!s.allows(AccessSurface::PublicWs));
        assert!(!s.allows(AccessSurface::PrivateRest));
        assert!(!s.allows(AccessSurface::PrivateWs));
        assert!(!s.allows(AccessSurface::Execution));
    }

    #[test]
    fn unknown_venue_uses_default_scope() {
        let p = sample_policy();
        assert_eq!(p.scope_for_venue("bybit"), VenueAccessScope::PublicOnly);
    }
}
