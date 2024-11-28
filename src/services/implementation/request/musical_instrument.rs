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
use crate::generic::{search_helpers, Injectable};
use crate::injection::ServiceDependencies;
use crate::model::interface::responses::MusicalInstrumentResponse;
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::repositories::definitions::MusicalInstrumentRepository;
use crate::services::definitions::request::{MusicalInstrumentRequestService, SearchController};
use actix_web::web::Data;
use serde::Serialize;
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

impl SearchController<MusicalInstrumentResponse> for Implementation {
    fn search(
        &self,
        mut session: Session,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<MusicalInstrumentResponse>>
    where
        MusicalInstrumentResponse: Serialize,
    {
        let term = params.term.clone().unwrap_or_default();
        let (total_count, page_size, results) =
            self.musical_instrument_repository
                .search(&mut session, params.page_offset, &term)?;
        let rows: Vec<MusicalInstrumentResponse> = results
            .iter()
            .map(MusicalInstrumentResponse::from)
            .collect();
        let row_len = rows.len();
        Ok(SearchResult {
            total_count,
            page_offset: params.page_offset,
            page_count: search_helpers::calculate_page_count(page_size, total_count),
            rows,
            start: params.page_offset * page_size,
            end: (params.page_offset * page_size) + row_len,
        })
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
