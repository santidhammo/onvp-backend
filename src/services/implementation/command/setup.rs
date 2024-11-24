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
use crate::injection::ServiceDependencies;
use crate::model::interface::commands::FirstOperatorRegisterCommand;
use crate::model::primitives::Role;
use crate::model::storage::extended_entities::ExtendedMember;
use crate::repositories::definitions::{MemberRepository, MemberRoleRepository};
use crate::services::definitions::command::SetupCommandService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    member_repository: Data<dyn MemberRepository>,
    member_role_repository: Data<dyn MemberRoleRepository>,
}

impl SetupCommandService for Implementation {
    fn register_first_operator(
        &self,
        mut session: Session,
        command: &FirstOperatorRegisterCommand,
    ) -> BackendResult<String> {
        if !self.has_operators(&mut session)? {
            let extended_member = ExtendedMember::from(command);

            let member_id = self
                .member_repository
                .create_inactive(&mut session, &extended_member)?;

            self.member_role_repository
                .associate(&mut session, member_id, Role::Member)?;

            self.member_role_repository
                .associate(&mut session, member_id, Role::Operator)?;

            Ok(extended_member.activation_string)
        } else {
            Err(BackendError::bad())
        }
    }
}

impl Implementation {
    fn has_operators(&self, session: &mut Session) -> BackendResult<bool> {
        Ok(self
            .member_repository
            .count_members_with_role(session, Role::Operator)?
            > 0)
    }
}

impl Injectable<ServiceDependencies, dyn SetupCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn SetupCommandService> {
        let implementation = Self {
            member_repository: dependencies.member_repository.clone(),
            member_role_repository: dependencies.member_role_repository.clone(),
        };
        let arc: Arc<dyn SetupCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
