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

//! Work groups are collections of members, allowing for additional roles.

use crate::generic::result::BackendResult;
use crate::model::interface::commands::WorkgroupRegisterCommand;
use crate::model::interface::responses::WorkgroupResponse;
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::services::traits::command::WorkgroupCommandService;
use crate::services::traits::request::WorkgroupRequestService;
use actix_web::web::{Data, Json};
use actix_web::{get, post, web};

pub const CONTEXT: &str = "/api/workgroups";

/// Register a new work group
///
/// Work groups are used to create groups of members which have a particular extended task to
/// perform within the orchestra. Further, the members of a work group can have additional
/// functionality enabled through the role they have within the work group.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful registration"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    )
)]
#[post("/")]
pub async fn register(
    service: Data<dyn WorkgroupCommandService>,
    command: Json<WorkgroupRegisterCommand>,
) -> BackendResult<Json<i32>> {
    Ok(Json(service.register(&command)?))
}

/// Search for work groups
///
/// Searches the name of the work group
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "A list of matching work groups", body=[SearchResult<MemberWithDetail>]),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    ),
    params(
        ("q" = String, Query, description = "Part of the first name, last name and/or email address"),
        ("p" = Option<String>, Query, description = "The page offset to use (counting from 0)")
    )
)]
#[get("/search")]
pub async fn search(
    service: web::Data<dyn WorkgroupRequestService>,
    search_params: web::Query<SearchParams>,
) -> BackendResult<web::Json<SearchResult<WorkgroupResponse>>> {
    Ok(Json(service.search(&search_params)?))
}
