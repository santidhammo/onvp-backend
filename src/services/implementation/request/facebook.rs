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

//! The facebook service returns public requests about members. It is therefore
//! an isolated service, using an isolated repository which is tailored at
//! only showing that information which is relevant to members.

use crate::generic::result::BackendResult;
use crate::generic::search_helpers::create_like_string;
use crate::generic::storage::session::Session;
use crate::generic::{search_helpers, Injectable};
use crate::injection::ServiceDependencies;
use crate::model::interface::responses::FacebookResponse;
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::model::primitives::Role;
use crate::repositories::definitions::{
    AuthorizationRepository, FacebookRepository, MemberRepository, MusicalInstrumentRepository,
};
use crate::services::definitions::request::{FacebookRequestService, SearchController};
use actix_web::web::Data;
use serde::Serialize;
use std::sync::Arc;

pub struct Implementation {
    facebook_repository: Data<dyn FacebookRepository>,
    member_repository: Data<dyn MemberRepository>,
    authorization_repository: Data<dyn AuthorizationRepository>,
    musical_instrument_repository: Data<dyn MusicalInstrumentRepository>,
}

impl SearchController<FacebookResponse> for Implementation {
    fn search(
        &self,
        mut session: Session,
        params: &SearchParams,
    ) -> BackendResult<SearchResult<FacebookResponse>>
    where
        FacebookResponse: Serialize,
    {
        let term = create_like_string(params.term.clone().unwrap_or_default());
        let (total_count, page_size, results) =
            self.facebook_repository
                .search(&mut session, params.page_offset, &term)?;

        let rows: Vec<FacebookResponse> = results
            .iter()
            .map(|m| {
                let workgroup_names = self
                    .member_repository
                    .find_workgroups(&mut session, m.id)
                    .unwrap_or(vec![]);

                let roles = self
                    .authorization_repository
                    .find_composite_roles_by_member_id(&mut session, m.id)
                    .unwrap_or(vec![])
                    .iter()
                    .map(|r| *r)
                    .filter(|r| r != &Role::Operator && r != &Role::Public && r != &Role::Member)
                    .collect();

                let musical_instrument = m
                    .musical_instrument_id
                    .map(|id| {
                        self.musical_instrument_repository
                            .find_by_id(&mut session, id)
                            .ok()
                    })
                    .flatten();
                FacebookResponse::from((m, &musical_instrument, &workgroup_names, &roles))
            })
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

impl FacebookRequestService for Implementation {}

impl Injectable<ServiceDependencies, dyn FacebookRequestService> for Implementation {
    fn make(dependencies: &ServiceDependencies) -> Data<dyn FacebookRequestService> {
        let implementation = Self {
            facebook_repository: dependencies.facebook_repository.clone(),
            member_repository: dependencies.member_repository.clone(),
            authorization_repository: dependencies.authorization_repository.clone(),
            musical_instrument_repository: dependencies.musical_instrument_repository.clone(),
        };

        let arc: Arc<dyn FacebookRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
