use ucel_cex_bitbank::private::signing::{make_payload, sign_hex};

#[test]
fn signing_golden_get() {
    let payload = make_payload("1700000000000", "GET", "/v1/me/getbalance?foo=bar", "");
    let sig = sign_hex("dummy_secret", &payload).unwrap();
    assert_eq!(
        sig,
        "f32b1d556bb3a2be2655f9896469701afba076cac831e58437fdfbaea491697a"
    );
}

#[test]
fn signing_golden_post_body() {
    let payload = make_payload(
        "1700000000000",
        "POST",
        "/v1/me/sendchildorder",
        r#"{"a":1}"#,
    );
    let sig = sign_hex("dummy_secret", &payload).unwrap();
    assert_eq!(
        sig,
        "6d40e247ef8323411601ba5b3832021f3b6cf87db59212012b21e6051a7e5c8e"
    );
}

#[test]
fn signing_golden_post_empty_body() {
    let payload = make_payload("1700000000000", "POST", "/v1/me/empty", "");
    let sig = sign_hex("dummy_secret", &payload).unwrap();
    assert_eq!(
        sig,
        "1d093ded3a6bb1b50704a1ae45dd389c3aba565e78f915607f0fddde7d6ef948"
    );
}
