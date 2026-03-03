use ucel_testkit::http_mock;

#[tokio::test]
async fn domestic_private_request_shapes_are_stable() {
    let server = http_mock::start().await;
    http_mock::expect_method_path(&server, "GET", "/v1/user/assets", 200).await;
    http_mock::expect_method_path(&server, "GET", "/v1/me/getbalance", 200).await;
    http_mock::expect_method_path(&server, "GET", "/v1/account/assets", 200).await;
    http_mock::expect_method_path(&server, "GET", "/api/accounts/balance", 200).await;

    let bb = ucel_cex_bitbank::private::request_builders::build_get_assets_request(
        "dummy_key",
        "123",
        "dummy_sig",
    );
    assert_eq!(bb.method, "GET");
    assert_eq!(bb.path, "/v1/user/assets");
    assert!(bb.headers.contains_key("ACCESS-KEY"));
    assert!(bb.headers.contains_key("ACCESS-NONCE"));
    assert!(bb.headers.contains_key("ACCESS-SIGNATURE"));

    let bf = ucel_cex_bitflyer::private::request_builders::build_get_balance_request(
        "dummy_key",
        "1700000000000",
        "dummy_sig",
    );
    assert_eq!(bf.method, "GET");
    assert_eq!(bf.path, "/v1/me/getbalance");
    assert!(bf.headers.contains_key("ACCESS-KEY"));
    assert!(bf.headers.contains_key("ACCESS-TIMESTAMP"));
    assert!(bf.headers.contains_key("ACCESS-SIGN"));

    let gmo = ucel_cex_gmocoin::private::request_builders::build_get_balance_request(
        "dummy_key",
        "1700000000000",
        "dummy_sig",
    );
    assert_eq!(gmo.method, "GET");
    assert_eq!(gmo.path, "/v1/account/assets");
    assert!(gmo.headers.contains_key("API-KEY"));
    assert!(gmo.headers.contains_key("API-TIMESTAMP"));
    assert!(gmo.headers.contains_key("API-SIGN"));

    let cc = ucel_cex_coincheck::private::request_builders::build_get_balance_request(
        "dummy_key",
        "123",
        "dummy_sig",
    );
    assert_eq!(cc.method, "GET");
    assert_eq!(cc.path, "/api/accounts/balance");
    assert!(cc.headers.contains_key("ACCESS-KEY"));
    assert!(cc.headers.contains_key("ACCESS-NONCE"));
    assert!(cc.headers.contains_key("ACCESS-SIGNATURE"));
}
