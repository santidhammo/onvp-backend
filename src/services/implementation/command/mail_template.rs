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
use crate::model::interface::commands::{CreateMailTemplateCommand, UpdateMailTemplateCommand};
use crate::model::storage::entities::MailTemplate;
use crate::repositories::definitions::MailTemplateRepository;
use crate::services::definitions::command::MailTemplateCommandService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    mail_template_repository: Data<dyn MailTemplateRepository>,
}

impl MailTemplateCommandService for Implementation {
    fn create(
        &self,
        mut session: Session,
        command: &CreateMailTemplateCommand,
    ) -> BackendResult<()> {
        let template = MailTemplate::from(command);
        self.mail_template_repository.create(&mut session, template)
    }

    fn update(
        &self,
        mut session: Session,
        mail_template_id: i32,
        command: &UpdateMailTemplateCommand,
    ) -> BackendResult<()> {
        let origin = self
            .mail_template_repository
            .find_by_id(&mut session, mail_template_id)?;
        let mail_template = MailTemplate::from((&origin, command));
        self.mail_template_repository
            .update(&mut session, mail_template)
    }

    fn delete(&self, mut session: Session, mail_template_id: i32) -> BackendResult<()> {
        self.mail_template_repository
            .delete(&mut session, mail_template_id)
    }
}

impl Injectable<ServiceDependencies, dyn MailTemplateCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn MailTemplateCommandService> {
        let implementation = Self {
            mail_template_repository: dependencies.mail_template_repository.clone(),
        };

        let arc: Arc<dyn MailTemplateCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
