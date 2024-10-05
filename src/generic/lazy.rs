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

use aes_gcm::{Aes256Gcm, Key, KeyInit};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::TimeDelta;
use lettre::message::Mailbox;
use std::env::var;
use std::sync::LazyLock;

pub static MEMBER_ACTIVATION_MINUTES: LazyLock<TimeDelta> = LazyLock::new(|| {
    let key = var("MEMBER_ACTIVATION_MINUTES").expect("MEMBER_ACTIVATION_MINUTES must be set");
    let value = key
        .parse::<u32>()
        .expect("MEMBER_ACTIVATION_MINUTES must be integer");
    TimeDelta::minutes(value as i64)
});
pub static FIRST_OPERATOR_ACTIVATION_MINUTES: LazyLock<TimeDelta> = LazyLock::new(|| {
    let key = var("FIRST_OPERATOR_ACTIVATION_MINUTES")
        .expect("FIRST_OPERATOR_ACTIVATION_MINUTES must be set");
    let value = key
        .parse::<u32>()
        .expect("FIRST_OPERATOR_ACTIVATION_MINUTES must be integer");
    TimeDelta::minutes(value as i64)
});
/// Returns the page size of each page for a search, defaults to 10 if the environment variable
/// SEARCH_PAGE_SIZE is not set.
pub static SEARCH_PAGE_SIZE: LazyLock<usize> = LazyLock::new(|| {
    var("SEARCH_PAGE_SIZE")
        .unwrap_or("10".to_owned())
        .parse()
        .expect("invalid SEARCH_PAGE_SIZE, should be an unsigned integer")
});
pub static OTP_CIPHER: LazyLock<Aes256Gcm> = LazyLock::new(|| {
    let key = var("OTP_KEY").expect("OTP_KEY must be set");
    let buffer = general_purpose::STANDARD
        .decode(&key)
        .expect("invalid OTP key, not properly encoded");
    let key = Key::<Aes256Gcm>::from_slice(&buffer);
    Aes256Gcm::new(&key)
});

/// Returns the token expiry high water mark, defaults to 120 seconds if the environment variable
/// TOKEN_EXPIRY_HIGH_WATER_MARK is not set.
pub static TOKEN_EXPIRY_HIGH_WATER_MARK: LazyLock<u64> = LazyLock::new(|| {
    var("TOKEN_EXPIRY_HIGH_WATER_MARK")
        .map(|s| {
            s.parse()
                .expect("TOKEN_EXPIRY_HIGH_WATER_MARK must be a positive integer")
        })
        .unwrap_or(120)
});

pub static SEND_ACTIVATION_EMAIL_CONFIG: LazyLock<SendEmailConfig> = LazyLock::new(|| {
    let email_dev_mode: bool = var("EMAIL_DEV_MODE")
        .unwrap_or("false".to_owned())
        .parse()
        .expect("invalid EMAIL_DEV_MODE, should be a boolean");
    let email_from: Mailbox = var("EMAIL_FROM")
        .expect("EMAIL_FROM must be set")
        .parse()
        .expect("invalid EMAIL_FROM");
    let email_subject =
        var("EMAIL_REGISTRATION_SUBJECT").expect("EMAIL_REGISTRATION_SUBJECT must be set");
    let email_body_template =
        var("EMAIL_REGISTRATION_BODY").expect("EMAIL_REGISTRATION_BODY must be set");
    let email_smtp_user = if !email_dev_mode {
        var("EMAIL_SMTP_USER").expect("EMAIL_SMTP_USER must be set")
    } else {
        "".to_owned()
    };
    let email_smtp_password = if !email_dev_mode {
        var("EMAIL_SMTP_PASSWORD").expect("EMAIL_SMTP_PASSWORD must be set")
    } else {
        "".to_owned()
    };
    let email_smtp_relay = var("EMAIL_SMTP_RELAY").expect("EMAIL_SMTP_RELAY must be set");
    let email_smtp_port: u16 = var("EMAIL_SMTP_PORT")
        .unwrap_or("587".to_owned())
        .parse()
        .expect("invalid EMAIL_SMTP_PORT, should be an unsigned integer");

    SendEmailConfig {
        email_dev_mode,
        email_from,
        email_subject,
        email_body_template,
        email_smtp_user,
        email_smtp_password,
        email_smtp_relay,
        email_smtp_port,
    }
});

#[derive(Clone)]
pub struct SendEmailConfig {
    pub email_dev_mode: bool,
    pub email_from: Mailbox,
    pub email_subject: String,
    pub email_body_template: String,
    pub email_smtp_user: String,
    pub email_smtp_password: String,
    pub email_smtp_relay: String,
    pub email_smtp_port: u16,
}
