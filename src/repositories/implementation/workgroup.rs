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
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::storage::database::DatabaseConnection;
use crate::generic::{search_helpers, Injectable};
use crate::model::storage::entities::Workgroup;
use crate::repositories::definitions::WorkgroupRepository;
use crate::schema::workgroups;
use actix_web::web::Data;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

pub struct Implementation {
    page_size: usize,
}

impl WorkgroupRepository for Implementation {
    fn register(&self, conn: &mut DatabaseConnection, workgroup: Workgroup) -> BackendResult<i32> {
        let result: usize = diesel::insert_into(workgroups::table)
            .values(workgroup)
            .returning(workgroups::id)
            .execute(conn)?;
        Ok(result as i32)
    }

    fn find_by_id(&self, conn: &mut DatabaseConnection, id: i32) -> BackendResult<Workgroup> {
        let workgroup: Workgroup = workgroups::table
            .filter(workgroups::id.eq(id))
            .select(Workgroup::as_select())
            .first(conn)?;
        Ok(workgroup)
    }

    fn save(&self, conn: &mut DatabaseConnection, workgroup: Workgroup) -> BackendResult<()> {
        diesel::update(workgroups::table)
            .filter(workgroups::id.eq(workgroup.id))
            .set(workgroup)
            .execute(conn)?;
        Ok(())
    }

    fn search(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<Workgroup>)> {
        let like_search_string = search_helpers::create_like_string(term);
        let result = conn.transaction::<(usize, usize, Vec<Workgroup>), BackendError, _>(|conn| {
            // ILIKE is only supported on PostgreSQL
            match conn {
                DatabaseConnection::PostgreSQL(ref mut conn) => {
                    use diesel::PgTextExpressionMethods;
                    let filter = workgroups::name.ilike(&like_search_string);

                    let total_count: usize = workgroups::table
                        .filter(filter)
                        .count()
                        .get_result::<i64>(conn)?
                        as usize;

                    let workgroups: Vec<Workgroup> = workgroups::table
                        .filter(filter)
                        .order_by(workgroups::name)
                        .limit(self.page_size as i64)
                        .offset((page_offset * self.page_size) as i64)
                        .select(Workgroup::as_select())
                        .load(conn)?;

                    Ok((total_count, self.page_size, workgroups))
                }

                DatabaseConnection::SQLite(ref mut conn) => {
                    use diesel::TextExpressionMethods;
                    let filter = workgroups::name.like(&like_search_string);

                    let total_count: usize = workgroups::table
                        .filter(filter)
                        .count()
                        .get_result::<i64>(conn)?
                        as usize;

                    let workgroups: Vec<Workgroup> = workgroups::table
                        .filter(filter)
                        .order_by(workgroups::name)
                        .limit(self.page_size as i64)
                        .offset((page_offset * self.page_size) as i64)
                        .select(Workgroup::as_select())
                        .load(conn)?;

                    Ok((total_count, self.page_size, workgroups))
                }
            }
        });
        result
    }
}

impl Injectable<(), dyn WorkgroupRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn WorkgroupRepository> {
        let arc: Arc<dyn WorkgroupRepository> = Arc::new(Self {
            page_size: *SEARCH_PAGE_SIZE,
        });
        Data::from(arc)
    }
}
