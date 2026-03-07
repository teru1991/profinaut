use crate::ir_inventory::load_ir_inventory;
use std::path::{Path, PathBuf};
use ucel_core::{
    IrAccessPolicyClass, IrArtifactKind, IrDocumentFamily, IrIssuerIdentityKind, IrSourceFamily,
};

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn parse_source_family(v: &str) -> IrSourceFamily {
    match v {
        "jp_statutory_disclosure" => IrSourceFamily::JpStatutoryDisclosure,
        "jp_timely_disclosure" => IrSourceFamily::JpTimelyDisclosure,
        "jp_issuer_ir_site" => IrSourceFamily::JpIssuerIrSite,
        "us_sec_disclosure" => IrSourceFamily::UsSecDisclosure,
        "us_issuer_ir_site" => IrSourceFamily::UsIssuerIrSite,
        _ => IrSourceFamily::Other(v.to_string()),
    }
}

pub fn parse_access_policy(v: &str) -> Option<IrAccessPolicyClass> {
    Some(match v {
        "free_public_noauth_allowed" => IrAccessPolicyClass::FreePublicNoAuthAllowed,
        "free_public_noauth_review_required" => IrAccessPolicyClass::FreePublicNoAuthReviewRequired,
        "excluded_paid_or_contract" => IrAccessPolicyClass::ExcludedPaidOrContract,
        "excluded_login_required" => IrAccessPolicyClass::ExcludedLoginRequired,
        "excluded_policy_blocked" => IrAccessPolicyClass::ExcludedPolicyBlocked,
        _ => return None,
    })
}

pub fn parse_identity_kind(v: &str) -> Option<IrIssuerIdentityKind> {
    Some(match v {
        "jp_edinet_code_like" => IrIssuerIdentityKind::JpEdinetCodeLike,
        "jp_local_code_like" => IrIssuerIdentityKind::JpLocalCodeLike,
        "jp_exchange_code_like" => IrIssuerIdentityKind::JpExchangeCodeLike,
        "us_cik_like" => IrIssuerIdentityKind::UsCikLike,
        "ticker_like" => IrIssuerIdentityKind::TickerLike,
        "exchange_ticker_like" => IrIssuerIdentityKind::ExchangeTickerLike,
        "issuer_site_slug_like" => IrIssuerIdentityKind::IssuerSiteSlugLike,
        "url_like" => IrIssuerIdentityKind::UrlLike,
        _ => return None,
    })
}

pub fn parse_document_family(v: &str) -> Option<IrDocumentFamily> {
    Some(match v {
        "statutory_annual" => IrDocumentFamily::StatutoryAnnual,
        "statutory_quarterly" => IrDocumentFamily::StatutoryQuarterly,
        "statutory_current" => IrDocumentFamily::StatutoryCurrent,
        "timely_disclosure" => IrDocumentFamily::TimelyDisclosure,
        "earnings_release" => IrDocumentFamily::EarningsRelease,
        "earnings_presentation" => IrDocumentFamily::EarningsPresentation,
        "transcript" => IrDocumentFamily::Transcript,
        "press_release" => IrDocumentFamily::PressRelease,
        "proxy" => IrDocumentFamily::Proxy,
        "integrated_report" => IrDocumentFamily::IntegratedReport,
        "sustainability_report" => IrDocumentFamily::SustainabilityReport,
        "fact_sheet" => IrDocumentFamily::FactSheet,
        "governance_report" => IrDocumentFamily::GovernanceReport,
        "misc_ir_document" => IrDocumentFamily::MiscIrDocument,
        _ => return None,
    })
}

pub fn parse_artifact_kind(v: &str) -> Option<IrArtifactKind> {
    Some(match v {
        "html" => IrArtifactKind::Html,
        "pdf" => IrArtifactKind::Pdf,
        "xbrl" => IrArtifactKind::Xbrl,
        "ixbrl" => IrArtifactKind::Ixbrl,
        "xml" => IrArtifactKind::Xml,
        "txt" => IrArtifactKind::Txt,
        "csv" => IrArtifactKind::Csv,
        "zip" => IrArtifactKind::Zip,
        "json" => IrArtifactKind::Json,
        "rss" => IrArtifactKind::Rss,
        _ => return None,
    })
}

pub fn assert_inventory_is_canonical(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let inv = load_ir_inventory(root)?;
    for s in inv.sources {
        let _ = parse_source_family(&s.source_family);
        parse_access_policy(&s.access_policy_class).ok_or("bad policy")?;
        for x in s.issuer_identity_kind {
            parse_identity_kind(&x).ok_or("bad identity")?;
        }
        for x in s.document_family {
            parse_document_family(&x).ok_or("bad doc")?;
        }
        for x in s.artifact_kind {
            parse_artifact_kind(&x).ok_or("bad artifact")?;
        }
    }
    Ok(())
}
