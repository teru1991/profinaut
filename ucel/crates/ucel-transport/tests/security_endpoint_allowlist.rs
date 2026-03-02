use ucel_transport::security::{EndpointAllowlist, SubdomainPolicy};

#[test]
fn allowlist_allows_https_and_exact_host() {
    let al = EndpointAllowlist::new(["api.example.com"], SubdomainPolicy::Exact).unwrap();
    let u = al.validate_https_wss("https://api.example.com/v1").unwrap();
    assert_eq!(u.scheme(), "https");
    assert_eq!(u.host_str().unwrap(), "api.example.com");
}

#[test]
fn allowlist_rejects_http_scheme() {
    let al = EndpointAllowlist::new(["api.example.com"], SubdomainPolicy::Exact).unwrap();
    let err = al
        .validate_https_wss("http://api.example.com/v1")
        .unwrap_err();
    assert!(err.message.contains("scheme"));
}

#[test]
fn allowlist_subdomains_policy() {
    let al = EndpointAllowlist::new(["example.com"], SubdomainPolicy::AllowSubdomains).unwrap();
    al.validate_https_wss("wss://stream.example.com/ws")
        .unwrap();
}

#[test]
fn allowlist_rejects_non_allowlisted_host() {
    let al = EndpointAllowlist::new(["example.com"], SubdomainPolicy::Exact).unwrap();
    let err = al.validate_https_wss("https://evil.com/").unwrap_err();
    assert!(err.message.contains("not allowlisted"));
}
