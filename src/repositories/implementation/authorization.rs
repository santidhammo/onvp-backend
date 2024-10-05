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
use crate::generic::Injectable;
use crate::model::primitives::Role;
use crate::repositories::definitions::AuthorizationRepository;
use crate::schema::{
    member_role_associations, workgroup_member_relationships, workgroup_role_associations,
};
use actix_web::web::Data;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use std::collections::HashSet;
use std::sync::Arc;

pub struct Implementation;

impl AuthorizationRepository for Implementation {
    fn find_composite_roles_by_member_id(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
    ) -> BackendResult<Vec<Role>> {
        let mut roles = member_role_associations::table
            .select(member_role_associations::system_role)
            .filter(member_role_associations::member_id.eq(member_id))
            .load::<Role>(conn)?;

        let workgroup_ids = workgroup_member_relationships::table
            .select(workgroup_member_relationships::workgroup_id)
            .filter(workgroup_member_relationships::member_id.eq(member_id))
            .load::<i32>(conn)?;

        for workgroup_id in workgroup_ids {
            let workgroup_roles = workgroup_role_associations::table
                .select(workgroup_role_associations::system_role)
                .filter(workgroup_role_associations::workgroup_id.eq(workgroup_id))
                .load::<Role>(conn)?;

            roles.extend(workgroup_roles);
        }
        roles.push(Role::Public);
        roles.push(Role::Member);

        let result: HashSet<Role> = HashSet::from_iter(roles);

        Ok(result.iter().map(|v| *v).collect())
    }
}

impl Injectable<(), dyn AuthorizationRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn AuthorizationRepository> {
        let arc: Arc<dyn AuthorizationRepository> = Arc::new(Self);
        Data::from(arc)
    }
}
