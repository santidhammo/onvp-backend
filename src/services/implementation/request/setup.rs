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
use crate::model::primitives::Role;
use crate::repositories::definitions::MemberRepository;
use crate::services::definitions::request::SetupRequestService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    member_repository: Data<dyn MemberRepository>,
}

impl SetupRequestService for Implementation {
    fn should_setup(&self, mut session: Session) -> BackendResult<bool> {
        Ok(self
            .member_repository
            .count_members_with_role(&mut session, Role::Operator)?
            == 0)
    }
}

impl Injectable<ServiceDependencies, dyn SetupRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn SetupRequestService> {
        let implementation = Self {
            member_repository: dependencies.member_repository.clone(),
        };
        let arc: Arc<dyn SetupRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
