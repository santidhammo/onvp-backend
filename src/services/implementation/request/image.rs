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
use crate::model::interface::responses::{ImageMetaDataResponse, ImageResponse};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::model::primitives::Role;
use crate::model::storage::entities::Image;
use crate::repositories::definitions::ImageRepository;
use crate::services::definitions::request::traits::RoleContainer;
use crate::services::definitions::request::ImageRequestService;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Read;
use std::sync::Arc;

pub struct Implementation {
    image_repository: Data<dyn ImageRepository>,
}

impl ImageRequestService for Implementation {
    fn find_by_id(
        &self,
        mut session: Session,
        image_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<ImageMetaDataResponse> {
        let known_roles = self
            .image_repository
            .find_associated_roles_by_id(&mut session, image_id)?;

        if !roles.has_role(Role::Operator) {
            let known_role_set: HashSet<Role> = HashSet::from_iter(known_roles.iter().cloned());
            if roles.set().is_disjoint(&known_role_set) {
                return Err(BackendError::forbidden());
            }
        }

        let image = self.image_repository.find_by_id(&mut session, image_id)?;

        if roles.has_role(Role::Operator) {
            Ok(ImageMetaDataResponse::from((&image, &known_roles)))
        } else {
            Ok(ImageMetaDataResponse::from((&image, &vec![])))
        }
    }

    fn find_content_by_id(
        &self,
        mut session: Session,
        image_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<ImageResponse> {
        let known_roles = self
            .image_repository
            .find_associated_roles_by_id(&mut session, image_id)?;
        if !roles.has_role(Role::Operator) {
            let known_role_set: HashSet<Role> = HashSet::from_iter(known_roles.iter().cloned());
            if roles.set().is_disjoint(&known_role_set) {
                return Err(BackendError::forbidden());
            }
        }

        let image = self.image_repository.find_by_id(&mut session, image_id)?;
        let content = Self::read_asset(&image.asset)?;
        Ok(ImageResponse {
            bytes: content,
            content_type: ContentType::png(),
        })
    }

    fn search(
        &self,
        mut session: Session,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<ImageMetaDataResponse>> {
        let term = params.term.clone().unwrap_or_default();
        let (total_count, page_size, results) =
            self.image_repository
                .search(&mut session, params.page_offset, &term)?;
        let rows: Vec<ImageMetaDataResponse> = results
            .iter()
            .map(|i| self.merge_roles(&mut session, i))
            .collect();
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
    fn read_asset(asset_id: &String) -> BackendResult<Vec<u8>> {
        let pb = crate::path_for_asset(&asset_id)?;
        let mut r = OpenOptions::new().read(true).open(&pb)?;
        let mut buf = Vec::new();
        let _ = r.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn merge_roles(&self, session: &mut Session, i: &Image) -> ImageMetaDataResponse {
        let roles = self
            .image_repository
            .find_associated_roles_by_id(session, i.id)
            .unwrap_or(vec![Role::Operator]);
        ImageMetaDataResponse::from((i, &roles))
    }
}

impl Injectable<ServiceDependencies, dyn ImageRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn ImageRequestService> {
        let implementation = Self {
            image_repository: dependencies.image_repository.clone(),
        };
        let arc: Arc<dyn ImageRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
