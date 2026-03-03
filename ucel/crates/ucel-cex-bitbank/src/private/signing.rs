use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn make_payload(timestamp: &str, method: &str, path_with_query: &str, body: &str) -> String {
    format!("{timestamp}{method}{path_with_query}{body}")
}

pub fn sign_hex(secret: &str, payload: &str) -> Result<String, String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| format!("hmac init failed: {e}"))?;
    mac.update(payload.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}
