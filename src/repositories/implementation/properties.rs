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
use crate::repositories::definitions::PropertiesRepository;
use crate::schema::*;
use actix_web::web::Data;
use diesel::prelude::*;
use std::sync::Arc;

pub struct Implementation {}

impl PropertiesRepository for Implementation {
    fn maybe_int_property(&self, session: &mut Session, key: &str) -> Option<i32> {
        let result = session.run(|conn| {
            Ok(properties::table
                .filter(properties::key.eq(key))
                .select(properties::value)
                .first::<String>(conn)?)
        });
        if let Ok(value) = result {
            if let Ok(value) = value.parse() {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn set_int_property(
        &self,
        session: &mut Session,
        key: &str,
        value: Option<i32>,
    ) -> BackendResult<()> {
        session.run(|conn| {
            let result: usize = properties::table
                .filter(properties::key.eq(key))
                .count()
                .get_result::<i64>(conn)? as usize;

            if result > 0 {
                if let Some(value) = value {
                    diesel::update(properties::table)
                        .filter(properties::key.eq(key))
                        .set(properties::value.eq(value.to_string()))
                        .execute(conn)?;
                } else {
                    diesel::delete(properties::table)
                        .filter(properties::key.eq(key))
                        .execute(conn)?;
                }
            } else {
                if let Some(value) = value {
                    diesel::insert_into(properties::table)
                        .values((
                            properties::key.eq(key),
                            properties::value.eq(value.to_string()),
                        ))
                        .execute(conn)?;
                }
            }

            Ok(())
        })
    }
}

impl Injectable<(), dyn PropertiesRepository> for Implementation {
    fn make(_: &()) -> Data<dyn PropertiesRepository> {
        let arc: Arc<dyn PropertiesRepository> = Arc::new(Self {});
        Data::from(arc)
    }
}
