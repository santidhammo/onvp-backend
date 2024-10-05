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

use crate::generic::result::{BackendError, BackendResult};
use crate::model::interface::client::UserClaims;
use crate::model::interface::requests::AuthorizationRequest;
use crate::model::primitives::Role;
use crate::services::traits::request::{AuthorizationRequestService, MemberRequestService};
use actix_web::web::{Data, Json};
use actix_web::{get, post, HttpRequest, HttpResponse};
use log::info;

/// This is the context of the authorization part of the API
pub const CONTEXT: &str = "/api/authorization";

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
    service: Data<dyn AuthorizationRequestService>,
    login_data: Json<AuthorizationRequest>,
) -> BackendResult<HttpResponse> {
    info!("Attempting member login: {}", &login_data.email_address);
    let cookies = service.login(&login_data)?;
    let mut response = HttpResponse::Ok();
    for cookie in cookies {
        response.cookie(cookie);
    }
    Ok(response.finish())
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
#[get("/refresh")]
pub async fn refresh(
    service: Data<dyn AuthorizationRequestService>,
    user_claims: UserClaims,
    http_request: HttpRequest,
) -> BackendResult<HttpResponse> {
    info!("Refreshing member login: {}", &user_claims.email_address);
    let origin_access_cookie = http_request
        .cookie("access_token")
        .ok_or(BackendError::bad())?;
    let origin_refresh_cookie = http_request
        .cookie("refresh_token")
        .ok_or(BackendError::bad())?;

    let cookies = service.refresh(&user_claims, &origin_access_cookie, &origin_refresh_cookie)?;
    let mut response = HttpResponse::Ok();
    for cookie in cookies {
        response.cookie(cookie);
    }
    Ok(response.finish())
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
pub async fn logout(service: Data<dyn AuthorizationRequestService>) -> BackendResult<HttpResponse> {
    let cookies = service.logout()?;
    let mut response = HttpResponse::Ok();
    for cookie in cookies {
        response.cookie(cookie);
    }
    Ok(response.finish())
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
    service: Data<dyn MemberRequestService>,
) -> BackendResult<Json<String>> {
    let member_response = service.find_by_email_address(&user_claims.email_address)?;
    Ok(Json(member_response.full_name()))
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
pub async fn logged_in_is_operator(user_claims: UserClaims) -> BackendResult<HttpResponse> {
    match user_claims.has_role(Role::Operator) {
        true => Ok(HttpResponse::Ok().finish()),
        false => Err(BackendError::bad()),
    }
}
