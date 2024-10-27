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

use crate::{BACKEND_SOURCE_CODE_URL, FRONTEND_SOURCE_CODE_URL};
use actix_web::get;
use actix_web::web::Json;
use serde::Serialize;
use std::sync::LazyLock;
use utoipa::ToSchema;

/// Shows the source code details of the frontend and backend
#[utoipa::path(
        context_path = "",
        responses(
            (status = 200, description = "Source code details")
        )
    )]
#[get("/")]
pub async fn details() -> Json<SourceCodeDetails> {
    Json(SOURCE_CODE_DETAILS.clone())
}

static SOURCE_CODE_DETAILS: LazyLock<SourceCodeDetails> = LazyLock::new(|| SourceCodeDetails {
    frontend_url: FRONTEND_SOURCE_CODE_URL.clone(),
    backend_url: BACKEND_SOURCE_CODE_URL.clone(),
});

#[derive(Serialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceCodeDetails {
    frontend_url: String,
    backend_url: String,
}
