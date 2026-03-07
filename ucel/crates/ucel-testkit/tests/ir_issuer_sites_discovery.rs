use ucel_core::{IrIssuerKey, IrMarket};
use ucel_ir::issuer_sites::{discovery::discovery_from_seed, discovery::seeded_discovery, IssuerSitePolitenessPolicy, profile::IssuerSiteSeed};

#[test]
fn ir_issuer_sites_discovery() {
    let from_inventory = seeded_discovery("jp_issuer_ir_html_public", IrMarket::Jp, "JP-ACME-1111")
        .expect("seeded discovery");
    assert!(!from_inventory.is_empty());

    let profile = discovery_from_seed(
        &IssuerSiteSeed {
            source_id: "us_issuer_ir_html_public".into(),
            issuer_key: IrIssuerKey { market: IrMarket::Us, canonical_id: "US-ACME-2222".into() },
            seed_url: "https://investors.example.com".into(),
            provenance: "official_metadata".into(),
        },
        IssuerSitePolitenessPolicy::default(),
    )
    .expect("discovery from official metadata seed");
    assert!(profile[0].selectors.iter().any(|s| s.css.contains("a[href]")));

    let err = discovery_from_seed(
        &IssuerSiteSeed {
            source_id: "us_issuer_ir_html_public".into(),
            issuer_key: IrIssuerKey { market: IrMarket::Us, canonical_id: "US-ACME-2222".into() },
            seed_url: "https://investors.example.com".into(),
            provenance: "official_metadata".into(),
        },
        IssuerSitePolitenessPolicy { page_budget: 0, ..IssuerSitePolitenessPolicy::default() },
    )
    .expect_err("budget exceeded should fail");
    assert!(err.to_string().contains("DiscoveryBudgetExceeded"));
}
