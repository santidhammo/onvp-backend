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
use crate::model::interface::commands::MemberActivationCommand;
use crate::model::interface::responses::MemberResponse;
use crate::repositories::definitions::MemberRepository;
use crate::services::definitions::command::MemberActivationCommandService;
use actix_web::web::Data;
use std::sync::Arc;
use totp_rs::TOTP;

pub struct Implementation {
    member_repository: Data<dyn MemberRepository>,
}

impl MemberActivationCommandService for Implementation {
    fn activate(&self, mut session: Session, data: &MemberActivationCommand) -> BackendResult<()> {
        let extended_member = self
            .member_repository
            .find_extended_by_activation_string(&mut session, &data.activation_string)?;
        let member_response = MemberResponse::from(&extended_member);
        let totp: TOTP = member_response.try_into()?;
        totp.check_current(&data.token)?;
        self.member_repository
            .activate_by_id(&mut session, *(&extended_member.id))?;
        Ok(())
    }
}

impl Injectable<ServiceDependencies, dyn MemberActivationCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn MemberActivationCommandService> {
        let implementation = Self {
            member_repository: dependencies.member_repository.clone(),
        };
        let arc: Arc<dyn MemberActivationCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
