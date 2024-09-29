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

//! Members are a very core component of the backend and involve a lot of interfaces regarding
//! member management as well as performing requests regarding members from normal website usage.

use crate::generic::{activation, assets, security};
use crate::model::prelude::*;
use crate::{dal, Error, Result};
use actix_jwt_auth_middleware::TokenSigner;
use actix_web::cookie::time::OffsetDateTime;
use actix_web::cookie::{Cookie, Expiration, SameSite};
use actix_web::http::header::ContentType;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use diesel::Connection;
use jwt_compact::alg::Ed25519;
use jwt_compact::UntrustedToken;
use log::info;
use std::ops::Deref;

/// This is the context of this part of the API
pub const CONTEXT: &str = "/api/members";

/// Register a member
///
/// Registers a new member with the necessary details. Sends an E-Mail to the
/// newly registered member to activate the account.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful registration"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[put("/member")]
pub async fn register(
    pool: web::Data<dal::DbPool>,
    registration_data: web::Json<MemberRegisterCommand>,
) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    conn.transaction::<_, Error, _>(|conn| {
        // Create the member record
        let activation_string = security::create_activation_string();
        dal::members::register(conn, &registration_data, &activation_string)?;
        activation::send_activation_email(&registration_data.email_address, &activation_string)?;
        Ok(())
    })?;

    Ok(HttpResponse::Ok().finish())
}

/// Associate a role to member
///
/// Member role association is used to directly manage the role of a member.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful association of a role"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[put("/{id}/associate_role/{role}")]
pub async fn associate_role(
    pool: web::Data<dal::DbPool>,
    id_and_role: web::Path<(i32, Role)>,
) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    let (id, role) = id_and_role.deref();
    dal::members::associate_role(&mut conn, &id, &role)?;
    Ok(HttpResponse::Ok().finish())
}

/// Dissociate a role from a member
///
/// Member role association is used to directly manage the role of a member.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful dissociation of a role"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[delete("/{id}/dissociate_role/{role}")]
pub async fn dissociate_role(
    pool: web::Data<dal::DbPool>,
    id_and_role: web::Path<(i32, Role)>,
) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    let (id, role) = id_and_role.deref();
    dal::members::dissociate_role(&mut conn, &id, &role)?;
    Ok(HttpResponse::Ok().finish())
}

/// Get the roles of a member
///
/// Get the current available roles for a member
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "List of roles of the given member", body=[[Role]]),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[delete("/{id}/role_list")]
pub async fn roles(pool: web::Data<dal::DbPool>, id: web::Path<i32>) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    dal::members::role_list(&mut conn, &id)?;
    Ok(HttpResponse::Ok().finish())
}

/// Search for members
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
pub async fn search_member_details(
    pool: web::Data<dal::DbPool>,
    search_params: web::Query<SearchParams>,
) -> Result<web::Json<SearchResult<MemberWithDetailLogicalEntity>>> {
    let mut conn = dal::connect(&pool)?;
    let query = search_params.query.as_ref().ok_or(Error::bad_request())?;

    Ok(web::Json(dal::members::find_with_details_by_search_string(
        &mut conn,
        query,
        10,
        search_params.page_offset,
    )?))
}

/// Get a member and the primary detail by id
///
/// Searches for a member and the primary detail by using the member identifier. If found,
/// a single record with the member and primary detail is returned.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Member and primary detail", body=[MemberWithDetail]),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[get("/{id}/detail")]
pub async fn member_with_detail_by_id(
    pool: web::Data<dal::DbPool>,
    id: web::Path<i32>,
) -> Result<web::Json<MemberWithDetailLogicalEntity>> {
    let mut conn = dal::connect(&pool)?;

    Ok(web::Json(dal::members::get_member_with_detail_by_id(
        &mut conn,
        id.into_inner(),
    )?))
}

