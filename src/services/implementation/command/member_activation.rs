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
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::Injectable;
use crate::model::interface::commands::MemberActivationCommand;
use crate::model::interface::responses::MemberResponse;
use crate::repositories::traits::MemberRepository;
use crate::services::traits::command::MemberActivationCommandService;
use actix_web::web::Data;
use diesel::Connection;
use std::sync::Arc;
use totp_rs::TOTP;

pub struct Implementation {
    pool: DbPool,
    member_repository: Data<dyn MemberRepository>,
}

impl MemberActivationCommandService for Implementation {
    fn activate(&self, data: &MemberActivationCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<_, BackendError, _>(|conn| {
            let extended_member = self
                .member_repository
                .find_extended_by_activation_string(conn, &data.activation_string)?;
            let member_response = MemberResponse::from(&extended_member);
            let totp: TOTP = member_response.try_into()?;
            totp.check_current(&data.token)?;
            self.member_repository
                .activate_by_id(conn, *(&extended_member.id))?;
            Ok(())
        })
    }
}

impl Injectable<(&DbPool, &Data<dyn MemberRepository>), dyn MemberActivationCommandService>
    for Implementation
{
    fn injectable(
        (pool, member_repository): (&DbPool, &Data<dyn MemberRepository>),
    ) -> Data<dyn MemberActivationCommandService> {
        let implementation = Self {
            pool: pool.clone(),
            member_repository: member_repository.clone(),
        };
        let arc: Arc<dyn MemberActivationCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
