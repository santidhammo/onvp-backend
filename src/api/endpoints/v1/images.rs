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
use crate::model::interface::commands::{ImageUploadCommand, PublishImageCommand};
use crate::model::interface::responses::ImageMetaDataResponse;
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::services::definitions::command::ImageCommandService;
use crate::services::definitions::request::ImageRequestService;
use actix_web::web::{Bytes, Data, Json, Path, Query};
use actix_web::{delete, get, post, HttpResponse};
use serde::Deserialize;
use std::ops::Deref;
use utoipa::ToSchema;

/// Search for images
///
/// Searches on titles matching the given query.
#[utoipa::path(
    tag = "images",
    responses(
        (status = 200, description = "A list of matching images", body=SearchResult<ImageMetaDataResponse>),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error", body=[String])
    ),
    params(
        ("q" = String, Query, description = "Part of the first name, last name and/or email address"),
        ("p" = Option<String>, Query, description = "The image offset to use (counting from 0)")
    )
)]
#[get("/search")]
pub async fn search(
    service: Data<dyn ImageRequestService>,
    search_params: Query<SearchParams>,
) -> BackendResult<Json<SearchResult<ImageMetaDataResponse>>> {
    Ok(Json(service.search(search_params.deref())?))
}

/// Creates a new image
#[utoipa::path(
    request_body(content(("image/png"), ("image/jpg"))),
    tag = "images",
    responses(
        (status = 200, description = "A new image is created"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    ),
    params(
        ("title" = String, Query, description = "The title of the image to upload"),
    )
)]
#[post("/image/")]
pub async fn upload(
    service: Data<dyn ImageCommandService>,
    upload_params: Query<ImageUploadParams>,
    data: Bytes,
) -> BackendResult<Json<String>> {
    let r = upload_params.deref();
    let command = ImageUploadCommand {
        title: r.title.clone(),
        data,
    };
    Ok(Json(service.upload(&command)?))
}

/// Returns an existing image
#[utoipa::path(
    tag = "images",
    responses(
        (status = 200, description = "The image metadata", body=ImageMetaDataResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/image/{id}")]
pub async fn find_by_id(
    id: Path<i32>,
    service: Data<dyn ImageRequestService>,
    roles: ClaimRoles,
) -> BackendResult<Json<ImageMetaDataResponse>> {
    Ok(Json(service.find_by_id(id.into_inner(), &roles)?))
}

/// Returns an image asset
#[utoipa::path(
    tag = "images",
    responses(
        (status = 200, description = "The image metadata", content_type="image/png"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/asset/{id}.png")]
pub async fn asset(
    id: Path<i32>,
    service: Data<dyn ImageRequestService>,
    roles: ClaimRoles,
) -> BackendResult<HttpResponse> {
    let result = service.find_content_by_id(id.into_inner(), &roles)?;
    Ok(HttpResponse::Ok()
        .insert_header(result.content_type)
        .body(Bytes::from(result.bytes)))
}

/// Publish an existing image
#[utoipa::path(
    tag = "images",
    responses(
        (status = 200, description = "Page is published"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/image/{id}/publication")]
pub async fn publish(
    id: Path<i32>,
    command: Json<PublishImageCommand>,
    service: Data<dyn ImageCommandService>,
) -> BackendResult<HttpResponse> {
    service.publish(id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Unpublish an existing image
#[utoipa::path(
    tag = "images",
    responses(
        (status = 200, description = "Page is unpublished"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[delete("/image/{id}/publication")]
pub async fn unpublish(
    id: Path<i32>,
    service: Data<dyn ImageCommandService>,
) -> BackendResult<HttpResponse> {
    service.unpublish(id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}

/// Deletes an existing image
#[utoipa::path(
    tag = "images",
    responses(
        (status = 200, description = "Page is deleted"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[delete("/image/{id}")]
pub async fn delete(
    id: Path<i32>,
    service: Data<dyn ImageCommandService>,
) -> BackendResult<HttpResponse> {
    service.delete(id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct ImageUploadParams {
    pub title: String,
}
