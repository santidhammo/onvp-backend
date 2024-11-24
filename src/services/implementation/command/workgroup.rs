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
use crate::model::interface::commands::{
    AssociateMemberToWorkgroupCommand, DissociateMemberFromWorkgroupCommand,
    WorkgroupRegisterCommand, WorkgroupUpdateCommand,
};
use crate::model::storage::entities::Workgroup;
use crate::repositories::definitions::WorkgroupRepository;
use crate::services::definitions::command::WorkgroupCommandService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    workgroup_repository: Data<dyn WorkgroupRepository>,
}

impl WorkgroupCommandService for Implementation {
    fn register(
        &self,
        mut session: Session,
        command: &WorkgroupRegisterCommand,
    ) -> BackendResult<i32> {
        self.workgroup_repository
            .register(&mut session, Workgroup::from(command))
    }

    fn update(
        &self,
        mut session: Session,
        workgroup_id: i32,
        command: &WorkgroupUpdateCommand,
    ) -> BackendResult<()> {
        let origin = self
            .workgroup_repository
            .find_by_id(&mut session, workgroup_id)?;
        let new = Workgroup::from((&origin, command));
        self.workgroup_repository.save(&mut session, new)
    }

    fn unregister(&self, mut session: Session, workgroup_id: i32) -> BackendResult<()> {
        self.workgroup_repository
            .unregister(&mut session, workgroup_id)
    }

    fn associate_member_to_workgroup(
        &self,
        mut session: Session,
        command: &AssociateMemberToWorkgroupCommand,
    ) -> BackendResult<()> {
        self.workgroup_repository.associate_member_to_workgroup(
            &mut session,
            command.member_id,
            command.workgroup_id,
        )
    }

    fn dissociate_member_from_workgroup(
        &self,
        mut session: Session,
        command: &DissociateMemberFromWorkgroupCommand,
    ) -> BackendResult<()> {
        self.workgroup_repository.dissociate_member_from_workgroup(
            &mut session,
            command.member_id,
            command.workgroup_id,
        )
    }
}

impl Injectable<ServiceDependencies, dyn WorkgroupCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn WorkgroupCommandService> {
        let implementation = Self {
            workgroup_repository: dependencies.workgroup_repository.clone(),
        };
        let arc: Arc<dyn WorkgroupCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