/// Save a member and the primary detail by id
///
/// Updates an existing member and primary detail record given the data.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Member is updated"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal backend error", body=[String]),
    )
)]
#[post("/{id}")]
pub async fn update(
    pool: web::Data<dal::DbPool>,
    id: web::Path<i32>,
    command: web::Json<MemberUpdateCommand>,
) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    dal::members::update(&mut conn, &id, &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Update the address information of a member
///
/// Given the address details of a member, saves te address details
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Member is updated"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal backend error", body=[String]),
    )
)]
#[post("/{id}/address")]
pub async fn update_address(
    pool: web::Data<dal::DbPool>,
    id: web::Path<i32>,
    command: web::Json<MemberUpdateAddressCommand>,
) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    dal::members::update_address(&mut conn, &id, &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Upload the picture of a member
///
/// Uploads the picture of a member, adjusting it to the appropriate size by cropping it and
/// scaling it automatically.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful upload of the picture", body=[String]),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[post("/{id}/picture")]
pub async fn upload_member_picture(
    pool: web::Data<dal::DbPool>,
    id: web::Path<i32>,
    data: web::Bytes,
) -> Result<web::Json<String>> {
    let mut conn = dal::connect(&pool)?;
    let result = assets::handle_upload_member_picture(&mut conn, &id, &data)?;
    Ok(web::Json(result))
}

/// Retrieves the picture of a member, if available
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful picture retrieval", content_type="image/png"),
        (status = 410, description = "Picture not available"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[get("/{id}/picture.png")]
pub async fn retrieve_member_picture_asset(
    pool: web::Data<dal::DbPool>,
    id: web::Path<i32>,
    claims: UserClaims,
) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    let result = if claims.has_role(Role::Operator) {
        assets::handle_retrieve_member_picture_operator(&mut conn, &id)?
    } else if claims.has_role(Role::Member) {
        assets::handle_retrieve_member_picture_dpia(&mut conn, &id)?
    } else {
        return Err(Error::bad_request());
    };
    match result {
        None => Ok(HttpResponse::Gone().finish()),
        Some(data) => Ok(HttpResponse::Ok()
            .insert_header(ContentType::png())
            .body(data)),
    }
}

/// Retrieves the picture asset identifier of a member
///
/// If a member has a picture asset identifier, retrieves it. If the result is empty, no picture
/// is available.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful picture retrieval", body=[Option<String>]),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[get("/{id}/picture")]
pub async fn retrieve_member_picture(
    pool: web::Data<dal::DbPool>,
    id: web::Path<i32>,
    claims: UserClaims,
) -> Result<web::Json<Option<String>>> {
    let mut conn = dal::connect(&pool)?;
    let member = dal::members::get_member_by_id(&mut conn, &id)?;
    let result = if claims.has_role(Role::Operator) {
        member.picture_asset_id
    } else if claims.has_role(Role::Member) {
        if member.allow_privacy_info_sharing {
            member.picture_asset_id
        } else {
            None
        }
    } else {
        return Err(Error::bad_request());
    };
    Ok(web::Json(result))
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
    pool: web::Data<dal::DbPool>,
    activation_string: web::Path<String>,
) -> Result<web::Json<String>> {
    let mut conn = dal::connect(&pool)?;
    let member = dal::members::get_member_by_activation_string(&mut conn, &activation_string)?;
    let totp = security::get_member_totp(&mut conn, &member)?;
    Ok(web::Json(security::generate_qr_code(totp)?))
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
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal backend error", body=[String]),
    )
)]
#[post("/activation/activate")]
pub async fn activate(
    pool: web::Data<dal::DbPool>,
    activation_data: web::Json<TokenData>,
) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    let member = dal::members::get_member_by_activation_string(
        &mut conn,
        &activation_data.activation_string,
    )?;
    let totp = security::get_member_totp(&mut conn, &member)?;
    totp.check_current(&activation_data.token)?;
    dal::members::activate(&mut conn, &member)?;
    Ok(HttpResponse::Ok().finish())
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
    pool: web::Data<dal::DbPool>,
    login_data: web::Json<LoginData>,
    token_signer: web::Data<TokenSigner<UserClaims, Ed25519>>,
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

fn pre_login(
    conn: &mut dal::DbConnection,
    login_data: &web::Json<LoginData>,
) -> Result<security::TOTP> {
    let member = dal::members::get_member_by_email_address(conn, &login_data.email_address)?;
    Ok(security::get_member_totp(conn, &member)?)
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
    pool: web::Data<dal::DbPool>,
    user_claims: UserClaims,
    http_request: HttpRequest,
    token_signer: web::Data<TokenSigner<UserClaims, Ed25519>>,
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
    pool: web::Data<dal::DbPool>,
) -> Result<web::Json<String>> {
    let mut conn = dal::connect(&pool)?;
    let member_details =
        dal::members::get_member_detail_by_email_address(&mut conn, &user_claims.email_address)?;
    let name = format!("{} {}", member_details.first_name, member_details.last_name)
        .trim()
        .to_string();
    Ok(web::Json(name))
}

/// Is logged in member an operator
///
/// Returns if the logged in member is an operator
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful check on the operator role"),
        (status = 400, description = "Bad Request", body=[String])
    )
)]
#[get("/logged_in_is_operator")]
pub async fn logged_in_is_operator(user_claims: UserClaims) -> Result<HttpResponse> {
    match user_claims.has_role(Role::Operator) {
        true => Ok(HttpResponse::Ok().finish()),
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
