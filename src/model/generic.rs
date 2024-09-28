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

//! Contains general use components which may be used throughout the system

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// To be able to search for several kinds of records, a generic search parameter structure is
/// set up to be able to perform those search requests. The "query" or "q" parameter is used to
/// set up the text to query on, the "page_offset" or "p" parameter is used to indicate which
/// page offset to use.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Default)]
pub struct SearchParams {
    /// The string to search on
    #[serde(rename = "q")]
    pub query: Option<String>,

    /// The page to query
    #[serde(rename = "p", default)]
    pub page_offset: usize,
}

/// To facilitate the results of a search operation, a generic container is used which contains
/// the search results themselves, but also the total count of found results, the page offset
/// returned and the count of pages in the system.
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult<T: Serialize> {
    pub total_count: usize,
    pub page_offset: usize,
    pub page_count: usize,
    pub rows: Vec<T>,
}
