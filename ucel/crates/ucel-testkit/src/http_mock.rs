use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub async fn start() -> MockServer {
    MockServer::start().await
}

pub async fn expect_method_path(server: &MockServer, m: &str, p: &str, status: u16) {
    Mock::given(method(m))
        .and(path(p))
        .respond_with(ResponseTemplate::new(status))
        .mount(server)
        .await;
}
