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
use crate::dal::DbConnection;
use crate::generic::result::BackendResult;
use crate::model::prelude::Role;
use crate::model::storage::extended_entities::ExtendedMember;

pub trait MemberRepository {
    fn create_inactive(
        &self,
        conn: &mut DbConnection,
        member_extended: &ExtendedMember,
    ) -> BackendResult<i32>;

    fn find_extended_by_id(
        &self,
        conn: &mut DbConnection,
        id: i32,
    ) -> BackendResult<ExtendedMember>;

    fn count_members_with_role(&self, conn: &mut DbConnection, role: Role) -> BackendResult<usize>;
}

pub trait MemberRoleRepository {
    fn associate_role(
        &self,
        conn: &mut DbConnection,
        member_id: i32,
        role: Role,
    ) -> BackendResult<()>;
    fn dissociate_role(
        &self,
        conn: &mut DbConnection,
        member_id: i32,
        role: Role,
    ) -> BackendResult<()>;
}

pub trait WorkgroupRoleRepository {
    fn associate_role(
        &self,
        conn: &mut DbConnection,
        workgroup_id: i32,
        role: Role,
    ) -> BackendResult<()>;
    fn dissociate_role(
        &self,
        conn: &mut DbConnection,
        workgroup_id: i32,
        role: Role,
    ) -> BackendResult<()>;
}
