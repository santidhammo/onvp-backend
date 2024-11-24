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
use crate::model::interface::commands::{ImageUploadCommand, PublishImageCommand};
use crate::model::storage::entities::Image;
use crate::repositories::definitions::ImageRepository;
use crate::services::definitions::command::ImageCommandService;
use actix_web::web::Data;
use image::EncodableLayout;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

pub struct Implementation {
    image_repository: Data<dyn ImageRepository>,
}

impl ImageCommandService for Implementation {
    fn upload(&self, mut session: Session, command: &ImageUploadCommand) -> BackendResult<String> {
        let image = Image::from(command);
        let asset = image.asset.clone();
        self.image_repository.create(&mut session, image)?;
        let pb = crate::path_for_asset(&asset)?;
        let mut w = OpenOptions::new().write(true).create_new(true).open(&pb)?;
        w.write(&command.data.as_bytes())?;
        Ok(asset)
    }

    fn publish(
        &self,
        mut session: Session,
        image_id: i32,
        command: &PublishImageCommand,
    ) -> BackendResult<()> {
        self.image_repository.reset_roles(&mut session, image_id)?;
        self.image_repository
            .assign_roles(&mut session, image_id, &command.roles)
    }

    fn unpublish(&self, mut session: Session, image_id: i32) -> BackendResult<()> {
        self.image_repository.reset_roles(&mut session, image_id)
    }

    fn delete(&self, mut session: Session, image_id: i32) -> BackendResult<()> {
        self.image_repository.delete(&mut session, image_id)
    }
}

impl Injectable<ServiceDependencies, dyn ImageCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn ImageCommandService> {
        let implementation = Self {
            image_repository: dependencies.image_repository.clone(),
        };
        let arc: Arc<dyn ImageCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
