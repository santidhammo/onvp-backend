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
use crate::model::primitives::{Role, RoleClass};
use crate::repositories::definitions::{MemberRoleRepository, WorkgroupRoleRepository};
use crate::services::definitions::request::RoleRequestService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    member_role_repository: Data<dyn MemberRoleRepository>,
    workgroup_role_repository: Data<dyn WorkgroupRoleRepository>,
}

impl RoleRequestService for Implementation {
    fn list_by_id_and_class(
        &self,
        mut session: Session,
        id: i32,
        class: RoleClass,
    ) -> BackendResult<Vec<Role>> {
        match class {
            RoleClass::Member => self.member_role_repository.list_by_id(&mut session, id),
            RoleClass::Workgroup => self.workgroup_role_repository.list_by_id(&mut session, id),
        }
    }
}

impl Injectable<ServiceDependencies, dyn RoleRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn RoleRequestService> {
        let implementation = Self {
            member_role_repository: dependencies.member_role_repository.clone(),
            workgroup_role_repository: dependencies.workgroup_role_repository.clone(),
        };
        let arc: Arc<dyn RoleRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
