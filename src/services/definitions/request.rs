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
use crate::model::interface::client::UserClaims;
use crate::model::interface::requests::AuthorizationRequest;
use crate::model::interface::responses::{
    AuthorizationResponse, ImageAssetIdResponse, ImageResponse, MemberAddressResponse,
    MemberResponse, WorkgroupResponse,
};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::model::primitives::{Role, RoleClass};
use actix_web::cookie::Cookie;
use serde::Serialize;

/// Controls actions for data retrieval belonging to the setup process
pub trait SetupRequestService {
    /// Checks if setup mode should be activated
    fn should_setup(&self) -> BackendResult<bool>;
}

/// Controls actions for authorization of members
pub trait AuthorizationRequestService {
    /// Performs the login procedure of a member
    fn login(&self, login_data: &AuthorizationRequest) -> BackendResult<AuthorizationResponse>;

    /// Refreshes the member's current login, updates roles if refresh is due
    fn refresh(
        &self,
        client_user_claims: &UserClaims,
        access_cookie: &Cookie<'static>,
        refresh_cookie: &Cookie<'static>,
    ) -> BackendResult<AuthorizationResponse>;

    /// Logs out a member
    fn logout(&self) -> BackendResult<Vec<Cookie<'static>>>;
}

/// Controls actions for retrieval of role information
pub trait RoleRequestService {
    /// Lists all roles belonging to an id of a record belonging to the associated class
    fn list_by_id_and_class(&self, id: i32, class: RoleClass) -> BackendResult<Vec<Role>>;
}

/// Controls actions for data retrieval belonging to members
pub trait MemberRequestService: SearchController<MemberResponse> {
    /// Finds a member by the member identifier
    fn find_by_id(&self, member_id: i32) -> BackendResult<MemberResponse>;

    /// Finds a member address by the member identifier
    fn find_address_by_id(&self, member_id: i32) -> BackendResult<MemberAddressResponse>;

    /// Finds a member by the member's activation string
    fn find_by_activation_string(&self, activation_string: &str) -> BackendResult<MemberResponse>;

    /// Lists the work groups associated to the member
    fn find_workgroups(&self, member_id: i32) -> BackendResult<Vec<WorkgroupResponse>>;
}

pub trait MemberPictureRequestService {
    fn find_asset_by_member_id(
        &self,
        member_id: i32,
        user_claims: &UserClaims,
    ) -> BackendResult<Option<ImageResponse>>;

    fn find_asset_id_by_member_id(
        &self,
        member_id: i32,
        user_claims: &UserClaims,
    ) -> BackendResult<ImageAssetIdResponse>;
}

/// Controls actions for data retrieval belonging to work groups
pub trait WorkgroupRequestService: SearchController<WorkgroupResponse> {
    /// Find the work group
    ///
    /// Finds the work group using the identifier of the work group
    fn find_by_id(&self, id: i32) -> BackendResult<WorkgroupResponse>;

    /// Find all the members belonging to the work group
    ///
    /// Finds the work group members using the identifier of the work group
    fn find_members_by_id(&self, id: i32) -> BackendResult<Vec<MemberResponse>>;

    /// Search for all members available to the work group
    ///
    /// Searches the members available to the work group (thus the members *not* in
    /// the work group) using the work group identifier and the search parameters
    fn available_members_search(
        &self,
        workgroup_id: i32,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<MemberResponse>>;
}

pub trait SearchController<T> {
    fn search(&self, params: &SearchParams) -> BackendResult<SearchResult<T>>
    where
        T: Serialize;
}
