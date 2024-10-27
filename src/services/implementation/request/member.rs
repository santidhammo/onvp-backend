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
use crate::generic::{search_helpers, Injectable};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::repositories::definitions::MemberRepository;

use crate::generic::search_helpers::create_like_string;
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::model::interface::responses::{
    MemberAddressResponse, MemberPrivacyInfoSharingResponse, MemberResponse, WorkgroupResponse,
};
use crate::services::definitions::request::{MemberRequestService, SearchController};
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    member_repository: Data<dyn MemberRepository>,
}

impl MemberRequestService for Implementation {
    fn find_by_id(&self, member_id: i32) -> BackendResult<MemberResponse> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut conn, member_id)?;
        Ok(MemberResponse::from(&extended_member))
    }

    fn find_address_by_id(&self, member_id: i32) -> BackendResult<MemberAddressResponse> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut conn, member_id)?;
        Ok(MemberAddressResponse::from(&extended_member))
    }

    fn find_privacy_info_sharing_by_id(
        &self,
        member_id: i32,
    ) -> BackendResult<MemberPrivacyInfoSharingResponse> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut conn, member_id)?;
        Ok(MemberPrivacyInfoSharingResponse::from(&extended_member))
    }

    fn find_by_activation_string(&self, activation_string: &str) -> BackendResult<MemberResponse> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_activation_string(&mut conn, activation_string)?;
        Ok(MemberResponse::from(&extended_member))
    }

    fn find_workgroups(&self, member_id: i32) -> BackendResult<Vec<WorkgroupResponse>> {
        let mut conn = self.pool.get()?;
        let workgroups = self
            .member_repository
            .find_workgroups(&mut conn, member_id)?;
        Ok(workgroups.iter().map(WorkgroupResponse::from).collect())
    }
}

impl SearchController<MemberResponse> for Implementation {
    fn search(&self, params: &SearchParams) -> BackendResult<SearchResult<MemberResponse>> {
        let term = create_like_string(params.term.clone().unwrap_or_default());
        let mut conn = self.pool.get()?;
        let (total_count, page_size, results) =
            self.member_repository
                .search(&mut conn, params.page_offset, &term)?;
        let rows: Vec<MemberResponse> = results.iter().map(MemberResponse::from).collect();
        let row_len = rows.len();
        Ok(SearchResult {
            total_count,
            page_offset: params.page_offset,
            page_count: search_helpers::calculate_page_count(page_size, total_count),
            rows,
            start: params.page_offset * page_size,
            end: (params.page_offset * page_size) + row_len,
        })
    }
}

impl Injectable<(&DatabaseConnectionPool, &Data<dyn MemberRepository>), dyn MemberRequestService>
    for Implementation
{
    fn injectable(
        (pool, member_repository): (&DatabaseConnectionPool, &Data<dyn MemberRepository>),
    ) -> Data<dyn MemberRequestService> {
        let implementation = Self {
            pool: pool.clone(),
            member_repository: member_repository.clone(),
        };

        let arc: Arc<dyn MemberRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
