use super::fetch::IssuerSiteAdapter;

pub fn jp_issuer_html_adapter() -> IssuerSiteAdapter {
    IssuerSiteAdapter::new("jp_issuer_ir_html_public")
}

pub fn jp_issuer_feed_adapter() -> IssuerSiteAdapter {
    IssuerSiteAdapter::new("jp_issuer_ir_feed_public")
}
