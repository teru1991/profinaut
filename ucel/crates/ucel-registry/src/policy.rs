use crate::CatalogEntry;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use ucel_core::policy::enforce_access;
use ucel_core::{
    AccessSurface, ErrorCode, ResidencyClass, UcelError, VenueAccessPolicy, VenueAccessScope,
};

const JP_POLICY_PATH: &str = "coverage/coverage_v2/jurisdictions/jp_resident_access.json";

pub fn load_jp_resident_policy(repo_root: &Path) -> Result<VenueAccessPolicy, UcelError> {
    let path = repo_root.join(JP_POLICY_PATH);
    let bytes = std::fs::read(&path).map_err(|e| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("read {}: {e}", path.display()),
        )
    })?;
    let policy: VenueAccessPolicy = serde_json::from_slice(&bytes).map_err(|e| {
        UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("parse {}: {e}", path.display()),
        )
    })?;
    if policy.residency != ResidencyClass::JpResident {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            format!("unexpected residency in {}", path.display()),
        ));
    }
    Ok(normalize(policy))
}

pub fn default_jp_resident_policy() -> Result<&'static VenueAccessPolicy, UcelError> {
    static POLICY: OnceLock<Result<VenueAccessPolicy, UcelError>> = OnceLock::new();
    POLICY
        .get_or_init(|| {
            let root = detect_ucel_repo_root()?;
            load_jp_resident_policy(&root)
        })
        .as_ref()
        .map_err(|e| e.clone())
}

pub fn scope_for_venue(venue: &str) -> Result<VenueAccessScope, UcelError> {
    Ok(default_jp_resident_policy()?.scope_for_venue(venue))
}

pub fn enforce_private_surface_allowed(venue: &str) -> Result<(), UcelError> {
    let policy = default_jp_resident_policy()?;
    enforce_access(policy, venue, AccessSurface::PrivateWs)
}

pub fn enforce_surface_for_catalog_entry(
    venue: &str,
    entry: &CatalogEntry,
) -> Result<(), UcelError> {
    let policy = default_jp_resident_policy()?;
    let surface = infer_surface(entry);
    enforce_access(policy, venue, surface)
}

fn infer_surface(entry: &CatalogEntry) -> AccessSurface {
    let id = entry.id.to_ascii_lowercase();
    let op = entry
        .operation
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let private_hint = id.contains(".private.")
        || id.contains("private")
        || op.contains("place_order")
        || op.contains("cancel_order")
        || op.contains("amend_order")
        || op.contains("execution")
        || id.contains("place_order")
        || id.contains("cancel_order")
        || id.contains("amend_order")
        || id.contains("execution");
    if private_hint {
        return if is_ws_entry(entry) {
            AccessSurface::PrivateWs
        } else if is_execution_entry(entry) {
            AccessSurface::Execution
        } else {
            AccessSurface::PrivateRest
        };
    }

    if entry_visibility_private(entry) || entry.requires_auth.unwrap_or(false) {
        if is_ws_entry(entry) {
            AccessSurface::PrivateWs
        } else {
            AccessSurface::PrivateRest
        }
    } else if is_ws_entry(entry) {
        AccessSurface::PublicWs
    } else {
        AccessSurface::PublicRest
    }
}

fn is_execution_entry(entry: &CatalogEntry) -> bool {
    let id = entry.id.to_ascii_lowercase();
    let op = entry
        .operation
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    id.contains("place_order")
        || id.contains("cancel_order")
        || id.contains("amend_order")
        || id.contains("execution")
        || op.contains("place_order")
        || op.contains("cancel_order")
        || op.contains("amend_order")
        || op.contains("execution")
}

fn is_ws_entry(entry: &CatalogEntry) -> bool {
    entry.ws_url.is_some()
        || entry.ws.is_some()
        || entry.channel.is_some()
        || entry.id.contains(".ws.")
}

fn entry_visibility_private(entry: &CatalogEntry) -> bool {
    matches!(
        entry.visibility.to_ascii_lowercase().as_str(),
        "private" | "public/private"
    )
}

fn detect_ucel_repo_root() -> Result<PathBuf, UcelError> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| UcelError::new(ErrorCode::Internal, "failed to detect ucel repo root"))?
        .to_path_buf();
    Ok(root)
}

fn normalize(mut policy: VenueAccessPolicy) -> VenueAccessPolicy {
    for entry in &mut policy.entries {
        entry.venue = entry.venue.to_ascii_lowercase();
    }
    policy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CatalogAuth;

    fn entry(id: &str, visibility: &str, requires_auth: Option<bool>, ws: bool) -> CatalogEntry {
        CatalogEntry {
            id: id.into(),
            visibility: visibility.into(),
            requires_auth,
            channel: if ws { Some("ch".into()) } else { None },
            operation: Some(id.into()),
            method: if ws { None } else { Some("GET".into()) },
            base_url: if ws {
                None
            } else {
                Some("https://api.x".into())
            },
            path: if ws { None } else { Some("/p".into()) },
            ws_url: if ws {
                Some("wss://api.x/ws".into())
            } else {
                None
            },
            ws: None,
            auth: CatalogAuth::default(),
        }
    }

    #[test]
    fn bitbank_private_allowed() {
        let e = entry("spot.private.rest.balance", "private", Some(true), false);
        assert!(enforce_surface_for_catalog_entry("bitbank", &e).is_ok());
    }

    #[test]
    fn sbivc_private_denied() {
        let e = entry("spot.private.rest.balance", "private", Some(true), false);
        let err = enforce_surface_for_catalog_entry("sbivc", &e).unwrap_err();
        assert_eq!(err.code, ErrorCode::PermissionDenied);
    }

    #[test]
    fn unspecified_venue_defaults_to_public_only() {
        let e = entry("spot.private.ws.orders", "private", Some(true), true);
        let err = enforce_surface_for_catalog_entry("bybit", &e).unwrap_err();
        assert_eq!(err.code, ErrorCode::PermissionDenied);
    }

    #[test]
    fn public_surfaces_allowed() {
        let e = entry("spot.public.rest.ticker", "public", Some(false), false);
        assert!(enforce_surface_for_catalog_entry("sbivc", &e).is_ok());
        let ews = entry("spot.public.ws.trades", "public", Some(false), true);
        assert!(enforce_surface_for_catalog_entry("bybit", &ews).is_ok());
    }
}
