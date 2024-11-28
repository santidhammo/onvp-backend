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
use crate::generic::lazy::SEARCH_PAGE_SIZE;
use crate::generic::result::BackendResult;
use crate::generic::storage::session::Session;
use crate::generic::{search_helpers, Injectable};
use crate::model::storage::entities::MusicalInstrument;
use crate::repositories::definitions::MusicalInstrumentRepository;
use crate::schema::*;
use actix_web::web::Data;
use diesel::{ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

pub struct Implementation {
    page_size: usize,
}

impl MusicalInstrumentRepository for Implementation {
    fn create(&self, session: &mut Session, instrument: MusicalInstrument) -> BackendResult<()> {
        session.run(|conn| {
            let _ = diesel::insert_into(musical_instruments::table)
                .values(instrument)
                .returning(musical_instruments::id)
                .execute(conn)?;
            Ok(())
        })
    }

    fn update(&self, session: &mut Session, instrument: MusicalInstrument) -> BackendResult<()> {
        session.run(|conn| {
            diesel::update(musical_instruments::table)
                .filter(musical_instruments::id.eq(instrument.id))
                .set(instrument)
                .execute(conn)?;
            Ok(())
        })
    }

    fn delete(&self, session: &mut Session, instrument_id: i32) -> BackendResult<()> {
        session.run(|conn| {
            diesel::delete(musical_instruments::table)
                .filter(musical_instruments::id.eq(instrument_id))
                .execute(conn)?;
            Ok(())
        })
    }

    fn find_by_id(&self, session: &mut Session, image_id: i32) -> BackendResult<MusicalInstrument> {
        session.run(|conn| {
            let image = musical_instruments::table
                .filter(musical_instruments::id.eq(image_id))
                .select(MusicalInstrument::as_select())
                .first::<MusicalInstrument>(conn)?;
            Ok(image)
        })
    }

    fn search(
        &self,
        session: &mut Session,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<MusicalInstrument>)> {
        let like_search_string = search_helpers::create_like_string(term);
        let (total_count, pages) = session.run(|conn| {
            let total_count: usize = musical_instruments::table
                .filter(&musical_instruments::name.ilike(&like_search_string))
                .count()
                .get_result::<i64>(conn)? as usize;

            let result: Vec<MusicalInstrument> = musical_instruments::table
                .filter(&musical_instruments::name.ilike(&like_search_string))
                .order_by(musical_instruments::name)
                .limit(self.page_size as i64)
                .offset((page_offset * self.page_size) as i64)
                .select(MusicalInstrument::as_select())
                .load(conn)?;

            Ok((total_count, result))
        })?;
        Ok((total_count, self.page_size, pages))
    }
}

impl Injectable<(), dyn MusicalInstrumentRepository> for Implementation {
    fn make(_: &()) -> Data<dyn MusicalInstrumentRepository> {
        let arc: Arc<dyn MusicalInstrumentRepository> = Arc::new(Self {
            page_size: *SEARCH_PAGE_SIZE,
        });
        Data::from(arc)
    }
}
