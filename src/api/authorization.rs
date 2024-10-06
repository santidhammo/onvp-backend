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
use crate::services::definitions::request::AuthorizationRequestService;
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
    authorization_request_service: Data<dyn AuthorizationRequestService>,
    login_data: Json<AuthorizationRequest>,
) -> BackendResult<HttpResponse> {
    info!("Attempting member login: {}", &login_data.email_address);
    let authorization_response = authorization_request_service.login(&login_data)?;
    let mut response = HttpResponse::Ok();
    for cookie in &authorization_response.clone().cookies {
        response.cookie(cookie.clone());
    }
    Ok(response.json(authorization_response))
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
    authorization_request_service: Data<dyn AuthorizationRequestService>,
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

    let authorization_response = authorization_request_service.refresh(
        &user_claims,
        &origin_access_cookie,
        &origin_refresh_cookie,
    )?;

    let mut response = HttpResponse::Ok();
    for cookie in &authorization_response.clone().cookies {
        response.cookie(cookie.clone());
    }
    Ok(response.json(authorization_response))
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
        response.cookie(cookie.clone());
    }
    Ok(response.finish())
}
