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
use crate::model::interface::commands::{
    RegisterMusicalInstrumentCommand, UpdateMusicalInstrumentCommand,
};
use crate::model::storage::entities::MusicalInstrument;
use crate::repositories::definitions::MusicalInstrumentRepository;
use crate::services::definitions::command::MusicalInstrumentCommandService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    musical_instrument_repository: Data<dyn MusicalInstrumentRepository>,
}

impl MusicalInstrumentCommandService for Implementation {
    fn register(
        &self,
        mut session: Session,
        command: &RegisterMusicalInstrumentCommand,
    ) -> BackendResult<()> {
        let instrument = MusicalInstrument::from(command);
        self.musical_instrument_repository
            .create(&mut session, instrument)
    }

    fn update(
        &self,
        mut session: Session,
        musical_instrument_id: i32,
        command: &UpdateMusicalInstrumentCommand,
    ) -> BackendResult<()> {
        let origin = self
            .musical_instrument_repository
            .find_by_id(&mut session, musical_instrument_id)?;
        let page = MusicalInstrument::from((&origin, command));
        self.musical_instrument_repository
            .update(&mut session, page)
    }

    fn delete(&self, mut session: Session, musical_instrument_id: i32) -> BackendResult<()> {
        self.musical_instrument_repository
            .delete(&mut session, musical_instrument_id)
    }
}

impl Injectable<ServiceDependencies, dyn MusicalInstrumentCommandService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn MusicalInstrumentCommandService> {
        let implementation = Self {
            musical_instrument_repository: dependencies.musical_instrument_repository.clone(),
        };
        let arc: Arc<dyn MusicalInstrumentCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
