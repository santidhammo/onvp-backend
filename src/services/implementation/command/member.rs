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
use crate::generic::lazy::{SendEmailConfig, SEND_ACTIVATION_EMAIL_CONFIG};
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::generic::Injectable;
use crate::model::interface::commands::{
    MemberRegisterCommand, MemberUpdateAddressCommand, MemberUpdateCommand,
};
use crate::model::primitives::Role;
use crate::model::storage::extended_entities::ExtendedMember;
use crate::repositories::definitions::{MemberRepository, MemberRoleRepository};
use crate::services::definitions::command::MemberCommandService;
use actix_web::web::Data;
use diesel::Connection;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    member_repository: Data<dyn MemberRepository>,
    member_role_repository: Data<dyn MemberRoleRepository>,
    send_activation_email_config: SendEmailConfig,
}

impl MemberCommandService for Implementation {
    fn register_inactive(&self, command: &MemberRegisterCommand) -> BackendResult<i32> {
        let mut conn = self.pool.get()?;
        conn.transaction::<i32, BackendError, _>(|conn| {
            let extended_member = ExtendedMember::from(command);

            let member_id = self
                .member_repository
                .create_inactive(conn, &extended_member)?;

            self.member_role_repository
                .associate(conn, member_id, Role::Member)?;

            self.send_activation_email(
                &command.detail_register_sub_command.email_address,
                &extended_member.activation_string,
            )?;

            Ok(member_id)
        })
    }

    fn update(&self, member_id: i32, command: &MemberUpdateCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<_, BackendError, _>(|conn| {
            let origin = self
                .member_repository
                .find_extended_by_id(conn, member_id)?;
            let new = ExtendedMember::from((&origin, command));
            self.member_repository.save(conn, new)?;
            Ok(())
        })
    }

    fn update_address(
        &self,
        member_id: i32,
        command: &MemberUpdateAddressCommand,
    ) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<_, BackendError, _>(|conn| {
            let origin = self
                .member_repository
                .find_extended_by_id(conn, member_id)?;
            let new = ExtendedMember::from((&origin, command));
            self.member_repository.save(conn, new)?;
            Ok(())
        })
    }
}

impl
    Injectable<
        (
            &DatabaseConnectionPool,
            &Data<dyn MemberRepository>,
            &Data<dyn MemberRoleRepository>,
        ),
        dyn MemberCommandService,
    > for Implementation
{
    fn injectable(
        (pool, member_repository, member_role_repository): (
            &DatabaseConnectionPool,
            &Data<dyn MemberRepository>,
            &Data<dyn MemberRoleRepository>,
        ),
    ) -> Data<dyn MemberCommandService> {
        let implementation = Self {
            pool: pool.clone(),
            member_repository: member_repository.clone(),
            member_role_repository: member_role_repository.clone(),
            send_activation_email_config: SEND_ACTIVATION_EMAIL_CONFIG.clone(),
        };
        let arc: Arc<dyn MemberCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}

impl Implementation {
    fn send_activation_email(
        &self,
        email_address: &str,
        activation_string: &str,
    ) -> BackendResult<()> {
        let email_body = self
            .send_activation_email_config
            .email_body_template
            .replace("{}", &activation_string);
        let email = Message::builder()
            .from(self.send_activation_email_config.email_from.clone())
            .to(email_address.parse()?)
            .subject(self.send_activation_email_config.email_subject.clone())
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(email_body)?;

        let mut builder =
            SmtpTransport::relay(&self.send_activation_email_config.email_smtp_relay)?
                .port(self.send_activation_email_config.email_smtp_port);
        if !self.send_activation_email_config.email_dev_mode {
            let smtp_relay_credentials = lettre::transport::smtp::authentication::Credentials::new(
                self.send_activation_email_config.email_smtp_user.clone(),
                self.send_activation_email_config
                    .email_smtp_password
                    .clone(),
            );
            builder = builder.credentials(smtp_relay_credentials)
        } else {
            builder = builder.tls(Tls::None)
        }
        let relay = builder.build();
        relay.send(&email)?;
        Ok(())
    }
}