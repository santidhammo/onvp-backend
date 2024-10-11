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
use crate::model::interface::commands::{WorkgroupRegisterCommand, WorkgroupUpdateCommand};
use crate::model::interface::responses::WorkgroupResponse;
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::services::definitions::command::WorkgroupCommandService;
use crate::services::definitions::request::WorkgroupRequestService;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, web, HttpResponse};

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
    service: Data<dyn WorkgroupRequestService>,
    search_params: web::Query<SearchParams>,
) -> BackendResult<Json<SearchResult<WorkgroupResponse>>> {
    Ok(Json(service.search(&search_params)?))
}

/// Get a work group by id
///
/// Searches for a work group by using the work group identifier. If found,
/// a single record with the work group is returned.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Work group", body=WorkgroupResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/{id}")]
pub async fn find(
    service: Data<dyn WorkgroupRequestService>,
    id: Path<i32>,
) -> BackendResult<Json<WorkgroupResponse>> {
    Ok(Json(service.find_by_id(id.into_inner())?))
}

/// Save a work group by id
///
/// Updates an existing work group record given the data.
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Work group is updated"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=[String]),
    )
)]
#[post("/{id}")]
pub async fn update(
    service: Data<dyn WorkgroupCommandService>,
    id: Path<i32>,
    command: Json<WorkgroupUpdateCommand>,
) -> BackendResult<HttpResponse> {
    service.update(id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}
