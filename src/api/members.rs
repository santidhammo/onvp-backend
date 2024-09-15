//! This file contains the API to manage members and to perform the necessary log in, log out and
//! token refreshing routines for a member.

use crate::model::generic::{SearchParams, SearchResult};
use crate::model::members::{Member, MemberWithDetail};
use crate::model::security::{LoginData, Role, TokenData, UserClaims};
use crate::security::OTP_CIPHER;
use crate::{dal, security, Error, Result};
use actix_jwt_auth_middleware::TokenSigner;
use actix_web::cookie::time::OffsetDateTime;
use actix_web::cookie::{Cookie, Expiration, SameSite};
use actix_web::web::{Data, Json, Query};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use aes_gcm::aead::Aead;
use jwt_compact::alg::Ed25519;
use jwt_compact::UntrustedToken;
use log::info;
use std::ops::Deref;
use totp_rs::TOTP;

/// This is the context of this part of the API
pub const CONTEXT: &str = "/api/members";

/// Searches for members
///
/// Searches on first name, last name and/or email address matching the given query. If no query
/// is given, results in a Bad Request
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "A list of matching members", body=[SearchResult<MemberWithDetail>]),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    ),
    params(
        ("q" = String, Query, description = "Part of the first name, last name and/or email address"),
        ("p" = Option<String>, Query, description = "The page offset to use (counting from 0)")
    )
)]
#[get("/search_member_details")]
pub async fn search_member_details<'p>(
    pool: Data<dal::DbPool>,
    search_params: Query<SearchParams>,
) -> Result<Json<SearchResult<MemberWithDetail>>> {
    let mut conn = dal::connect(&pool)?;
    let query = search_params.query.as_ref().ok_or(Error::bad_request())?;
    // The query should not be empty
    if query.is_empty() {
        return Err(Error::bad_request());
    }

    Ok(Json(
        dal::members::find_members_with_details_by_search_string(
            &mut conn,
            query,
            20,
            search_params.page_offset,
        )?,
    ))
}

/// Generate an activation code
///
/// Generates an activation code for a user to be activated
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "The activation code (in QR form)", body=[String]),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal backend error")
    )
)]
#[get("/activation/code/{activation_string}")]
pub async fn activation_code(
    pool: Data<dal::DbPool>,
    activation_string: web::Path<String>,
) -> Result<Json<String>> {
    let mut conn = dal::connect(&pool)?;
    let member = dal::members::get_member_by_activation_string(&mut conn, &activation_string)?;
    let totp = get_member_totp(&mut conn, &member)?;
    Ok(Json(generate_qr_code(totp)?))
}

/// Activate a member
///
/// Returns if the member is activated if a member can be activated. If a member does not exist,
/// returns a Bad Request. if a member is already activated by the activation string it also returns
/// a Bad Request.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Member is activated"),
        (status = 400, description = "Bad Request", body=[String]),
        (status = 500, description = "Internal backend error", body=[String]),
    )
)]
#[post("/activation/activate")]
pub async fn activate(
    pool: Data<dal::DbPool>,
    activation_data: Json<TokenData>,
) -> Result<Json<()>> {
    let mut conn = dal::connect(&pool)?;
    let member = dal::members::get_member_by_activation_string(
        &mut conn,
        &activation_data.activation_string,
    )?;
    let totp = get_member_totp(&mut conn, &member)?;
    totp.check_current(&activation_data.token)?;
    dal::members::activate(&mut conn, &member)?;
    Ok(Json(()))
}

/// Login a member
///
/// Logs in the member using the OTP code, then creates a JWT token out of that, which can be
/// further verified against the software issuing the token.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Logged in successfully"),
        (status = 400, description = "Bad Request", body=[String]),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[post("/login")]
