use std::collections::BTreeSet;
use std::path::PathBuf;

use ucel_registry::hub::{registry, ExchangeId, Hub};
use ucel_registry::{default_capabilities_for_exchange, exchange_ids};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root")
        .to_path_buf()
}

fn workspace_cex_members() -> BTreeSet<String> {
    let root = repo_root();
    let cargo = std::fs::read_to_string(root.join("ucel/Cargo.toml")).expect("cargo toml");
    cargo
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('"') && line.contains("crates/ucel-cex-") {
                Some(
                    line.trim_matches(',')
                        .trim_matches('"')
                        .trim_start_matches("crates/")
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect()
}

fn expected_logical_ids_from_members() -> BTreeSet<String> {
    workspace_cex_members()
        .into_iter()
        .map(|name| name.trim_start_matches("ucel-cex-").to_string())
        .collect()
}

#[test]
fn workspace_cex_members_are_reachable_from_exchange_id_all() {
    let all_ids: BTreeSet<String> = ExchangeId::all()
        .iter()
        .map(|id| id.as_str().to_string())
        .collect();
    let expected = expected_logical_ids_from_members();
    for logical in expected {
        assert!(
            all_ids.contains(&logical),
            "missing ExchangeId for workspace member logical id={logical}"
        );
    }
}

#[test]
fn all_registrations_have_catalog_and_capabilities() {
    for id in exchange_ids() {
        let catalog = registry::catalog_for(id).expect("catalog parse");
        let caps = default_capabilities_for_exchange(id).expect("capabilities");
        assert_eq!(caps.name, id.as_str());
        let registration = registry::find_registration(id.as_str()).expect("registration");
        if catalog.rest_endpoints.is_empty() && catalog.ws_channels.is_empty() {
            assert!(
                registration.notes.contains("catalog-empty"),
                "catalog unexpectedly empty without explicit note for {}",
                id.as_str()
            );
        }
    }
}

#[test]
fn alias_resolution_and_hub_surface_are_stable() {
    assert_eq!(
        "binance-spot".parse::<ExchangeId>().unwrap(),
        ExchangeId::Binance
    );
    assert_eq!("huobi".parse::<ExchangeId>().unwrap(), ExchangeId::Htx);

    let hub = Hub::default();
    assert!(hub.exchange_exists("binance-spot"));
    assert!(hub.exchange_exists("htx"));
    assert!(!hub.exchange_exists("unknown-venue"));
}
