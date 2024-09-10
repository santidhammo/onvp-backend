use crate::dal::DbConnection;
use crate::model::members::Member;
use crate::model::security::{LoginData, Role, TokenData, UserClaims};
use crate::security::OTP_CIPHER;
use crate::{dal, result, security, Error};
use actix_jwt_auth_middleware::TokenSigner;
use actix_web::cookie::time::OffsetDateTime;
use actix_web::cookie::{Cookie, Expiration, SameSite};
use actix_web::web::{Data, Json};
use actix_web::{get, post, web, HttpResponse, Responder};
use aes_gcm::aead::Aead;
use jwt_compact::alg::Ed25519;
use log::info;
use std::ops::Deref;
use totp_rs::TOTP;

pub const CONTEXT: &str = "/api/members";

/// Searches for members by first name, last name and/or email address
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "A list of matching members", body=[Vec<MemberDetail>]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[get("/search_user")]
pub async fn search() -> impl Responder {
    let members: Vec<Member> = vec![];
    HttpResponse::Ok().json(members)
}k

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
) -> Result<Json<String>, Error> {
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
    activation_data: web::Json<TokenData>,
) -> Result<Json<()>, Error> {
    let mut conn = dal::connect(&pool)?;
    let member = dal::members::get_member_by_activation_string(
        &mut conn,
        &activation_data.activation_string,
    )?;
    let totp = get_member_totp(&mut conn, &member)?;
    totp.check_current(&activation_data.token)?;
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
) -> Result<HttpResponse, Error> {
    info!("Attempting member login: {}", &login_data.email_address);
    let mut conn = dal::connect(&pool)?;
    let member = dal::members::get_member_by_email_address(&mut conn, &login_data.email_address)?;
    let member_roles = dal::members::get_member_roles_by_member_id(&mut conn, &member.id)?;
    let claim = UserClaims::new(&login_data.email_address, &member_roles);

    let mut access_cookie = token_signer.create_access_cookie(&claim)?;
    let mut refresh_cookie = token_signer.create_refresh_cookie(&claim)?;

    access_cookie.set_same_site(SameSite::Strict);
    refresh_cookie.set_same_site(SameSite::Strict);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(()))
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
pub async fn check_login_status() -> Json<()> {
    Json(())
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
) -> Result<Json<String>, Error> {
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
pub async fn logged_in_is_operator(user_claims: UserClaims) -> Result<Json<()>, Error> {
    match user_claims.has_role(Role::Operator) {
        true => Ok(Json(())),
        false => Err(Error::bad_request()),
    }
}

fn get_member_totp(conn: &mut DbConnection, member: &Member) -> Result<TOTP, Error> {
    let nonce = member.decoded_nonce()?;
    let activation_bytes = member.activation_string.as_bytes();
    let otp_cipher = OTP_CIPHER.deref();
    let cipher_text = otp_cipher.encrypt(&nonce, activation_bytes)?;
    let details = dal::members::get_member_details_by_id(conn, &member.id)?;
    security::generate_totp(cipher_text, details.email_address)
}

fn generate_qr_code(totp: TOTP) -> Result<String, result::Error> {
    totp.get_qr_base64()
        .map_err(|e| result::Error::qr_code_generation(e))
}
