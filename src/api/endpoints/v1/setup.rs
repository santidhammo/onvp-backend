/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024-2025.  Sjoerd van Leent
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

use crate::generic::result::BackendResult;
use crate::generic::storage::session::Session;
use crate::model::interface::commands::FirstOperatorRegisterCommand;
use crate::services::definitions::command::SetupCommandService;
use crate::services::definitions::request::SetupRequestService;
use actix_web::web::{Data, Json};
use actix_web::{get, post};

/// Should setup be started?
///
/// Checks if the software should run the set-up procedure. Returns true when there are no
/// (operator) members. In that case, the set-up procedure should be started.
#[utoipa::path(
    tag = "setup",
    responses(
        (status = 200, description = "Returns whether or not operators are available", body=bool),
        (status = 500, description = "Internal backend error", body=[String])
    )
)]
#[get("/should_setup")]
pub async fn should_setup(
    session: Session,
    service: Data<dyn SetupRequestService>,
) -> BackendResult<Json<bool>> {
    Ok(Json(service.should_setup(session)?))
}

/// Set up the first operator
///
/// The first operator should contain enough information to create a member with the operator role,
/// and the associated details, including the address details, such that the system can be started.
/// The whole operation is performed using two steps:
/// 1. Enter the data into the storage;
/// 2. Let the frontend navigate to the account activation step using a **TOTP** solution.
///
/// ⚠️ If an operator already exists, this API call (for obvious reasons) becomes invalid.
#[utoipa::path(
    tag = "setup",
    responses(
        (status = 200, description = "Created a new first operator", body=String),
        (status = 400, description = "Bad Request", body=String),
        (status = 500, description = "Internal backend error", body=String)
    )
)]
#[post("/setup_first_operator")]
pub async fn setup_first_operator(
    session: Session,
    command: Json<FirstOperatorRegisterCommand>,
    service: Data<dyn SetupCommandService>,
) -> BackendResult<Json<String>> {
    Ok(Json(service.register_first_operator(session, &command)?))
}
