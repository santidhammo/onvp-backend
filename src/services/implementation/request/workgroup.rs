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
use crate::generic::search_helpers::create_like_string;
use crate::generic::storage::session::Session;
use crate::generic::{search_helpers, Injectable};
use crate::injection::ServiceDependencies;
use crate::model::interface::responses::{MemberResponse, WorkgroupResponse};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::repositories::definitions::WorkgroupRepository;
use crate::services::definitions::request::{SearchController, WorkgroupRequestService};
use actix_web::web::Data;
use serde::Serialize;
use std::sync::Arc;

pub struct Implementation {
    workgroup_repository: Data<dyn WorkgroupRepository>,
}

impl WorkgroupRequestService for Implementation {
    fn find_by_id(&self, mut session: Session, id: i32) -> BackendResult<WorkgroupResponse> {
        self.workgroup_repository
            .find_by_id(&mut session, id)
            .map(|w| WorkgroupResponse::from(&w))
    }

    fn find_members_by_id(
        &self,
        mut session: Session,
        id: i32,
    ) -> BackendResult<Vec<MemberResponse>> {
        self.workgroup_repository
            .find_members_by_id(&mut session, id)
            .map(|v| v.iter().map(|w| MemberResponse::from(w)).collect())
    }

    fn available_members_search(
        &self,
        mut session: Session,
        workgroup_id: i32,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<MemberResponse>> {
        let term = create_like_string(params.term.clone().unwrap_or_default());
        let (total_count, page_size, results) = self
            .workgroup_repository
            .available_members_search(&mut session, workgroup_id, params.page_offset, &term)?;
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

impl SearchController<WorkgroupResponse> for Implementation {
    fn search(
        &self,
        mut session: Session,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<WorkgroupResponse>>
    where
        WorkgroupResponse: Serialize,
    {
        let term = params.term.clone().unwrap_or("".to_owned());
        let (total_count, page_size, results) =
            self.workgroup_repository
                .search(&mut session, params.page_offset, &term)?;
        let rows: Vec<WorkgroupResponse> = results.iter().map(WorkgroupResponse::from).collect();
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

impl Injectable<ServiceDependencies, dyn WorkgroupRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn WorkgroupRequestService> {
        let implementation = Self {
            workgroup_repository: dependencies.workgroup_repository.clone(),
        };
        let arc: Arc<dyn WorkgroupRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
