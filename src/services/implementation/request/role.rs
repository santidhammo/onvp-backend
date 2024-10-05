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
use crate::generic::Injectable;
use crate::model::primitives::{Role, RoleClass};
use crate::repositories::traits::{
    MemberRoleRepository, WorkgroupRoleRepository,
};
use crate::services::traits::request::RoleRequestService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    pool: DbPool,
    member_role_repository: Data<dyn MemberRoleRepository>,
    workgroup_role_repository: Data<dyn WorkgroupRoleRepository>,
}

impl RoleRequestService for Implementation {
    fn list_by_id_and_class(&self, id: i32, class: RoleClass) -> BackendResult<Vec<Role>> {
        let mut conn = self.pool.get()?;
        match class {
            RoleClass::Member => self.member_role_repository.list_by_id(&mut conn, id),
            RoleClass::Workgroup => self.workgroup_role_repository.list_by_id(&mut conn, id),
        }
    }
}

impl
    Injectable<
        (
            &DbPool,
            &Data<dyn MemberRoleRepository>,
            &Data<dyn WorkgroupRoleRepository>,
        ),
        dyn RoleRequestService,
    > for Implementation
{
    fn injectable(
        (pool, member_role_repository, workgroup_role_repository): (
            &DbPool,
            &Data<dyn MemberRoleRepository>,
            &Data<dyn WorkgroupRoleRepository>,
        ),
    ) -> Data<dyn RoleRequestService> {
        let implementation = Self {
            pool: pool.clone(),
            member_role_repository: member_role_repository.clone().clone(),
            workgroup_role_repository: workgroup_role_repository.clone().clone(),
        };
        let arc: Arc<dyn RoleRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
