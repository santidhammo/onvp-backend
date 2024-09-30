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

use crate::model::database::prelude::*;
use crate::model::interface::prelude::*;
use crate::model::security::Role;
use crate::schema::*;
use crate::{dal, Error, Result};
use diesel::prelude::*;

pub(crate) fn register(conn: &mut DbConnection, command: &WorkgroupRegisterCommand) -> Result<()> {
    diesel::insert_into(workgroups::table)
        .values(command)
        .execute(conn)?;
    Ok(())
}

pub(crate) fn associate_role(
    conn: &mut DbConnection,
    workgroup_id: &i32,
    role: &Role,
) -> Result<()> {
    // It is not possible to associate the public role or member role
    if role == &Role::Public || role == &Role::Member {
        return Err(Error::bad_request());
    }

    diesel::insert_into(workgroup_role_associations::table)
        .values((
            workgroup_role_associations::workgroup_id.eq(workgroup_id),
            workgroup_role_associations::system_role.eq(role),
        ))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn dissociate_role(
    conn: &mut DbConnection,
    workgroup_id: &i32,
    role: &Role,
) -> Result<()> {
    let deleted_rows = diesel::delete(workgroup_role_associations::table)
        .filter(
            workgroup_role_associations::workgroup_id
                .eq(workgroup_id)
                .and(workgroup_role_associations::system_role.eq(role)),
        )
        .execute(conn)?;
    if deleted_rows > 0 {
        Ok(())
    } else {
        Err(Error::not_enough_records())
    }
}

pub(crate) fn search(
    conn: &mut DbConnection,
    name: &String,
    page_size: usize,
    page_offset: usize,
) -> Result<SearchResult<WorkgroupResponse>> {
    let like_search_string = dal::create_like_string(name);

    conn.transaction::<SearchResult<WorkgroupResponse>, Error, _>(|conn| {
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
