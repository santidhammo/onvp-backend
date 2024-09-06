use crate::model::members::Member;
use crate::model::security::TokenData;
use crate::security::OTP_CIPHER;
use crate::{dal, security, DbPool};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder};
use aes_gcm::aead::Aead;
use log::warn;
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
        (status = 400, description = "Bad Request", body=[String]),
        (status = 500, description = "Internal backend error", body=[String])
    )
)]
#[get("/activation/code/{activation_string}")]
pub async fn activation_code(
    pool: web::Data<DbPool>,
    activation_string: web::Path<String>,
) -> impl Responder {
    let member = match dal::members::get_member_by_activation_string(&pool, &activation_string) {
        Ok(member) => member,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
    };

    let totp = match get_member_totp(&pool, &member) {
        Ok(value) => value,
        Err(value) => return HttpResponse::InternalServerError().json(value),
    };

    match totp.get_qr_base64() {
        Ok(qr) => HttpResponse::Ok().json(qr),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
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
    pool: web::Data<DbPool>,
    activation_data: web::Json<TokenData>,
) -> impl Responder {
    let mut member = match dal::members::get_member_by_activation_string(
        &pool,
        &activation_data.activation_string,
    ) {
        Ok(member) => member,
        Err(e) => {
            warn!("Error when processing activation: {e}");
            return HttpResponse::BadRequest().json(());
        }
    };

    let totp = match get_member_totp(&pool, &member) {
        Ok(value) => value,
        Err(value) => return HttpResponse::InternalServerError().json(value),
    };

    match totp.check_current(&activation_data.token) {
        Ok(true) => match dal::members::activate(&pool, &mut member) {
            Ok(_) => HttpResponse::Ok().json(()),
            Err(_) => HttpResponse::BadRequest().json(()),
        },
        Ok(false) | Err(_) => HttpResponse::BadRequest().json(()),
    }
}

fn get_member_totp(pool: &Data<DbPool>, member: &Member) -> Result<TOTP, String> {
    let nonce = match member.decoded_nonce() {
        Ok(nonce) => nonce,
        Err(e) => return Err(e),
    };

    let activation_bytes = member.activation_string.as_bytes();
    let otp_cipher = OTP_CIPHER.deref();
    let cipher_text = match otp_cipher.encrypt(&nonce, activation_bytes) {
        Ok(cipher_text) => cipher_text,
        Err(e) => return Err(e.to_string()),
    };

    let email_address = match dal::members::get_member_details_by_id(&pool, &member.id) {
        Ok(details) => details.email_address,
        Err(e) => return Err(e.to_string()),
    };

    security::generate_totp(cipher_text, email_address).map_err(|e| e.to_string())
}
