use crate::model::security::{Role, UserClaims};
use crate::result::Result;
use crate::Error;
use aes_gcm::aead::OsRng;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use base64::{engine::general_purpose, Engine as _};
use chrono::{TimeDelta, Utc};
use jwt_compact::UntrustedToken;
use std::env;
use std::ops::Add;
use std::sync::LazyLock;
use totp_rs::{Algorithm, Secret, TOTP};

/// This is the high watermark on which an access and/or refresh token needs to be recreated. If
/// an access token nearly expires, only a new access token is generated. The expiry of a
/// refresh token triggers a new log in procedure.
pub const TOKEN_EXPIRY_HIGH_WATER_MARK: i64 = 2 * 60;

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

pub fn generate_totp(cipher_text: Vec<u8>, email_address: String) -> Result<TOTP> {
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

pub fn operator_state_guard(claims: &UserClaims) -> Result<()> {
    if claims.has_role(Role::Operator) {
        Ok(())
    } else {
        Err(Error::bad_request())
    }
}

pub fn token_nearly_expires(token: UntrustedToken) -> Result<bool> {
    let expiration = token
        .deserialize_claims_unchecked::<UserClaims>()?
        .expiration
        .ok_or(Error::bad_request())?;
    let delta = TimeDelta::seconds(TOKEN_EXPIRY_HIGH_WATER_MARK);
    let high_water_mark = expiration.add(-delta);
    Ok(high_water_mark.le(&Utc::now()))
}

pub static MEMBER_ACTIVATION_MINUTES: LazyLock<TimeDelta> = LazyLock::new(|| {
    let key = env::var("MEMBER_ACTIVATION_MINUTES").expect("MEMBER_ACTIVATION_MINUTES must be set");
    let value = key
        .parse::<u32>()
        .expect("MEMBER_ACTIVATION_MINUTES must be integer");
    TimeDelta::minutes(value as i64)
});
pub static FIRST_OPERATOR_ACTIVATION_MINUTES: LazyLock<TimeDelta> = LazyLock::new(|| {
    let key = env::var("FIRST_OPERATOR_ACTIVATION_MINUTES")
        .expect("FIRST_OPERATOR_ACTIVATION_MINUTES must be set");
    let value = key
        .parse::<u32>()
        .expect("FIRST_OPERATOR_ACTIVATION_MINUTES must be integer");
    TimeDelta::minutes(value as i64)
});
