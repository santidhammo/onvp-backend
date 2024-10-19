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
use crate::model::interface::responses::FacebookResponse;
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::services::definitions::request::FacebookRequestService;
use actix_web::get;
use actix_web::web::{Data, Json, Query};
use std::ops::Deref;

pub const CONTEXT: &str = "/api/facebook";

/// Search for members
///
/// Searches on first name and last name given the query, does not search through
/// inactive members nor through members who disallow public recognition (GDPR).
#[utoipa::path(
    context_path = CONTEXT,
    responses(
        (status = 200, description = "A list of matching members and work groups", body=SearchResult<FacebookResponse>),
        (status = 400, description = "Bad Request", body=Option<String>),
        (status = 401, description = "Unauthorized", body=Option<String>),
        (status = 500, description = "Internal Server Error", body=Option<String>)
    ),
    params(
        ("q" = String, Query, description = "Part of the first name or last name"),
        ("p" = Option<String>, Query, description = "The page offset to use (counting from 0)")
    )
)]
#[get("/search")]
pub async fn search(
    service: Data<dyn FacebookRequestService>,
    search_params: Query<SearchParams>,
) -> BackendResult<Json<SearchResult<FacebookResponse>>> {
    Ok(Json(service.search(search_params.deref())?))
}
