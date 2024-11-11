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
use crate::generic::security::ClaimRoles;
use crate::model::interface::commands::{CreatePageCommand, PublishPageCommand, UpdatePageCommand};
use crate::model::interface::responses::{ExtendedPageResponse, PageResponse};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::services::definitions::command::PageCommandService;
use crate::services::definitions::request::PageRequestService;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, post, put, HttpResponse};
use std::ops::Deref;

/// Search for pages
///
/// Searches on titles matching the given query.
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "A list of matching pages", body=SearchResult<PageResponse>),
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
    service: Data<dyn PageRequestService>,
    search_params: Query<SearchParams>,
    roles: ClaimRoles,
) -> BackendResult<Json<SearchResult<PageResponse>>> {
    Ok(Json(service.search(search_params.deref(), &roles)?))
}

/// Return all main menu pages
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "The pages", body=Vec<PageResponse>),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/main-menu")]
pub async fn main_menu(
    service: Data<dyn PageRequestService>,
    roles: ClaimRoles,
) -> BackendResult<Json<Vec<PageResponse>>> {
    Ok(Json(service.list_by_parent_id(0, &roles)?))
}

/// Creates a new page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "A new page is created"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/page/")]
pub async fn create(
    command: Json<CreatePageCommand>,
    service: Data<dyn PageCommandService>,
) -> BackendResult<HttpResponse> {
    service.create(&command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Sets the content of a page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "Content of the given page is set"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[put("/page/{id}/content")]
pub async fn set_content(
    id: Path<i32>,
    data: String,
    service: Data<dyn PageCommandService>,
) -> BackendResult<HttpResponse> {
    service.set_content(id.into_inner(), &data)?;
    Ok(HttpResponse::Ok().finish())
}

/// Returns an existing page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "The page", body=ExtendedPageResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/page/{id}")]
pub async fn find_by_id(
    id: Path<i32>,
    service: Data<dyn PageRequestService>,
    roles: ClaimRoles,
) -> BackendResult<Json<ExtendedPageResponse>> {
    Ok(Json(service.find_by_id(id.into_inner(), &roles)?))
}

/// Returns the default page if set
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "The page", body=ExtendedPageResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/default")]
pub async fn get_default(
    service: Data<dyn PageRequestService>,
    roles: ClaimRoles,
) -> BackendResult<Json<Option<ExtendedPageResponse>>> {
    Ok(Json(service.default(&roles)?))
}

/// Sets the default page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "The default page is set"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[put("/default/{id}")]
pub async fn put_default(
    id: Path<i32>,
    service: Data<dyn PageCommandService>,
) -> BackendResult<HttpResponse> {
    service.set_default(id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}

/// Returns page content
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "The page", content_type="text/plain"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/page/{id}/content")]
pub async fn content(
    id: Path<i32>,
    service: Data<dyn PageRequestService>,
    roles: ClaimRoles,
) -> BackendResult<HttpResponse> {
    let result = service.find_content_by_id(id.into_inner(), &roles)?;
    Ok(HttpResponse::Ok()
        .insert_header(("content-type", "text/plain"))
        .body(result))
}

/// Updates an existing page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "Page is updated"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[put("/page/{id}")]
pub async fn update(
    id: Path<i32>,
    command: Json<UpdatePageCommand>,
    service: Data<dyn PageCommandService>,
) -> BackendResult<HttpResponse> {
    service.update(id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Publish an existing page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "Page is published"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/page/{id}/publication")]
pub async fn publish(
    id: Path<i32>,
    command: Json<PublishPageCommand>,
    service: Data<dyn PageCommandService>,
) -> BackendResult<HttpResponse> {
    service.publish(id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Unpublish an existing page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "Page is unpublished"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[delete("/page/{id}/publication")]
pub async fn unpublish(
    id: Path<i32>,
    service: Data<dyn PageCommandService>,
) -> BackendResult<HttpResponse> {
    service.unpublish(id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}

/// Deletes an existing page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "Page is deleted"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[delete("/page/{id}")]
pub async fn delete(
    id: Path<i32>,
    service: Data<dyn PageCommandService>,
) -> BackendResult<HttpResponse> {
    service.delete(id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}
