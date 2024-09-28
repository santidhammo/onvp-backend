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
use crate::model::commands::WorkgroupRegisterCommand;
use crate::model::security::Role;
use crate::schema::*;
use crate::{Error, Result};
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
