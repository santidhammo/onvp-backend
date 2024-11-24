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
use std::sync::Arc;

use crate::generic::search_helpers::create_like_string;
use crate::generic::storage::session::Session;
use crate::injection::ServiceDependencies;
use crate::model::interface::responses::{
    MemberAddressResponse, MemberPrivacyInfoSharingResponse, MemberResponse, WorkgroupResponse,
};
use crate::services::definitions::request::{MemberRequestService, SearchController};
use actix_web::web::Data;

pub struct Implementation {
    member_repository: Data<dyn MemberRepository>,
}

impl MemberRequestService for Implementation {
    fn find_by_id(&self, mut session: Session, member_id: i32) -> BackendResult<MemberResponse> {
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut session, member_id)?;
        Ok(MemberResponse::from(&extended_member))
    }

    fn find_address_by_id(
        &self,
        mut session: Session,
        member_id: i32,
    ) -> BackendResult<MemberAddressResponse> {
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut session, member_id)?;
        Ok(MemberAddressResponse::from(&extended_member))
    }

    fn find_privacy_info_sharing_by_id(
        &self,
        mut session: Session,
        member_id: i32,
    ) -> BackendResult<MemberPrivacyInfoSharingResponse> {
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut session, member_id)?;
        Ok(MemberPrivacyInfoSharingResponse::from(&extended_member))
    }

    fn find_by_activation_string(
        &self,
        mut session: Session,
        activation_string: &str,
    ) -> BackendResult<MemberResponse> {
        let extended_member = self
            .member_repository
            .find_extended_by_activation_string(&mut session, activation_string)?;
        Ok(MemberResponse::from(&extended_member))
    }

    fn find_workgroups(
        &self,
        mut session: Session,
        member_id: i32,
    ) -> BackendResult<Vec<WorkgroupResponse>> {
        let workgroups = self
            .member_repository
            .find_workgroups(&mut session, member_id)?;
        Ok(workgroups.iter().map(WorkgroupResponse::from).collect())
    }
}

impl SearchController<MemberResponse> for Implementation {
    fn search(
        &self,
        mut session: Session,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<MemberResponse>> {
        let term = create_like_string(params.term.clone().unwrap_or_default());
        let (total_count, page_size, results) =
            self.member_repository
                .search(&mut session, params.page_offset, &term)?;
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

impl Injectable<ServiceDependencies, dyn MemberRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn MemberRequestService> {
        let implementation = Self {
            member_repository: dependencies.member_repository.clone(),
        };

        let arc: Arc<dyn MemberRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
