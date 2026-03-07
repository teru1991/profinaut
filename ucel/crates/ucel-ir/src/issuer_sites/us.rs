use super::fetch::IssuerSiteAdapter;

pub fn us_issuer_html_adapter() -> IssuerSiteAdapter {
    IssuerSiteAdapter::new("us_issuer_ir_html_public")
}

pub fn us_issuer_feed_adapter() -> IssuerSiteAdapter {
    IssuerSiteAdapter::new("us_issuer_ir_feed_public")
}
