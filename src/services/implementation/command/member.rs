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
use crate::dal::members::create_inactive_member;
use crate::dal::DbPool;
use crate::generic::activation::send_activation_email;
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::security::{create_activation_string, MEMBER_ACTIVATION_MINUTES};
use crate::injection::Injectable;
use crate::model::interface::commands::MemberRegisterCommand;
use crate::model::security::Role;
use crate::services::traits::command::MemberCommandService;
use actix_web::web::Data;
use diesel::Connection;
use std::sync::Arc;

pub struct Implementation {
    pool: DbPool,
}

impl MemberCommandService for Implementation {
    fn register_inactive(&self, command: &MemberRegisterCommand) -> BackendResult<i32> {
        let activation_string = create_activation_string();
        let mut conn = crate::dal::connect(&self.pool)?;
        conn.transaction::<i32, BackendError, _>(|conn| {
            let member_id = create_inactive_member(
                conn,
                &command.detail_register_sub_command,
                &command.address_register_sub_command,
                &activation_string,
                *MEMBER_ACTIVATION_MINUTES,
                Role::Member,
            )?;

            send_activation_email(
                &command.detail_register_sub_command.email_address,
                &activation_string,
            )?;

            Ok(member_id)
        })
    }
}

impl Injectable<&DbPool, dyn MemberCommandService> for Implementation {
    fn injectable(pool: &DbPool) -> Data<dyn MemberCommandService> {
        let implementation = Self { pool: pool.clone() };
        let member_command_controller_arc: Arc<dyn MemberCommandService> = Arc::new(implementation);
        Data::from(member_command_controller_arc)
    }
}
