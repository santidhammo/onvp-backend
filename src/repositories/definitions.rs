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
use crate::generic::storage::database::DatabaseConnection;
use crate::model::primitives::Role;
use crate::model::storage::entities::Workgroup;
use crate::model::storage::extended_entities::ExtendedMember;

pub trait MemberRepository {
    fn create_inactive(
        &self,
        conn: &mut DatabaseConnection,
        member_extended: &ExtendedMember,
    ) -> BackendResult<i32>;

    fn find_extended_by_id(
        &self,
        conn: &mut DatabaseConnection,
        id: i32,
    ) -> BackendResult<ExtendedMember>;

    fn find_extended_by_activation_string(
        &self,
        conn: &mut DatabaseConnection,
        activation_string: &str,
    ) -> BackendResult<ExtendedMember>;

    fn find_extended_by_email_address(
        &self,
        conn: &mut DatabaseConnection,
        email_address: &str,
    ) -> BackendResult<ExtendedMember>;

    fn find_workgroups(
        &self,
        conn: &mut DatabaseConnection,
        id: i32,
    ) -> BackendResult<Vec<Workgroup>>;

    fn save(&self, conn: &mut DatabaseConnection, member: ExtendedMember) -> BackendResult<()>;

    fn count_members_with_role(
        &self,
        conn: &mut DatabaseConnection,
        role: Role,
    ) -> BackendResult<usize>;

    fn activate_by_id(&self, conn: &mut DatabaseConnection, member_id: i32) -> BackendResult<()>;

    fn search(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<ExtendedMember>)>;

    fn unregister(&self, conn: &mut DatabaseConnection, member_id: i32) -> BackendResult<()>;
}

pub trait WorkgroupRepository {
    fn register(&self, conn: &mut DatabaseConnection, workgroup: Workgroup) -> BackendResult<i32>;

    fn find_by_id(&self, conn: &mut DatabaseConnection, id: i32) -> BackendResult<Workgroup>;

    fn save(&self, conn: &mut DatabaseConnection, workgroup: Workgroup) -> BackendResult<()>;

    fn search(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<Workgroup>)>;

    fn unregister(&self, conn: &mut DatabaseConnection, workgroup_id: i32) -> BackendResult<()>;

    fn find_members_by_id(
        &self,
        conn: &mut DatabaseConnection,
        workgroup_id: i32,
    ) -> BackendResult<Vec<ExtendedMember>>;

    fn associate_member_to_workgroup(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
        workgroup_id: i32,
    ) -> BackendResult<()>;
    fn dissociate_member_from_workgroup(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
        workgroup_id: i32,
    ) -> BackendResult<()>;

    fn available_members_search(
        &self,
        conn: &mut DatabaseConnection,
        workgroup_id: i32,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<ExtendedMember>)>;
}

pub trait MemberPictureRepository {
    fn save_by_member_id(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
        picture_asset_id: &str,
    ) -> BackendResult<()>;
}

pub trait MemberRoleRepository {
    /// Associates a member and a role
    fn associate(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
        role: Role,
    ) -> BackendResult<()>;
    /// Dissociates a member from a role
    fn dissociate(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
        role: Role,
    ) -> BackendResult<()>;
    /// Lists all roles of a specific member
    fn list_by_id(&self, conn: &mut DatabaseConnection, member_id: i32)
        -> BackendResult<Vec<Role>>;
}

pub trait WorkgroupRoleRepository {
    /// Associates a work group and a role
    fn associate(
        &self,
        conn: &mut DatabaseConnection,
        workgroup_id: i32,
        role: Role,
    ) -> BackendResult<()>;
    /// Dissociates a work group from a role
    fn dissociate(
        &self,
        conn: &mut DatabaseConnection,
        workgroup_id: i32,
        role: Role,
    ) -> BackendResult<()>;
    /// Lists all roles of a specific workgroup
    fn list_by_id(
        &self,
        conn: &mut DatabaseConnection,
        workgroup_id: i32,
    ) -> BackendResult<Vec<Role>>;
}

/// Manages a virtual view over the different roles from both members and associated work groups
pub trait AuthorizationRepository {
    /// Finds all roles for a member from direct association and work group association
    fn find_composite_roles_by_member_id(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
    ) -> BackendResult<Vec<Role>>;
}
