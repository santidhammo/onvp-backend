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

use crate::dal::DbConnection;

use crate::dal;
use crate::generic::result::{BackendError, BackendResult};
use crate::model::interface::prelude::*;
use crate::model::storage::prelude::*;
use crate::schema::*;
use diesel::prelude::*;

pub(crate) fn register(
    conn: &mut DbConnection,
    command: &WorkgroupRegisterCommand,
) -> BackendResult<()> {
    diesel::insert_into(workgroups::table)
        .values(Workgroup::from(command))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn search(
    conn: &mut DbConnection,
    name: &String,
    page_size: usize,
    page_offset: usize,
) -> BackendResult<SearchResult<WorkgroupResponse>> {
    let like_search_string = dal::create_like_string(name);

    conn.transaction::<SearchResult<WorkgroupResponse>, BackendError, _>(|conn| {
        // ILIKE is only supported on PostgreSQL
        let (total_count, workgroups) = match conn {
            DbConnection::PostgreSQL(ref mut conn) => {
                let filter = workgroups::name.ilike(&like_search_string);

                let total_count: usize = workgroups::table
                    .filter(filter)
                    .count()
                    .get_result::<i64>(conn)? as usize;

                let workgroups: Vec<Workgroup> = workgroups::table
                    .filter(filter)
                    .order_by(workgroups::name)
                    .limit(page_size as i64)
                    .offset((page_offset * page_size) as i64)
                    .select(Workgroup::as_select())
                    .load(conn)?;

                (total_count, workgroups)
            }

            DbConnection::SQLite(ref mut conn) => {
                let filter = workgroups::name.like(&like_search_string);

                let total_count: usize = workgroups::table
                    .filter(filter)
                    .count()
                    .get_result::<i64>(conn)? as usize;

                let workgroups: Vec<Workgroup> = workgroups::table
                    .filter(filter)
                    .order_by(workgroups::name)
                    .limit(page_size as i64)
                    .offset((page_offset * page_size) as i64)
                    .select(Workgroup::as_select())
                    .load(conn)?;

                (total_count, workgroups)
            }
        };
        Ok(SearchResult {
            total_count,
            page_offset,
            page_count: dal::calculate_page_count(page_size, total_count),
            rows: workgroups.iter().map(WorkgroupResponse::from).collect(),
        })
    })
}
