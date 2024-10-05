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
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::Injectable;
use crate::model::primitives::Role;
use crate::model::storage::roles::WorkgroupRoleAssociation;
use crate::repositories::traits::WorkgroupRoleRepository;
use crate::schema::workgroup_role_associations;
use actix_web::web::Data;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

pub struct Implementation;

impl WorkgroupRoleRepository for Implementation {
    fn associate(
        &self,
        conn: &mut DbConnection,
        workgroup_id: i32,
        role: Role,
    ) -> BackendResult<()> {
        // The member role is reserved only for individual members only
        if role == Role::Member {
            return Err(BackendError::bad());
        }

        diesel::insert_into(workgroup_role_associations::table)
            .values((
                workgroup_role_associations::workgroup_id.eq(workgroup_id),
                workgroup_role_associations::system_role.eq(role),
            ))
            .execute(conn)?;
        Ok(())
    }

    fn dissociate(
        &self,
        conn: &mut DbConnection,
        workgroup_id: i32,
        role: Role,
    ) -> BackendResult<()> {
        let deleted_rows = diesel::delete(workgroup_role_associations::table)
            .filter(
                workgroup_role_associations::workgroup_id
                    .eq(workgroup_id)
                    .and(workgroup_role_associations::system_role.eq(role)),
            )
            .execute(conn)?;

        if deleted_rows == 0 {
            Err(BackendError::not_enough_records())
        } else {
            Ok(())
        }
    }

    fn list_by_id(&self, conn: &mut DbConnection, workgroup_id: i32) -> BackendResult<Vec<Role>> {
        let filter = workgroup_role_associations::workgroup_id.eq(workgroup_id);
        let role_associations: Vec<WorkgroupRoleAssociation> = workgroup_role_associations::table
            .filter(filter)
            .select(WorkgroupRoleAssociation::as_select())
            .load(conn)?;
        Ok(role_associations.iter().map(|ra| ra.system_role).collect())
    }
}

impl Injectable<(), dyn WorkgroupRoleRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn WorkgroupRoleRepository> {
        let arc: Arc<dyn WorkgroupRoleRepository> = Arc::new(Self);
        Data::from(arc)
    }
}
