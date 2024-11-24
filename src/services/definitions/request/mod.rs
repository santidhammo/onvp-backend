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
pub mod traits;

use crate::generic::result::BackendResult;
use crate::generic::security::ClaimRoles;
use crate::generic::storage::session::Session;
use crate::model::interface::client::UserClaims;
use crate::model::interface::requests::AuthorizationRequest;
use crate::model::interface::responses::{
    AuthorizationResponse, ExtendedPageResponse, FacebookResponse, ImageAssetIdResponse,
    ImageMetaDataResponse, ImageResponse, MemberAddressResponse, MemberPrivacyInfoSharingResponse,
    MemberResponse, PageResponse, WorkgroupResponse,
};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::model::primitives::{Role, RoleClass};
use crate::services::definitions::request::traits::RoleContainer;
use actix_web::cookie::Cookie;
use serde::Serialize;

/// Controls actions for data retrieval belonging to the setup process
pub trait SetupRequestService {
    /// Checks if setup mode should be activated
    fn should_setup(&self, session: Session) -> BackendResult<bool>;
}

/// Controls actions for authorization of members
pub trait AuthorizationRequestService {
    /// Performs the login procedure of a member
    fn login(
        &self,
        session: Session,
        login_data: &AuthorizationRequest,
    ) -> BackendResult<AuthorizationResponse>;

    /// Refreshes the member's current login, updates roles if refresh is due
    fn refresh(
        &self,
        session: Session,
        client_user_claims: &UserClaims,
        access_cookie: &Cookie<'static>,
        refresh_cookie: &Cookie<'static>,
    ) -> BackendResult<AuthorizationResponse>;

    /// Logs out a member
    fn logout(&self, session: Session) -> BackendResult<Vec<Cookie<'static>>>;
}

/// Controls actions for retrieval of role information
pub trait RoleRequestService {
    /// Lists all roles belonging to an id of a record belonging to the associated class
    fn list_by_id_and_class(
        &self,
        session: Session,
        id: i32,
        class: RoleClass,
    ) -> BackendResult<Vec<Role>>;
}

/// Controls actions for data retrieval belonging to members
pub trait MemberRequestService: SearchController<MemberResponse> {
    /// Finds a member by the member identifier
    fn find_by_id(&self, session: Session, member_id: i32) -> BackendResult<MemberResponse>;

    /// Finds a member address by the member identifier
    fn find_address_by_id(
        &self,
        session: Session,
        member_id: i32,
    ) -> BackendResult<MemberAddressResponse>;

    /// Finds a member privacy information sharing details record by member identifier
    fn find_privacy_info_sharing_by_id(
        &self,
        session: Session,
        member_id: i32,
    ) -> BackendResult<MemberPrivacyInfoSharingResponse>;

    /// Finds a member by the member's activation string
    fn find_by_activation_string(
        &self,
        session: Session,
        activation_string: &str,
    ) -> BackendResult<MemberResponse>;

    /// Lists the work groups associated to the member
    fn find_workgroups(
        &self,
        session: Session,
        member_id: i32,
    ) -> BackendResult<Vec<WorkgroupResponse>>;
}

pub trait MemberPictureRequestService {
    fn find_asset_by_member_id(
        &self,
        session: Session,
        member_id: i32,
        role_container: &dyn RoleContainer,
    ) -> BackendResult<Option<ImageResponse>>;

    fn find_asset_id_by_member_id(
        &self,
        session: Session,
        member_id: i32,
        role_container: &dyn RoleContainer,
    ) -> BackendResult<ImageAssetIdResponse>;
}

/// Controls actions for data retrieval belonging to work groups
pub trait WorkgroupRequestService: SearchController<WorkgroupResponse> {
    /// Find the work group
    ///
    /// Finds the work group using the identifier of the work group
    fn find_by_id(&self, session: Session, id: i32) -> BackendResult<WorkgroupResponse>;

    /// Find all the members belonging to the work group
    ///
    /// Finds the work group members using the identifier of the work group
    fn find_members_by_id(&self, session: Session, id: i32) -> BackendResult<Vec<MemberResponse>>;

    /// Search for all members available to the work group
    ///
    /// Searches the members available to the work group (thus the members *not* in
    /// the work group) using the work group identifier and the search parameters
    fn available_members_search(
        &self,
        session: Session,
        workgroup_id: i32,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<MemberResponse>>;
}

/// Controls actions for data retrieval belonging to the facebook
pub trait FacebookRequestService: SearchController<FacebookResponse> {}

/// Controls actions for data retrieval belonging to pages
pub trait PageRequestService {
    /// Finds a page using the identifier
    fn find_by_id(
        &self,
        session: Session,
        page_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<ExtendedPageResponse>;

    /// Finds a page's content using the identifier
    fn find_content_by_id(
        &self,
        session: Session,
        page_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<String>;

    /// Returns the default page, if there is a default page
    fn default(
        &self,
        session: Session,
        roles: &ClaimRoles,
    ) -> BackendResult<Option<ExtendedPageResponse>>;

    /// Lists all page's by parent identifier
    fn list_by_parent_id(
        &self,
        session: Session,
        parent_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<Vec<PageResponse>>;

    /// Searches pages by page title and allowed roles
    fn search(
        &self,
        session: Session,
        params: &SearchParams,
        roles: &ClaimRoles,
    ) -> BackendResult<SearchResult<PageResponse>>;
}

/// Controls actions for data retrieval belonging to images
pub trait ImageRequestService {
    /// Finds an image using the identifier
    fn find_by_id(
        &self,
        session: Session,
        image_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<ImageMetaDataResponse>;

    /// Finds an images content using the identifier
    fn find_content_by_id(
        &self,
        session: Session,
        page_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<ImageResponse>;

    /// Searches pages by page title and allowed roles
    fn search(
        &self,
        session: Session,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<ImageMetaDataResponse>>;
}

pub trait SearchController<T> {
    fn search(&self, session: Session, params: &SearchParams) -> BackendResult<SearchResult<T>>
    where
        T: Serialize;
}
