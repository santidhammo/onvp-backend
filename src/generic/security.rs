/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024.  Sjoerd van Leent
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::model::database::prelude::*;
use crate::model::security::{Role, UserClaims};
use crate::result::Result;
use crate::{dal, Error};
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use base64::{engine::general_purpose, Engine as _};
use chrono::{TimeDelta, Utc};
use jwt_compact::UntrustedToken;
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use std::env;
use std::ops::{Add, Deref};
use std::sync::LazyLock;
pub use totp_rs::TOTP;
use totp_rs::{Algorithm, Secret};

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

pub fn create_activation_string() -> String {
    let validation_string = Alphanumeric.sample_string(&mut thread_rng(), 32);
    validation_string
}

pub fn get_member_totp(conn: &mut dal::DbConnection, member: &Member) -> Result<TOTP> {
    let nonce = member.decoded_nonce()?;
    let activation_bytes = member.activation_string.as_bytes();
    let otp_cipher = OTP_CIPHER.deref();
    let cipher_text = otp_cipher.encrypt(&nonce, activation_bytes)?;
    let details = dal::members::find_detail_by_detail_id(conn, &member.member_details_id)?;
    generate_totp(cipher_text, details.email_address)
}

pub fn generate_qr_code(totp: TOTP) -> Result<String> {
    totp.get_qr_base64()
        .map_err(|e| Error::qr_code_generation(e))
}
