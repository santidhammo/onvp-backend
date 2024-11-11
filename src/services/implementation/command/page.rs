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
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::generic::Injectable;
use crate::model::interface::commands::{CreatePageCommand, PublishPageCommand, UpdatePageCommand};
use crate::model::storage::entities::Page;
use crate::repositories::definitions::{PageRepository, PropertiesRepository};
use crate::services::definitions::command::PageCommandService;
use actix_web::web::Data;
use diesel::Connection;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    page_repository: Data<dyn PageRepository>,
    properties_repository: Data<dyn PropertiesRepository>,
}

impl PageCommandService for Implementation {
    fn create(&self, command: &CreatePageCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        if let Some(event_date) = &command.event_date {
            event_date.validate()?;
        }
        let page = Page::from(command);
        self.page_repository.create(&mut conn, page)
    }

    fn set_content(&self, page_id: i32, content: &str) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<(), BackendError, _>(|conn| {
            let page: Page = self.page_repository.find_by_id(conn, page_id)?;
            let content_asset = page.content_asset;
            let pb = crate::path_for_asset(&content_asset)?;
            let mut w = OpenOptions::new().write(true).create(true).open(&pb)?;
            w.write(&content.as_bytes())?;
            Ok(())
        })
    }

    fn update(&self, page_id: i32, command: &UpdatePageCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<(), BackendError, _>(|conn| {
            let origin: Page = self.page_repository.find_by_id(conn, page_id)?;
            let page = Page::from((&origin, command));
            self.page_repository.update(conn, page)
        })
    }

    fn publish(&self, page_id: i32, command: &PublishPageCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<(), BackendError, _>(|conn| {
            self.page_repository.reset_roles(conn, page_id)?;
            self.page_repository
                .assign_roles(conn, page_id, &command.roles)
        })
    }

    fn unpublish(&self, page_id: i32) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        self.page_repository.reset_roles(&mut conn, page_id)
    }

    fn delete(&self, page_id: i32) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        self.page_repository.delete(&mut conn, page_id)
    }

    fn set_default(&self, page_id: i32) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<_, BackendError, _>(|conn| {
            // Verify the identifier
            let _ = self.page_repository.find_by_id(conn, page_id)?;
            self.properties_repository
                .set_int_property(conn, "default-page", Some(page_id))
        })
    }
}

impl
    Injectable<
        (
            &DatabaseConnectionPool,
            &Data<dyn PageRepository>,
            &Data<dyn PropertiesRepository>,
        ),
        dyn PageCommandService,
    > for Implementation
{
    fn injectable(
        (pool, page_repository, properties_repository): (
            &DatabaseConnectionPool,
            &Data<dyn PageRepository>,
            &Data<dyn PropertiesRepository>,
        ),
    ) -> Data<dyn PageCommandService> {
        let implementation = Self {
            pool: pool.clone(),
            page_repository: page_repository.clone(),
            properties_repository: properties_repository.clone(),
        };
        let arc: Arc<dyn PageCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
