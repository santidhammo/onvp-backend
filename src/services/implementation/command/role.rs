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
use crate::dal::DbPool;
use crate::generic::result::BackendResult;
use crate::injection::Injectable;
use crate::model::interface::commands::{AssociateRoleCommand, DissociateRoleCommand};
use crate::model::prelude::RoleClass;
use crate::repositories::traits::{MemberRoleRepository, WorkgroupRoleRepository};
use crate::services::traits::command::RoleCommandService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    pub pool: DbPool,
    pub member_role_repository: Data<dyn MemberRoleRepository>,
    pub workgroup_role_repository: Data<dyn WorkgroupRoleRepository>,
}

impl RoleCommandService for Implementation {
    fn associate_role(&self, command: &AssociateRoleCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        match command.class {
            RoleClass::Member => {
                self.member_role_repository
                    .associate_role(&mut conn, command.id, command.role)
            }
            RoleClass::Workgroup => {
                self.workgroup_role_repository
                    .associate_role(&mut conn, command.id, command.role)
            }
        }
    }

    fn dissociate_role(&self, command: &DissociateRoleCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        match command.class {
            RoleClass::Member => {
                self.member_role_repository
                    .dissociate_role(&mut conn, command.id, command.role)
            }
            RoleClass::Workgroup => {
                self.workgroup_role_repository
                    .dissociate_role(&mut conn, command.id, command.role)
            }
        }
    }
}

impl
    Injectable<
        (
            &Data<dyn MemberRoleRepository>,
            &Data<dyn WorkgroupRoleRepository>,
            &DbPool,
        ),
        dyn RoleCommandService,
    > for Implementation
{
    fn injectable(
        dependencies: (
            &Data<dyn MemberRoleRepository>,
            &Data<dyn WorkgroupRoleRepository>,
            &DbPool,
        ),
    ) -> Data<dyn RoleCommandService> {
        let (member_role_repository, workgroup_role_repository, pool) = dependencies;
        let implementation = Self {
            member_role_repository: member_role_repository.clone(),
            workgroup_role_repository: workgroup_role_repository.clone(),
            pool: pool.clone(),
        };
        let member_command_controller_arc: Arc<dyn RoleCommandService> = Arc::new(implementation);
        Data::from(member_command_controller_arc)
    }
}
