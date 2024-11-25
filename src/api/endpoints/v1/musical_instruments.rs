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
use crate::model::interface::commands::{
    RegisterMusicalInstrumentCommand, UpdateMusicalInstrumentCommand,
};
use crate::model::interface::responses::MusicalInstrumentResponse;
use crate::services::definitions::command::MusicalInstrumentCommandService;
use crate::services::definitions::request::MusicalInstrumentRequestService;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, put, HttpResponse};

/// Registers a new musical instrument
#[utoipa::path(
    tag = "musical-instruments",
    responses(
        (status = 200, description = "A new musical instrument is registered"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/instrument/")]
pub async fn register(
    command: Json<RegisterMusicalInstrumentCommand>,
    service: Data<dyn MusicalInstrumentCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.register(session, &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Returns an existing musical instrument
#[utoipa::path(
    tag = "musical-instruments",
    responses(
        (status = 200, description = "The data of the musical instrument", body=MusicalInstrumentResponse),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[get("/instrument/{id}")]
pub async fn find_by_id(
    id: Path<i32>,
    service: Data<dyn MusicalInstrumentRequestService>,
    session: Session,
) -> BackendResult<Json<MusicalInstrumentResponse>> {
    Ok(Json(service.find_by_id(session, id.into_inner())?))
}

/// Updates a registered musical instrument
#[utoipa::path(
    tag = "musical-instruments",
    responses(
        (status = 200, description = "Musical instrument data is updated"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[put("/instrument/{id}")]
pub async fn update(
    id: Path<i32>,
    command: Json<UpdateMusicalInstrumentCommand>,
    service: Data<dyn MusicalInstrumentCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.update(session, id.into_inner(), &command)?;
    Ok(HttpResponse::Ok().finish())
}

/// Deletes a registered musical instrument
#[utoipa::path(
    tag = "musical-instruments",
    responses(
        (status = 200, description = "Musical instrument is removed from the database"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[delete("/instrument/{id}")]
pub async fn delete(
    id: Path<i32>,
    service: Data<dyn MusicalInstrumentCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.delete(session, id.into_inner())?;
    Ok(HttpResponse::Ok().finish())
}
