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

use crate::model::security::Role;
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Selectable, Queryable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::workgroup_role_associations)]
pub struct WorkgroupRoleAssociation {
    #[schema(example = 1)]
    pub workgroup_id: i32,

    #[schema(example = Role::OrchestraCommittee)]
    pub system_role: Role,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Selectable, Queryable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = crate::schema::member_role_associations)]
pub struct MemberRoleAssociation {
    #[schema(example = 1)]
    pub member_id: i32,
    #[schema(example = 1)]
    pub system_role: Role,
}
