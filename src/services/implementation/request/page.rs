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
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::security::ClaimRoles;
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::generic::{search_helpers, Injectable};
use crate::model::interface::responses::{ExtendedPageResponse, PageResponse};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::model::primitives::Role;
use crate::repositories::definitions::PageRepository;
use crate::services::definitions::request::traits::RoleContainer;
use crate::services::definitions::request::PageRequestService;
use actix_web::web::Data;
use diesel::Connection;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Read;
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    page_repository: Data<dyn PageRepository>,
}

impl PageRequestService for Implementation {
    fn find_by_id(&self, page_id: i32, roles: &ClaimRoles) -> BackendResult<ExtendedPageResponse> {
        let mut conn = self.pool.get()?;
        conn.transaction::<ExtendedPageResponse, BackendError, _>(|conn| {
            let known_roles = self
                .page_repository
                .find_associated_roles_by_id(conn, page_id)?;

            if !roles.has_role(Role::Operator) {
                let known_role_set: HashSet<Role> = HashSet::from_iter(known_roles.iter().cloned());
                if roles.set().is_disjoint(&known_role_set) {
                    return Err(BackendError::forbidden());
                }
            }

            let page = self.page_repository.find_by_id(conn, page_id)?;

            if roles.has_role(Role::Operator) {
                Ok(ExtendedPageResponse::from((&page, &known_roles)))
            } else {
                Ok(ExtendedPageResponse::from((&page, &vec![])))
            }
        })
    }

    fn find_content_by_id(&self, page_id: i32, roles: &ClaimRoles) -> BackendResult<String> {
        let mut conn = self.pool.get()?;
        conn.transaction::<String, BackendError, _>(|conn| {
            let known_roles = self
                .page_repository
                .find_associated_roles_by_id(conn, page_id)?;
            if !roles.has_role(Role::Operator) {
                let known_role_set: HashSet<Role> = HashSet::from_iter(known_roles.iter().cloned());
                if roles.set().is_disjoint(&known_role_set) {
                    return Err(BackendError::forbidden());
                }
            }

            let page = self.page_repository.find_by_id(conn, page_id)?;
            let content = Self::read_asset(&page.content_asset)?;
            Ok(content)
        })
    }

    fn list_by_parent_id(
        &self,
        parent_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<Vec<PageResponse>> {
        let mut conn = self.pool.get()?;
        conn.transaction::<Vec<PageResponse>, BackendError, _>(|conn| {
            let pages = self
                .page_repository
                .list_by_parent_id(conn, parent_id, roles)?;
            Ok(pages.iter().map(PageResponse::from).collect())
        })
    }

    fn search(
        &self,
        params: &SearchParams,
        roles: &ClaimRoles,
    ) -> BackendResult<SearchResult<PageResponse>> {
        let mut conn = self.pool.get()?;
        let term = params.term.clone().unwrap_or_default();
        let (total_count, page_size, results) =
            self.page_repository
                .search(&mut conn, params.page_offset, &term, roles)?;
        let rows: Vec<PageResponse> = results.iter().map(PageResponse::from).collect();
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

impl Implementation {
    fn read_asset(asset_id: &String) -> BackendResult<String> {
        let pb = crate::path_for_asset_id(&asset_id)?;
        let mut r = OpenOptions::new().read(true).open(&pb)?;
        let mut buf = String::new();
        let _ = r.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

impl Injectable<(&DatabaseConnectionPool, &Data<dyn PageRepository>), dyn PageRequestService>
    for Implementation
{
    fn injectable(
        (pool, page_repository): (&DatabaseConnectionPool, &Data<dyn PageRepository>),
    ) -> Data<dyn PageRequestService> {
        let implementation = Self {
            pool: pool.clone(),
            page_repository: page_repository.clone(),
        };
        let arc: Arc<dyn PageRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
