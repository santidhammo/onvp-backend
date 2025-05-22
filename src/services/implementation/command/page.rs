/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024-2025.  Sjoerd van Leent
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
use crate::model::interface::commands::{CreatePageCommand, PublishPageCommand, UpdatePageCommand};
use crate::model::storage::entities::Page;
use crate::repositories::definitions::{PageRepository, PropertiesRepository};
use crate::services::definitions::command::PageCommandService;
use actix_web::web::Data;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

pub struct Implementation {
    page_repository: Data<dyn PageRepository>,
    properties_repository: Data<dyn PropertiesRepository>,
}

impl PageCommandService for Implementation {
    fn create(&self, mut session: Session, command: &CreatePageCommand) -> BackendResult<()> {
        if let Some(event_date) = &command.event_date {
            event_date.validate()?;
        }
        let page = Page::from(command);

        self.page_repository.create(&mut session, page)
    }

    fn set_content(&self, mut session: Session, page_id: i32, content: &str) -> BackendResult<()> {
        let page: Page = self.page_repository.find_by_id(&mut session, page_id)?;
        let content_asset = page.content_asset;
        let pb = crate::path_for_asset(&content_asset)?;
        let mut w = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&pb)?;
        w.write(&content.as_bytes())?;
        Ok(())
    }

    fn update(
        &self,
        mut session: Session,
        page_id: i32,
        command: &UpdatePageCommand,
    ) -> BackendResult<()> {
        let origin: Page = self.page_repository.find_by_id(&mut session, page_id)?;
        let page = Page::from((&origin, command));
        self.page_repository.update(&mut session, page)
    }

    fn publish(
        &self,
        mut session: Session,
        page_id: i32,
        command: &PublishPageCommand,
    ) -> BackendResult<()> {
        self.page_repository.reset_roles(&mut session, page_id)?;
        self.page_repository
            .assign_roles(&mut session, page_id, &command.roles)
    }

    fn unpublish(&self, mut session: Session, page_id: i32) -> BackendResult<()> {
        self.page_repository.reset_roles(&mut session, page_id)
    }

    fn delete(&self, mut session: Session, page_id: i32) -> BackendResult<()> {
        self.page_repository.delete(&mut session, page_id)
    }

    fn set_default(&self, mut session: Session, page_id: i32) -> BackendResult<()> {
        // Verify the identifier
        let _ = self.page_repository.find_by_id(&mut session, page_id)?;
        self.properties_repository
            .set_int_property(&mut session, "default-page", Some(page_id))
    }

    fn set_order(
        &self,
        mut session: Session,
        page_id: i32,
        order_number: i32,
    ) -> BackendResult<()> {
        // Verify that the page really exists
        let _ = self.page_repository.find_by_id(&mut session, page_id)?;
        self.page_repository
            .set_order_by_id(&mut session, page_id, order_number)
    }

    fn set_or_unset_parent_id(
        &self,
        mut session: Session,
        page_id: i32,
        maybe_parent_id: Option<i32>,
    ) -> BackendResult<()> {
        // Verify that the page really exists
        let _ = self.page_repository.find_by_id(&mut session, page_id)?;

        if let Some(parent_id) = maybe_parent_id {
            // Verify that the parent page really exists and has no parent page itself
            let parent_page = self.page_repository.find_by_id(&mut session, parent_id)?;
            if let Some(_) = parent_page.parent_id {
                return Err(BackendError::bad());
            }
        }
        self.page_repository
            .set_or_unset_parent_id_by_id(&mut session, page_id, maybe_parent_id)
    }
}

impl Injectable<ServiceDependencies, dyn PageCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn PageCommandService> {
        let implementation = Self {
            page_repository: dependencies.page_repository.clone(),
            properties_repository: dependencies.properties_repository.clone(),
        };
        let arc: Arc<dyn PageCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
