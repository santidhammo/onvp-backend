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
use crate::generic::storage::session::Session;
use crate::generic::Injectable;
use crate::injection::ServiceDependencies;
use crate::model::interface::commands::{AssociateRoleCommand, DissociateRoleCommand};
use crate::model::primitives::RoleClass;
use crate::repositories::definitions::{MemberRoleRepository, WorkgroupRoleRepository};
use crate::services::definitions::command::RoleCommandService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    pub member_role_repository: Data<dyn MemberRoleRepository>,
    pub workgroup_role_repository: Data<dyn WorkgroupRoleRepository>,
}

impl RoleCommandService for Implementation {
    fn associate_role(
        &self,
        mut session: Session,
        command: &AssociateRoleCommand,
    ) -> BackendResult<()> {
        match command.class {
            RoleClass::Member => {
                self.member_role_repository
                    .associate(&mut session, command.id, command.role)
            }
            RoleClass::Workgroup => {
                self.workgroup_role_repository
                    .associate(&mut session, command.id, command.role)
            }
        }
    }

    fn dissociate_role(
        &self,
        mut session: Session,
        command: &DissociateRoleCommand,
    ) -> BackendResult<()> {
        match command.class {
            RoleClass::Member => {
                self.member_role_repository
                    .dissociate(&mut session, command.id, command.role)
            }
            RoleClass::Workgroup => {
                self.workgroup_role_repository
                    .dissociate(&mut session, command.id, command.role)
            }
        }
    }
}

impl Injectable<ServiceDependencies, dyn RoleCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn RoleCommandService> {
        let implementation = Self {
            member_role_repository: dependencies.member_role_repository.clone(),
            workgroup_role_repository: dependencies.workgroup_role_repository.clone(),
        };
        let arc: Arc<dyn RoleCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
