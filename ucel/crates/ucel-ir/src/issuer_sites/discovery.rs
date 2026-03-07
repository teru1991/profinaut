use super::access::{ensure_budget, IssuerSitePolitenessPolicy};
use super::errors::{IssuerSiteError, IssuerSiteErrorCode};
use super::profile::{IssuerSiteAttachmentRule, IssuerSiteFeedDescriptor, IssuerSiteProfile, IssuerSiteSectionKind, IssuerSiteSeed, IssuerSiteSelectorRule};
use serde::Deserialize;
use ucel_core::{IrIssuerKey, IrMarket};

#[derive(Debug, Deserialize)]
struct SeedRow {
    source_id: String,
    market: String,
    issuer_canonical_id: String,
    seed_url: String,
    ir_paths: Vec<String>,
    feed_endpoints: Vec<String>,
}

fn seed_rows() -> Result<Vec<SeedRow>, IssuerSiteError> {
    serde_json::from_str(include_str!("../../../../../ucel/fixtures/ir_issuer_sites/seeds.json"))
        .map_err(|e| IssuerSiteError::new(IssuerSiteErrorCode::ParseFailed, e.to_string()))
}

pub fn discovery_from_seed(
    seed: &IssuerSiteSeed,
    policy: IssuerSitePolitenessPolicy,
) -> Result<Vec<IssuerSiteProfile>, IssuerSiteError> {
    ensure_budget(1, 1, policy)?;
    let rows = seed_rows()?;
    let row = rows
        .into_iter()
        .find(|r| r.source_id == seed.source_id && r.seed_url == seed.seed_url)
        .ok_or_else(|| IssuerSiteError::new(IssuerSiteErrorCode::IssuerSiteNotFound, "seed not found"))?;
    let market = if row.market == "jp" { IrMarket::Jp } else { IrMarket::Us };
    let selectors = vec![
        IssuerSiteSelectorRule { section: IssuerSiteSectionKind::IrTop, css: "main a[href]".into() },
        IssuerSiteSelectorRule { section: IssuerSiteSectionKind::NewsArchive, css: "a.news, a.press".into() },
        IssuerSiteSelectorRule { section: IssuerSiteSectionKind::PresentationLibrary, css: "a.presentation, a.library".into() },
    ];
    let feeds = row
        .feed_endpoints
        .into_iter()
        .map(|endpoint| IssuerSiteFeedDescriptor { endpoint, format: "rss".into() })
        .collect::<Vec<_>>();
    Ok(vec![IssuerSiteProfile {
        source_id: row.source_id,
        market,
        issuer_key: IrIssuerKey { market, canonical_id: row.issuer_canonical_id },
        root_url: row.seed_url,
        ir_index_candidates: row.ir_paths,
        feeds,
        selectors,
        attachment_rule: IssuerSiteAttachmentRule {
            extensions: vec!["pdf".into(), "html".into(), "zip".into(), "xbrl".into(), "xml".into()],
            allowed_content_types: vec!["text/html".into(), "application/pdf".into(), "application/xml".into(), "application/zip".into(), "application/rss+xml".into()],
        },
        language_hints: vec!["ja".into(), "en".into()],
        max_depth: policy.crawl_depth_cap,
        page_budget: policy.page_budget,
    }])
}

pub fn seeded_discovery(
    source_id: &str,
    market: IrMarket,
    issuer_canonical_id: &str,
) -> Result<Vec<IssuerSiteProfile>, IssuerSiteError> {
    let rows = seed_rows()?;
    let market_str = match market { IrMarket::Jp => "jp", IrMarket::Us => "us" };
    let row = rows
        .into_iter()
        .find(|r| r.source_id == source_id && r.market == market_str && r.issuer_canonical_id == issuer_canonical_id)
        .ok_or_else(|| IssuerSiteError::new(IssuerSiteErrorCode::IssuerSiteNotFound, "inventory seed missing"))?;

    discovery_from_seed(
        &IssuerSiteSeed {
            source_id: source_id.to_string(),
            issuer_key: IrIssuerKey { market, canonical_id: issuer_canonical_id.to_string() },
            seed_url: row.seed_url,
            provenance: "inventory_seed".into(),
        },
        IssuerSitePolitenessPolicy::default(),
    )
}
