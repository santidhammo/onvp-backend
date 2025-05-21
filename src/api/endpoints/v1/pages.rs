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
use crate::generic::security::ClaimRoles;
use crate::generic::storage::session::Session;
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
        ("q" = String, Query, description = "Part of the title of the page"),
        ("p" = Option<String>, Query, description = "The page offset to use (counting from 0)")
    )
)]
#[get("/search")]
pub async fn search(
    service: Data<dyn PageRequestService>,
    search_params: Query<SearchParams>,
    roles: ClaimRoles,
    session: Session,
) -> BackendResult<Json<SearchResult<PageResponse>>> {
    Ok(Json(service.search(
        session,
        search_params.deref(),
        &roles,
    )?))
}

/// Return all sub menu entries of a given page, if there are any
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "The pages", body=Vec<PageResponse>),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/sub-menu/{id}")]
pub async fn sub_menu(
    id: Path<i32>,
    service: Data<dyn PageRequestService>,
    roles: ClaimRoles,
    session: Session,
) -> BackendResult<Json<Vec<PageResponse>>> {
    Ok(Json(service.list_by_parent_id(
        session,
        id.into_inner(),
        &roles,
    )?))
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
    session: Session,
) -> BackendResult<Json<Vec<PageResponse>>> {
    Ok(Json(service.list_by_parent_id(session, 0, &roles)?))
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
    session: Session,
) -> BackendResult<HttpResponse> {
    service.create(session, &command)?;
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
    session: Session,
) -> BackendResult<HttpResponse> {
    service.set_content(session, id.into_inner(), &data)?;
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
    session: Session,
) -> BackendResult<Json<ExtendedPageResponse>> {
    Ok(Json(service.find_by_id(
        session,
        id.into_inner(),
        &roles,
    )?))
}

/// Finds the events for the upcoming months
#[utoipa::path(
    tag = "events",
    responses(
        (status = 200, description = "The events", body=[PageResponse]),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/events")]
pub async fn events(
    service: Data<dyn PageRequestService>,
    roles: ClaimRoles,
    session: Session,
) -> BackendResult<Json<Vec<PageResponse>>> {
    Ok(Json(service.events(session, &roles)?))
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
    session: Session,
) -> BackendResult<Json<Option<ExtendedPageResponse>>> {
    Ok(Json(service.default(session, &roles)?))
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
    session: Session,
) -> BackendResult<HttpResponse> {
    service.set_default(session, id.into_inner())?;
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
    session: Session,
) -> BackendResult<HttpResponse> {
    let result = service.find_content_by_id(session, id.into_inner(), &roles)?;
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
    session: Session,
) -> BackendResult<HttpResponse> {
    service.update(session, id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Updates the order of an existing page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "Page order is updated"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[put("/page/{id}/order")]
pub async fn set_order(
    id: Path<i32>,
    number: Json<i32>,
    service: Data<dyn PageCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.set_order(session, id.into_inner(), number.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}

/// Sets the parent page for a page, a parent page can only be set if the parent page
/// does not have a parent.
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "Parent page is set"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[put("/page/{id}/parent")]
pub async fn set_parent(
    id: Path<i32>,
    parent_id: Json<i32>,
    service: Data<dyn PageCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.set_or_unset_parent_id(session, id.into_inner(), Some(parent_id.into_inner()))?;
    Ok(HttpResponse::Ok().finish())
}

/// Unsets the parent page for a page
#[utoipa::path(
    tag = "pages",
    responses(
        (status = 200, description = "Parent page is unset"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[delete("/page/{id}/parent")]
pub async fn unset_parent(
    id: Path<i32>,
    service: Data<dyn PageCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.set_or_unset_parent_id(session, id.into_inner(), None)?;
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
    session: Session,
) -> BackendResult<HttpResponse> {
    service.publish(session, id.into_inner(), &command)?;
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
    session: Session,
) -> BackendResult<HttpResponse> {
    service.unpublish(session, id.into_inner())?;
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
    session: Session,
) -> BackendResult<HttpResponse> {
    service.delete(session, id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}
