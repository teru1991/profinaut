use ucel_transport::security::{EndpointAllowlist, SubdomainPolicy};

#[test]
fn registry_blocks_http_and_unknown_host() {
    let al = EndpointAllowlist::new(["api.example.com"], SubdomainPolicy::Exact).unwrap();
    assert!(al.validate_https_wss("http://api.example.com").is_err());
    assert!(al.validate_https_wss("https://evil.com").is_err());
}

#[test]
fn registry_allows_https_known_host() {
    let al = EndpointAllowlist::new(["api.example.com"], SubdomainPolicy::Exact).unwrap();
    al.validate_https_wss("https://api.example.com/v1").unwrap();
}
