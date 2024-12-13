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
use crate::model::interface::commands::SendMailCommand;
use crate::services::definitions::command::MailingCommandService;
use actix_web::web::{Data, Json};
use actix_web::{post, HttpResponse};

/// Sends an email based on an email template
#[utoipa::path(
    tag = "mailing",
    responses(
        (status = 200, description = "An email is sent"),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    )
)]
#[post("/send")]
pub async fn send(
    command: Json<SendMailCommand>,
    service: Data<dyn MailingCommandService>,
    session: Session,
) -> BackendResult<HttpResponse> {
    service.send(session, &command)?;
    Ok(HttpResponse::Ok().finish())
}
