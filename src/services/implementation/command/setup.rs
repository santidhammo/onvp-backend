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
use crate::generic::storage::database::{DatabaseConnection, DatabaseConnectionPool};
use crate::generic::Injectable;
use crate::model::interface::commands::FirstOperatorRegisterCommand;
use crate::model::primitives::Role;
use crate::model::storage::extended_entities::ExtendedMember;
use crate::repositories::traits::{MemberRepository, MemberRoleRepository};
use crate::services::traits::command::SetupCommandService;
use actix_web::web::Data;
use diesel::Connection;
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    member_repository: Data<dyn MemberRepository>,
    member_role_repository: Data<dyn MemberRoleRepository>,
}

impl Implementation {
    fn has_operators(&self, conn: &mut DatabaseConnection) -> BackendResult<bool> {
        Ok(self
            .member_repository
            .count_members_with_role(conn, Role::Operator)?
            > 0)
    }
}

impl SetupCommandService for Implementation {
    fn register_first_operator(
        &self,
        command: &FirstOperatorRegisterCommand,
    ) -> BackendResult<String> {
        let mut conn = self.pool.get()?;
        conn.transaction::<String, BackendError, _>(|conn| {
            if !self.has_operators(conn)? {
                let extended_member = ExtendedMember::from(command);

                let member_id = self
                    .member_repository
                    .create_inactive(conn, &extended_member)?;

                self.member_role_repository
                    .associate(conn, member_id, Role::Member)?;

                self.member_role_repository
                    .associate(conn, member_id, Role::Operator)?;

                Ok(extended_member.activation_string)
            } else {
                Err(BackendError::bad())
            }
        })
    }
}

impl
    Injectable<
        (
            &DatabaseConnectionPool,
            &Data<dyn MemberRepository>,
            &Data<dyn MemberRoleRepository>,
        ),
        dyn SetupCommandService,
    > for Implementation
{
    fn injectable(
        (pool, member_repository, member_role_repository): (
            &DatabaseConnectionPool,
            &Data<dyn MemberRepository>,
            &Data<dyn MemberRoleRepository>,
        ),
    ) -> Data<dyn SetupCommandService> {
        let implementation = Self {
            pool: pool.clone(),
            member_repository: member_repository.clone(),
            member_role_repository: member_role_repository.clone(),
        };
        let arc: Arc<dyn SetupCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
