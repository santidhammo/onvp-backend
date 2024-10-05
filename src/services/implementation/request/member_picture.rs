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
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::generic::Injectable;
use crate::model::interface::client::UserClaims;
use crate::model::interface::responses::{ImageAssetIdResponse, ImageResponse};
use crate::model::primitives::Role;
use crate::model::storage::extended_entities::ExtendedMember;
use crate::repositories::traits::MemberRepository;
use crate::services::traits::request::MemberPictureRequestService;
use actix_web::http::header::ContentType;
use actix_web::web::Data;
use std::fs::OpenOptions;
use std::io::Read;
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    member_repository: Data<dyn MemberRepository>,
}

impl MemberPictureRequestService for Implementation {
    fn find_asset_by_member_id(
        &self,
        member_id: i32,
        user_claims: &UserClaims,
    ) -> BackendResult<Option<ImageResponse>> {
        let result = if user_claims.has_role(Role::Operator) {
            self.handle_retrieve_member_picture_operator(member_id)?
        } else if user_claims.has_role(Role::Member) {
            self.handle_retrieve_member_picture_dpia(member_id)?
        } else {
            return Err(BackendError::bad());
        };
        Ok(result.map(|it| ImageResponse {
            bytes: it,
            content_type: ContentType::png(),
        }))
    }

    fn find_asset_id_by_member_id(
        &self,
        member_id: i32,
        user_claims: &UserClaims,
    ) -> BackendResult<ImageAssetIdResponse> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut conn, member_id)?;
        let result = if user_claims.has_role(Role::Operator) {
            extended_member.picture_asset_id
        } else if user_claims.has_role(Role::Member) {
            if extended_member.allow_privacy_info_sharing {
                extended_member.picture_asset_id
            } else {
                None
            }
        } else {
            return Err(BackendError::bad());
        };

        Ok(ImageAssetIdResponse { asset_id: result })
    }
}

impl Implementation {
    fn handle_retrieve_member_picture_operator(
        &self,
        member_id: i32,
    ) -> BackendResult<Option<Vec<u8>>> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut conn, member_id)?;
        Self::read_member_picture_asset(extended_member)
    }

    fn handle_retrieve_member_picture_dpia(
        &self,
        member_id: i32,
    ) -> BackendResult<Option<Vec<u8>>> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut conn, member_id)?;
        if extended_member.allow_privacy_info_sharing {
            Self::read_member_picture_asset(extended_member)
        } else {
            Ok(None)
        }
    }

    fn read_member_picture_asset(
        extended_member: ExtendedMember,
    ) -> BackendResult<Option<Vec<u8>>> {
        if let Some(asset_id) = extended_member.picture_asset_id {
            Ok(Some(Self::read_asset(&asset_id)?))
        } else {
            Ok(None)
        }
    }

    fn read_asset(asset_id: &String) -> BackendResult<Vec<u8>> {
        let pb = crate::path_for_asset_id(&asset_id)?;
        let mut r = OpenOptions::new().read(true).open(&pb)?;
        let mut v = Vec::new();
        r.read_to_end(&mut v)?;
        Ok(v)
    }
}

impl
    Injectable<
        (&DatabaseConnectionPool, &Data<dyn MemberRepository>),
        dyn MemberPictureRequestService,
    > for Implementation
{
    fn injectable(
        (pool, member_repository): (&DatabaseConnectionPool, &Data<dyn MemberRepository>),
    ) -> Data<dyn MemberPictureRequestService> {
        let implementation = Self {
            pool: pool.clone(),
            member_repository: member_repository.clone(),
        };

        let arc: Arc<dyn MemberPictureRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
