use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedExport {
    pub format: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    pub recipient_pubkey_ref: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("encrypt failed")]
    EncryptFailed,
    #[error("serialize failed: {0}")]
    Serde(#[from] serde_json::Error),
}

pub fn encrypt_bundle(bytes: &[u8], recipient_pubkey_ref: &str) -> Result<Vec<u8>, ExportError> {
    let key = XChaCha20Poly1305::generate_key(&mut OsRng);
    let cipher = XChaCha20Poly1305::new(&key);

    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut nonce);
    let nonce_ref = XNonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce_ref, bytes)
        .map_err(|_| ExportError::EncryptFailed)?;

    let output = EncryptedExport {
        format: "xchacha20poly1305".to_string(),
        nonce_b64: base64::engine::general_purpose::STANDARD.encode(nonce),
        ciphertext_b64: base64::engine::general_purpose::STANDARD.encode(ciphertext),
        recipient_pubkey_ref: recipient_pubkey_ref.to_string(),
    };

    Ok(serde_json::to_vec_pretty(&output)?)
}
