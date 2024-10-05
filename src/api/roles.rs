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

use crate::generic::result::BackendResult;
use crate::model::interface::commands::{AssociateRoleCommand, DissociateRoleCommand};
use crate::model::primitives::{Role, RoleClass};
use crate::services::definitions::command::RoleCommandService;
use crate::services::definitions::request::RoleRequestService;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, HttpResponse};

/// This is the context of the roles part of the API
pub const CONTEXT: &str = "/api/roles";

/// Associate a role to a member or work group
///
/// Member association is used to allow members to act on specific roles
/// Work group association is used to allow groups of members to act on specific roles
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful association of a role"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/associate")]
pub async fn associate(
    service: Data<dyn RoleCommandService>,
    command: Json<AssociateRoleCommand>,
) -> BackendResult<HttpResponse> {
    service.associate_role(&command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Dissociate a role from a member or work group
///
/// Member association is used to allow members to act on specific roles
/// Work group association is used to allow groups of members to act on specific roles
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "Successful dissociation of a role"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/dissociate")]
pub async fn dissociate(
    service: Data<dyn RoleCommandService>,
    command: Json<DissociateRoleCommand>,
) -> BackendResult<HttpResponse> {
    service.dissociate_role(&command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Lists roles for a member or work group
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "List of roles", body=Vec<Role>),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/{class}/list/{id}")]
pub async fn list(
    service: Data<dyn RoleRequestService>,
    path: Path<(RoleClass, i32)>,
) -> BackendResult<Json<Vec<Role>>> {
    let (class, id) = path.into_inner();
    let roles = service.list_by_id_and_class(id, class)?;
    Ok(Json(roles))
}
