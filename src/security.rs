use crate::result;
use aes_gcm::aead::OsRng;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use base64::{engine::general_purpose, Engine as _};
use std::env;
use std::sync::LazyLock;
use totp_rs::{Algorithm, Secret, TOTP};

pub static OTP_CIPHER: LazyLock<Aes256Gcm> = LazyLock::new(|| {
    let key = env::var("OTP_KEY").expect("OTP_KEY must be set");
    let buffer = general_purpose::STANDARD
        .decode(&key)
        .expect("invalid OTP key, not properly encoded");
    let key = Key::<Aes256Gcm>::from_slice(&buffer);
    Aes256Gcm::new(&key)
});

pub fn generate_encoded_nonce() -> String {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    general_purpose::STANDARD.encode(&nonce)
}

pub fn generate_totp(cipher_text: Vec<u8>, email_address: String) -> Result<TOTP, result::Error> {
    Ok(TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Raw(cipher_text).to_bytes().unwrap(),
        Some("ONVP".to_owned()),
        email_address,
    )?)
}
