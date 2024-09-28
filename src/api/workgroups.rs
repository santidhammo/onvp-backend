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

use crate::model::commands::WorkgroupRegisterCommand;
use crate::model::security::Role;
use crate::{dal, Result};
use actix_web::{delete, put, web, HttpResponse};
use std::ops::Deref;

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
#[put("/")]
pub async fn register(
    pool: web::Data<dal::DbPool>,
    data: web::Json<WorkgroupRegisterCommand>,
) -> Result<HttpResponse> {
    let mut conn = dal::connect(&pool)?;
    dal::workgroups::register(&mut conn, &data)?;
    Ok(HttpResponse::Ok().finish())
}

/// Associate a role to a work group
///
/// Work group role association is used to allow members of a workgroup to have an extended role if
/// they are part of the work group.
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
    dal::workgroups::associate_role(&mut conn, &id, &role)?;
    Ok(HttpResponse::Ok().finish())
}

/// Dissociate a role from a work group
///
/// Work group role association is used to allow members of a workgroup to have an extended role if
/// they are part of the work group.
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
    dal::workgroups::dissociate_role(&mut conn, &id, &role)?;
    Ok(HttpResponse::Ok().finish())
}
