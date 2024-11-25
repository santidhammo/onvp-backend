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
use crate::generic::storage::session::Session;
use crate::generic::{search_helpers, Injectable};
use crate::injection::ServiceDependencies;
use crate::model::interface::responses::{ExtendedPageResponse, PageResponse};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::model::primitives::Role;
use crate::model::traits::RoleContainer;
use crate::repositories::definitions::{PageRepository, PropertiesRepository};
use crate::services::definitions::request::PageRequestService;
use actix_web::web::Data;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Read;
use std::sync::Arc;

pub struct Implementation {
    page_repository: Data<dyn PageRepository>,
    properties_repository: Data<dyn PropertiesRepository>,
}

impl PageRequestService for Implementation {
    fn find_by_id(
        &self,
        mut session: Session,
        page_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<ExtendedPageResponse> {
        let known_roles = self
            .page_repository
            .find_associated_roles_by_id(&mut session, page_id)?;

        if !roles.has_role(Role::Operator) {
            let known_role_set: HashSet<Role> = HashSet::from_iter(known_roles.iter().cloned());
            if roles.set().is_disjoint(&known_role_set) {
                return Err(BackendError::forbidden());
            }
        }

        let page = self.page_repository.find_by_id(&mut session, page_id)?;

        if roles.has_role(Role::Operator) {
            Ok(ExtendedPageResponse::from((&page, &known_roles)))
        } else {
            Ok(ExtendedPageResponse::from((&page, &vec![])))
        }
    }

    fn find_content_by_id(
        &self,
        mut session: Session,
        page_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<String> {
        let known_roles = self
            .page_repository
            .find_associated_roles_by_id(&mut session, page_id)?;
        if !roles.has_role(Role::Operator) {
            let known_role_set: HashSet<Role> = HashSet::from_iter(known_roles.iter().cloned());
            if roles.set().is_disjoint(&known_role_set) {
                return Err(BackendError::forbidden());
            }
        }

        let page = self.page_repository.find_by_id(&mut session, page_id)?;
        let content = Self::read_asset(&page.content_asset)?;
        Ok(content)
    }

    fn default(
        &self,
        mut session: Session,
        roles: &ClaimRoles,
    ) -> BackendResult<Option<ExtendedPageResponse>> {
        let maybe_page_id = self
            .properties_repository
            .maybe_int_property(&mut session, "default-page");
        if let Some(page_id) = maybe_page_id {
            Ok(Some(self.find_by_id(session, page_id, roles)?))
        } else {
            Ok(None)
        }
    }

    fn list_by_parent_id(
        &self,
        mut session: Session,
        parent_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<Vec<PageResponse>> {
        let pages = self
            .page_repository
            .list_by_parent_id(&mut session, parent_id, roles)?;
        Ok(pages.iter().map(PageResponse::from).collect())
    }

    fn search(
        &self,
        mut session: Session,
        params: &SearchParams,
        roles: &ClaimRoles,
    ) -> BackendResult<SearchResult<PageResponse>> {
        let term = params.term.clone().unwrap_or_default();
        let (total_count, page_size, results) =
            self.page_repository
                .search(&mut session, params.page_offset, &term, roles)?;
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
        let pb = crate::path_for_asset(&asset_id)?;
        if pb.exists() {
            let mut r = OpenOptions::new().read(true).open(&pb)?;
            let mut buf = String::new();
            let _ = r.read_to_string(&mut buf)?;
            Ok(buf)
        } else {
            Ok("".to_owned())
        }
    }
}

impl Injectable<ServiceDependencies, dyn PageRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn PageRequestService> {
        let implementation = Self {
            page_repository: dependencies.page_repository.clone(),
            properties_repository: dependencies.properties_repository.clone(),
        };
        let arc: Arc<dyn PageRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
