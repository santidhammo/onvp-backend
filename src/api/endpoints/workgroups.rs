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
use crate::model::interface::commands::{
    AssociateMemberToWorkgroupCommand, DissociateMemberFromWorkgroupCommand,
    WorkgroupRegisterCommand, WorkgroupUpdateCommand,
};
use crate::model::interface::responses::{MemberResponse, WorkgroupResponse};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::services::definitions::command::WorkgroupCommandService;
use crate::services::definitions::request::WorkgroupRequestService;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post, web, HttpResponse};
use std::ops::Deref;

/// Register a new work group
///
/// Work groups are used to create groups of members which have a particular extended task to
/// perform within the orchestra. Further, the members of a work group can have additional
/// functionality enabled through the role they have within the work group.
#[utoipa::path(
    tag = "workgroups",
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
    tag = "workgroups",
    responses(
        (status = 200, description = "A list of matching work groups", body=[SearchResult<WorkgroupResponse>]),
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
    tag = "workgroups",
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
    tag = "workgroups",
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

/// Unregister a work group
///
/// Unregisters an existing work group
#[utoipa::path(
    tag = "workgroups",
    responses(
        (status = 200, description = "Work group is unregistered"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=Option<String>),
    )
)]
#[delete("/{id}")]
pub async fn unregister(
    service: Data<dyn WorkgroupCommandService>,
    id: Path<i32>,
) -> BackendResult<HttpResponse> {
    service.unregister(id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}

/// List all the members of the work group
#[utoipa::path(
    tag = "workgroups",
    responses(
        (status = 200, description = "List of members in the work group", body=Vec<MemberResponse>),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=Option<String>),
    )
)]
#[get("/{id}/members")]
pub async fn find_members(
    service: Data<dyn WorkgroupRequestService>,
    id: Path<i32>,
) -> BackendResult<Json<Vec<MemberResponse>>> {
    Ok(Json(service.find_members_by_id(id.into_inner())?))
}

/// Searches for members which are available for the work group
///
/// Searches for all members which are not part of the given work group
#[utoipa::path(
    tag = "workgroups",
    responses(
        (status = 200, description = "List of available members to the work group", body=SearchResult<MemberResponse>),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal backend error", body=Option<String>),
    )
)]
#[get("/{id}/members/available/search")]
pub async fn available_members_search(
    service: Data<dyn WorkgroupRequestService>,
    id: Path<i32>,
    search_params: Query<SearchParams>,
) -> BackendResult<Json<SearchResult<MemberResponse>>> {
    Ok(Json(service.available_members_search(
        id.into_inner(),
        search_params.deref(),
    )?))
}

/// Associate a member to a work group
#[utoipa::path(
    tag = "workgroups",
    responses(
        (status = 200, description = "Successful association of a member to a work group"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/associate")]
pub async fn associate(
    service: Data<dyn WorkgroupCommandService>,
    command: Json<AssociateMemberToWorkgroupCommand>,
) -> BackendResult<HttpResponse> {
    service.associate_member_to_workgroup(&command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Dissociate a member to a work group
#[utoipa::path(
    tag = "workgroups",
    responses(
        (status = 200, description = "Successful dissociation of a member from a work group"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/dissociate")]
pub async fn dissociate(
    service: Data<dyn WorkgroupCommandService>,
    command: Json<DissociateMemberFromWorkgroupCommand>,
) -> BackendResult<HttpResponse> {
    service.dissociate_member_from_workgroup(&command)?;
    Ok(HttpResponse::Ok().finish())
}