pub async fn login(
    pool: Data<dal::DbPool>,
    login_data: Json<LoginData>,
    token_signer: Data<TokenSigner<UserClaims, Ed25519>>,
) -> Result<HttpResponse> {
    info!("Attempting member login: {}", &login_data.email_address);
    let mut conn = dal::connect(&pool)?;
    let totp = match pre_login(&mut conn, &login_data) {
        Ok(totp) => totp,
        Err(_) => return Ok(HttpResponse::Forbidden().finish()),
    };

    let is_current = match totp.check_current(&login_data.token) {
        Ok(checked) => checked,
        Err(_) => return Ok(HttpResponse::Forbidden().finish()),
    };

    if is_current {
        login_or_renew(&mut conn, &login_data.email_address, &token_signer)
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}

fn pre_login(conn: &mut dal::DbConnection, login_data: &Json<LoginData>) -> Result<TOTP> {
    let member = dal::members::get_member_by_email_address(conn, &login_data.email_address)?;
    Ok(get_member_totp(conn, &member)?)
}

/// Check login status
///
/// Checks if the member has already logged in
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Logged in successfully"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[get("/check_login_status")]
pub async fn check_login_status(
    pool: Data<dal::DbPool>,
    user_claims: UserClaims,
    http_request: HttpRequest,
    token_signer: Data<TokenSigner<UserClaims, Ed25519>>,
) -> Result<HttpResponse> {
    info!("Refreshing member login: {}", &user_claims.email_address);
    let origin_access_cookie = http_request
        .cookie("access_token")
        .ok_or(Error::bad_request())?;
    let origin_refresh_cookie = http_request
        .cookie("refresh_token")
        .ok_or(Error::bad_request())?;

    // Convert cookies to the associated tokens. Verification is already done at this point in time,
    // it is only necessary to refresh the situation appropriately.
    let origin_access_token = UntrustedToken::new(origin_access_cookie.value())?;
    let origin_refresh_token = UntrustedToken::new(origin_refresh_cookie.value())?;

    // If the refresh token nearly expires, the login procedure is transparently performed, to
    // ensure that user roles are still the same. If the access token nearly expires, then a new
    // access token is simply created, otherwise nothing is done.
    if security::token_nearly_expires(origin_refresh_token)? {
        info!(
            "Refreshing tokens for member: {}",
            &user_claims.email_address
        );
        let mut conn = dal::connect(&pool)?;
        login_or_renew(&mut conn, &user_claims.email_address, &token_signer)
    } else if security::token_nearly_expires(origin_access_token)? {
        info!(
            "Create new access token for member: {}",
            &user_claims.email_address
        );
        Ok(HttpResponse::Ok()
            .cookie(token_signer.create_access_cookie(&user_claims)?)
            .json(()))
    } else {
        Ok(HttpResponse::Ok().json(()))
    }
}

/// Logout a member
///
/// Logs out a member, if already logged in
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Logged in successfully"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[get("/logout")]
pub async fn logout() -> HttpResponse {
    let access_cookie = Cookie::build("access_token".to_string(), "")
        .secure(true)
        .same_site(SameSite::Strict)
        .expires(Expiration::DateTime(OffsetDateTime::UNIX_EPOCH))
        .finish();

    let refresh_cookie = Cookie::build("refresh_token".to_string(), "")
        .secure(true)
        .same_site(SameSite::Strict)
        .expires(Expiration::DateTime(OffsetDateTime::UNIX_EPOCH))
        .finish();

    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(())
}

/// Logged in member name
///
/// Gets the name of the logged in member
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "The member name", body=[String]),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[get("/logged_in_name")]
pub async fn logged_in_name(
    user_claims: UserClaims,
    pool: Data<dal::DbPool>,
) -> Result<Json<String>> {
    let mut conn = dal::connect(&pool)?;
    let member_details =
        dal::members::get_member_details_by_email_address(&mut conn, &user_claims.email_address)?;
    let name = format!("{} {}", member_details.first_name, member_details.last_name)
        .trim()
        .to_string();
    Ok(Json(name))
}

/// Is logged in member an operator
///
/// Returns if the logged in member is an operator
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "The member name"),
        (status = 400, description = "Bad Request", body=[String])
    )
)]
#[get("/logged_in_is_operator")]
pub async fn logged_in_is_operator(user_claims: UserClaims) -> Result<Json<()>> {
    match user_claims.has_role(Role::Operator) {
        true => Ok(Json(())),
        false => Err(Error::bad_request()),
    }
}

fn login_or_renew(
    conn: &mut dal::DbConnection,
    email_address: &str,
    token_signer: &TokenSigner<UserClaims, Ed25519>,
) -> Result<HttpResponse> {
    let member = dal::members::get_member_by_email_address(conn, email_address)?;
    let member_roles = dal::members::get_member_roles_by_member_id(conn, &member.id)?;
    let user_claims = UserClaims::new(&email_address, &member_roles);

    let mut access_cookie = token_signer.create_access_cookie(&user_claims)?;
    let mut refresh_cookie = token_signer.create_refresh_cookie(&user_claims)?;

    access_cookie.set_same_site(SameSite::Strict);
    refresh_cookie.set_same_site(SameSite::Strict);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(()))
}

fn get_member_totp(conn: &mut dal::DbConnection, member: &Member) -> Result<TOTP> {
    let nonce = member.decoded_nonce()?;
    let activation_bytes = member.activation_string.as_bytes();
    let otp_cipher = OTP_CIPHER.deref();
    let cipher_text = otp_cipher.encrypt(&nonce, activation_bytes)?;
    let details = dal::members::get_member_details_by_id(conn, &member.id)?;
    security::generate_totp(cipher_text, details.email_address)
}

fn generate_qr_code(totp: TOTP) -> Result<String> {
    totp.get_qr_base64()
        .map_err(|e| Error::qr_code_generation(e))
}
