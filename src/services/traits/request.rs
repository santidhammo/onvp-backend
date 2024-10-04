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
use serde::Serialize;

use crate::model::interface::responses::MemberResponse;
use crate::model::interface::search::{SearchParams, SearchResult};

/// Controls actions for data retrieval belonging to the setup process
pub trait SetupRequestService {
    fn should_setup(&self) -> BackendResult<bool>;
}

/// Controls actions for data retrieval belonging to members
pub trait MemberRequestService: SearchController<MemberResponse> {
    fn find(&self, member_id: i32) -> BackendResult<MemberResponse>;
}

pub trait SearchController<T> {
    fn search(&self, params: &SearchParams) -> BackendResult<SearchResult<T>>
    where
        T: Serialize;
}
