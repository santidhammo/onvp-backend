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
use crate::model::interface::responses::MusicalInstrumentResponse;
use crate::repositories::definitions::MusicalInstrumentRepository;
use crate::services::definitions::request::MusicalInstrumentRequestService;
use actix_web::web::Data;
use std::sync::Arc;

pub struct Implementation {
    musical_instrument_repository: Data<dyn MusicalInstrumentRepository>,
}

impl MusicalInstrumentRequestService for Implementation {
    fn find_by_id(
        &self,
        mut session: Session,
        image_id: i32,
    ) -> BackendResult<MusicalInstrumentResponse> {
        let musical_instrument = self
            .musical_instrument_repository
            .find_by_id(&mut session, image_id)?;
        Ok(MusicalInstrumentResponse::from(&musical_instrument))
    }
}

impl Injectable<ServiceDependencies, dyn MusicalInstrumentRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn MusicalInstrumentRequestService> {
        let implementation = Self {
            musical_instrument_repository: dependencies.musical_instrument_repository.clone(),
        };
        let arc: Arc<dyn MusicalInstrumentRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
