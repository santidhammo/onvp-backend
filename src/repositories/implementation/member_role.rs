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
use crate::generic::storage::session::Session;
use crate::generic::Injectable;
use crate::model::primitives::Role;
use crate::model::storage::roles::MemberRoleAssociation;
use crate::repositories::definitions::MemberRoleRepository;
use crate::schema::member_role_associations;
use actix_web::web::Data;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

pub struct Implementation;

impl MemberRoleRepository for Implementation {
    fn associate(&self, session: &mut Session, member_id: i32, role: Role) -> BackendResult<()> {
        // The public role is assumed for any user, also users which are not a member, therefore
        // it can not be associated.
        if role == Role::Public {
            return Err(BackendError::bad());
        }
        session.run(|conn| {
            diesel::insert_into(member_role_associations::table)
                .values((
                    member_role_associations::member_id.eq(member_id),
                    member_role_associations::system_role.eq(role),
                ))
                .execute(conn)?;
            Ok(())
        })
    }

    fn dissociate(&self, session: &mut Session, member_id: i32, role: Role) -> BackendResult<()> {
        // Every member always has the member role, this can not be removed
        if role == Role::Member {
            return Err(BackendError::bad());
        }
        session.run(|conn| {
            let deleted_rows = diesel::delete(member_role_associations::table)
                .filter(
                    member_role_associations::member_id
                        .eq(member_id)
                        .and(member_role_associations::system_role.eq(role)),
                )
                .execute(conn)?;

            if deleted_rows == 0 {
                Err(BackendError::not_enough_records())
            } else {
                Ok(())
            }
        })
    }

    fn list_by_id(&self, session: &mut Session, member_id: i32) -> BackendResult<Vec<Role>> {
        let filter = member_role_associations::member_id.eq(member_id);
        session.run(|conn| {
            let role_associations: Vec<MemberRoleAssociation> = member_role_associations::table
                .filter(filter)
                .select(MemberRoleAssociation::as_select())
                .load(conn)?;
            Ok(role_associations.iter().map(|ra| ra.system_role).collect())
        })
    }
}

impl Injectable<(), dyn MemberRoleRepository> for Implementation {
    fn make(_: &()) -> Data<dyn MemberRoleRepository> {
        let arc: Arc<dyn MemberRoleRepository> = Arc::new(Self);
        Data::from(arc)
    }
}
