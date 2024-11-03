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
use crate::model::interface::commands::{ImageUploadCommand, PublishImageCommand};
use crate::model::storage::entities::Image;
use crate::repositories::definitions::ImageRepository;
use crate::services::definitions::command::ImageCommandService;
use actix_web::web::Data;
use diesel::Connection;
use image::EncodableLayout;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    image_repository: Data<dyn ImageRepository>,
}

impl ImageCommandService for Implementation {
    fn upload(&self, command: &ImageUploadCommand) -> BackendResult<String> {
        let mut conn = self.pool.get()?;
        conn.transaction::<String, BackendError, _>(|conn| {
            let image = Image::from(command);
            let asset = image.asset.clone();
            self.image_repository.create(conn, image)?;
            let pb = crate::path_for_asset(&asset)?;
            let mut w = OpenOptions::new().write(true).create_new(true).open(&pb)?;
            w.write(&command.data.as_bytes())?;
            Ok(asset)
        })
    }

    fn publish(&self, image_id: i32, command: &PublishImageCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<(), BackendError, _>(|conn| {
            self.image_repository.reset_roles(conn, image_id)?;
            self.image_repository
                .assign_roles(conn, image_id, &command.roles)
        })
    }

    fn unpublish(&self, image_id: i32) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        self.image_repository.reset_roles(&mut conn, image_id)
    }

    fn delete(&self, image_id: i32) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        self.image_repository.delete(&mut conn, image_id)
    }
}

impl Injectable<(&DatabaseConnectionPool, &Data<dyn ImageRepository>), dyn ImageCommandService>
    for Implementation
{
    fn injectable(
        (pool, image_repository): (&DatabaseConnectionPool, &Data<dyn ImageRepository>),
    ) -> Data<dyn ImageCommandService> {
        let implementation = Self {
            pool: pool.clone(),
            image_repository: image_repository.clone(),
        };
        let arc: Arc<dyn ImageCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
