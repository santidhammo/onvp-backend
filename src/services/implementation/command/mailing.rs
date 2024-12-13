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
use crate::generic::lazy::{SendEmailConfig, SEND_EMAIL_CONFIG};
use crate::generic::result::BackendResult;
use crate::generic::storage::session::Session;
use crate::generic::Injectable;
use crate::injection::ServiceDependencies;
use crate::model::interface::commands::send_mail::MailRecipientType;
use crate::model::interface::commands::SendMailCommand;
use crate::model::storage::entities::MailTemplate;
use crate::model::storage::extended_entities::ExtendedMember;
use crate::repositories::definitions::{
    MailTemplateRepository, MemberRepository, WorkgroupRepository,
};
use crate::services::definitions::command::MailingCommandService;
use actix_web::web::Data;
use handlebars::Handlebars;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use serde_json::json;
use std::sync::Arc;

pub struct Implementation {
    mail_template_repository: Data<dyn MailTemplateRepository>,
    workgroup_repository: Data<dyn WorkgroupRepository>,
    member_repository: Data<dyn MemberRepository>,
    send_email_config: SendEmailConfig,
}

impl MailingCommandService for Implementation {
    fn send(&self, mut session: Session, command: &SendMailCommand) -> BackendResult<()> {
        let mail_template = self
            .mail_template_repository
            .find_by_id(&mut session, command.mail_template_id)?;
        let members = self.list_members_by_recipient_type(&mut session, command)?;
        self.render_and_send_email(&command, mail_template, members)?;
        Ok(())
    }
}

impl Implementation {
    fn list_members_by_recipient_type(
        &self,
        mut session: &mut Session,
        command: &SendMailCommand,
    ) -> BackendResult<Vec<ExtendedMember>> {
        let members: Vec<ExtendedMember> = match command.recipient_type {
            MailRecipientType::Member => {
                vec![self
                    .member_repository
                    .find_extended_by_id(&mut session, command.recipient_id)?]
            }
            MailRecipientType::Workgroup => self
                .workgroup_repository
                .find_members_by_id(&mut session, command.recipient_id)?,
            MailRecipientType::MusicalInstrument => self
                .member_repository
                .list_by_musical_instrument(&mut session, command.recipient_id)?,
        };
        Ok(members)
    }

    fn render_and_send_email(
        &self,
        command: &&SendMailCommand,
        mail_template: MailTemplate,
        members: Vec<ExtendedMember>,
    ) -> BackendResult<()> {
        let mut reg = Handlebars::new();
        reg.register_template_string(&format!("{}", mail_template.id), mail_template.body)?;
        for member in &members {
            let body = reg.render(
                &format!("{}", mail_template.id),
                &json!({
                    "first_name": member.member_detail.first_name.clone(),
                    "last_name": member.member_detail.last_name.clone(),
                    }
                ),
            )?;
            self.send_email(&member.member_detail.email_address, &command.subject, &body)?;
        }
        Ok(())
    }

    fn send_email(&self, email_address: &str, subject: &str, body: &str) -> BackendResult<()> {
        let email = Message::builder()
            .from(self.send_email_config.email_from.clone())
            .to(email_address.parse()?)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body.to_string())?;

        let mut builder = SmtpTransport::relay(&self.send_email_config.email_smtp_relay)?
            .port(self.send_email_config.email_smtp_port);
        if !self.send_email_config.email_dev_mode {
            let smtp_relay_credentials = lettre::transport::smtp::authentication::Credentials::new(
                self.send_email_config.email_smtp_user.clone(),
                self.send_email_config.email_smtp_password.clone(),
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

impl Injectable<ServiceDependencies, dyn MailingCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn MailingCommandService> {
        let implementation = Self {
            mail_template_repository: dependencies.mail_template_repository.clone(),
            workgroup_repository: dependencies.workgroup_repository.clone(),
            member_repository: dependencies.member_repository.clone(),
            send_email_config: SEND_EMAIL_CONFIG.clone(),
        };
        let arc: Arc<dyn MailingCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
