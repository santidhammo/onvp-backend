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
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::generic::{search_helpers, Injectable};
use crate::model::interface::responses::{MemberResponse, WorkgroupResponse};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::repositories::definitions::WorkgroupRepository;
use crate::services::definitions::request::{SearchController, WorkgroupRequestService};
use actix_web::web::Data;
use serde::Serialize;
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    workgroup_repository: Data<dyn WorkgroupRepository>,
}

impl WorkgroupRequestService for Implementation {
    fn find_by_id(&self, id: i32) -> BackendResult<WorkgroupResponse> {
        let mut conn = self.pool.get()?;
        self.workgroup_repository
            .find_by_id(&mut conn, id)
            .map(|w| WorkgroupResponse::from(&w))
    }

    fn find_members_by_id(&self, id: i32) -> BackendResult<Vec<MemberResponse>> {
        let mut conn = self.pool.get()?;
        self.workgroup_repository
            .find_members_by_id(&mut conn, id)
            .map(|v| v.iter().map(|w| MemberResponse::from(w)).collect())
    }

    fn available_members_search(
        &self,
        workgroup_id: i32,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<MemberResponse>> {
        let term = create_like_string(params.term.clone().unwrap_or_default());
        let mut conn = self.pool.get()?;
        let (total_count, page_size, results) = self
            .workgroup_repository
            .available_members_search(&mut conn, workgroup_id, params.page_offset, &term)?;
        Ok(SearchResult {
            total_count,
            page_offset: params.page_offset,
            page_count: search_helpers::calculate_page_count(page_size, total_count),
            rows: results.iter().map(MemberResponse::from).collect(),
        })
    }
}

impl SearchController<WorkgroupResponse> for Implementation {
    fn search(&self, params: &SearchParams) -> BackendResult<SearchResult<WorkgroupResponse>>
    where
        WorkgroupResponse: Serialize,
    {
        let mut conn = self.pool.get()?;
        let term = params.term.clone().unwrap_or("".to_owned());
        let (total_count, page_size, results) =
            self.workgroup_repository
                .search(&mut conn, params.page_offset, &term)?;

        Ok(SearchResult {
            total_count,
            page_offset: params.page_offset,
            page_count: search_helpers::calculate_page_count(page_size, total_count),
            rows: results.iter().map(WorkgroupResponse::from).collect(),
        })
    }
}

impl
    Injectable<
        (&DatabaseConnectionPool, &Data<dyn WorkgroupRepository>),
        dyn WorkgroupRequestService,
    > for Implementation
{
    fn injectable(
        (pool, workgroup_repository): (&DatabaseConnectionPool, &Data<dyn WorkgroupRepository>),
    ) -> Data<dyn WorkgroupRequestService> {
        let implementation = Self {
            pool: pool.clone(),
            workgroup_repository: workgroup_repository.clone(),
        };
        let arc: Arc<dyn WorkgroupRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
