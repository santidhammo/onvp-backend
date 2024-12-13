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
use crate::generic::storage::session::Session;
use crate::model::interface::commands::{CreateMailTemplateCommand, UpdateMailTemplateCommand};
use crate::model::interface::responses::{MailTemplateNameResponse, MailTemplateResponse};
use crate::services::definitions::command::MailTemplateCommandService;
use crate::services::definitions::request::MailTemplateRequestService;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};

/// Lists all the email templates
#[utoipa::path(
    tag = "mail-templates",
    responses(
        (status = 200, description = "A list of email templates", body=Vec<MailTemplateNameResponse>),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/list")]
pub async fn list(
    service: Data<dyn MailTemplateRequestService>,
    session: Session,
) -> BackendResult<Json<Vec<MailTemplateNameResponse>>> {
    Ok(Json(service.list(session)?))
}

/// Creates a new email template
#[utoipa::path(
    tag = "mail-templates",
    responses(
        (status = 200, description = "A new email template is created"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/template/")]
pub async fn create(
    command: Json<CreateMailTemplateCommand>,
    service: Data<dyn MailTemplateCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.create(session, &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Returns an existing email template
#[utoipa::path(
    tag = "mail-templates",
    responses(
        (status = 200, description = "The data of the email template", body=MailTemplateResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/template/{id}")]
pub async fn find_by_id(
    id: Path<i32>,
    service: Data<dyn MailTemplateRequestService>,
    session: Session,
) -> BackendResult<Json<MailTemplateResponse>> {
    Ok(Json(service.find_by_id(session, id.into_inner())?))
}

/// Updates a registered email template
#[utoipa::path(
    tag = "mail-templates",
    responses(
        (status = 200, description = "Email template data is updated"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[put("/template/{id}")]
pub async fn update(
    id: Path<i32>,
    command: Json<UpdateMailTemplateCommand>,
    service: Data<dyn MailTemplateCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.update(session, id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Deletes a registered email template
#[utoipa::path(
    tag = "mail-templates",
    responses(
        (status = 200, description = "Email template is removed from the database"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[delete("/template/{id}")]
pub async fn delete(
    id: Path<i32>,
    service: Data<dyn MailTemplateCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.delete(session, id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}
