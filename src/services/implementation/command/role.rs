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
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::generic::Injectable;
use crate::model::interface::commands::{AssociateRoleCommand, DissociateRoleCommand};
use crate::model::primitives::RoleClass;
use crate::repositories::definitions::{MemberRoleRepository, WorkgroupRoleRepository};
use crate::services::definitions::command::RoleCommandService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    pub pool: DatabaseConnectionPool,
    pub member_role_repository: Data<dyn MemberRoleRepository>,
    pub workgroup_role_repository: Data<dyn WorkgroupRoleRepository>,
}

impl RoleCommandService for Implementation {
    fn associate_role(&self, command: &AssociateRoleCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        match command.class {
            RoleClass::Member => {
                self.member_role_repository
                    .associate(&mut conn, command.id, command.role)
            }
            RoleClass::Workgroup => {
                self.workgroup_role_repository
                    .associate(&mut conn, command.id, command.role)
            }
        }
    }

    fn dissociate_role(&self, command: &DissociateRoleCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        match command.class {
            RoleClass::Member => {
                self.member_role_repository
                    .dissociate(&mut conn, command.id, command.role)
            }
            RoleClass::Workgroup => {
                self.workgroup_role_repository
                    .dissociate(&mut conn, command.id, command.role)
            }
        }
    }
}

impl
    Injectable<
        (
            &DatabaseConnectionPool,
            &Data<dyn MemberRoleRepository>,
            &Data<dyn WorkgroupRoleRepository>,
        ),
        dyn RoleCommandService,
    > for Implementation
{
    fn injectable(
        (pool, member_role_repository, workgroup_role_repository): (
            &DatabaseConnectionPool,
            &Data<dyn MemberRoleRepository>,
            &Data<dyn WorkgroupRoleRepository>,
        ),
    ) -> Data<dyn RoleCommandService> {
        let implementation = Self {
            member_role_repository: member_role_repository.clone(),
            workgroup_role_repository: workgroup_role_repository.clone(),
            pool: pool.clone(),
        };
        let arc: Arc<dyn RoleCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
