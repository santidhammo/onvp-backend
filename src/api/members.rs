use crate::dal::DbConnection;
use crate::model::members::Member;
use crate::model::security::{LoginData, Role, TokenData, UserClaims};
use crate::security::OTP_CIPHER;
use crate::{dal, result, security, Error};
use actix_jwt_auth_middleware::TokenSigner;
use actix_web::cookie::SameSite;
use actix_web::dev::ConnectionInfo;
use actix_web::web::{Data, Json};
use actix_web::{get, post, web, HttpResponse, Responder};
use aes_gcm::aead::Aead;
use jwt_compact::alg::Ed25519;
use log::info;
use std::collections::HashSet;
use std::ops::Deref;
use totp_rs::TOTP;

pub const CONTEXT: &str = "/api/members";

#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Receive a list of members", body=[Member])
    )
)]
#[get("/list")]
pub async fn list() -> impl Responder {
    let members: Vec<Member> = vec![];
    HttpResponse::Ok().json(members)
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
