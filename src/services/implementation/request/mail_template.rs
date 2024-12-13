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
use crate::model::interface::responses::{MailTemplateNameResponse, MailTemplateResponse};
use crate::repositories::definitions::MailTemplateRepository;
use crate::services::definitions::request::MailTemplateRequestService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    mail_template_repository: Data<dyn MailTemplateRepository>,
}

impl MailTemplateRequestService for Implementation {
    fn list(&self, mut session: Session) -> BackendResult<Vec<MailTemplateNameResponse>> {
        let listing = self.mail_template_repository.list(&mut session)?;
        Ok(listing
            .iter()
            .cloned()
            .map(|(id, name)| MailTemplateNameResponse::from((id, &name as &str)))
            .collect())
    }

    fn find_by_id(
        &self,
        mut session: Session,
        mail_template_id: i32,
    ) -> BackendResult<MailTemplateResponse> {
        self.mail_template_repository
            .find_by_id(&mut session, mail_template_id)
            .map(|t| MailTemplateResponse::from(&t))
    }
}

impl Injectable<ServiceDependencies, dyn MailTemplateRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn MailTemplateRequestService> {
        let implementation = Self {
            mail_template_repository: dependencies.mail_template_repository.clone(),
        };

        let arc: Arc<dyn MailTemplateRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
