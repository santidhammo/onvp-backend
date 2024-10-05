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

use crate::generic::result::{BackendError, BackendResult};
use crate::generic::storage::database::{DatabaseConnection, DatabaseConnectionPool};
use crate::model::storage::entities::{Member, MemberAddressDetail, MemberDetail};
use crate::schema;
use crate::schema::{member_address_details, member_details, members};
use diesel::prelude::*;
use log::info;

pub fn clean_late_non_activated_members(pool: DatabaseConnectionPool) -> BackendResult<()> {
    let mut conn = pool.get()?;

    conn.transaction::<_, BackendError, _>(|conn| {
        let activated_filter = schema::members::activated.eq(false);
        let activation_time_elapsed_filter =
            schema::members::activation_time.lt(chrono::Utc::now().naive_utc());
        let result = schema::members::table
            .select(schema::members::all_columns)
            .filter(activated_filter.and(activation_time_elapsed_filter))
            .load::<Member>(conn)?;

        let mut deleted = 0;

        for member in result {
            {
                let details = find_detail_by_detail_id(conn, &member.member_details_id)?;
                info!(
                    "Deleting member: {} with name: {}",
                    member.id,
                    details.name()
                );
            }

            let result =
                diesel::delete(members::table.filter(members::id.eq(member.id))).execute(conn)?;

            if result != 1 {
                return Err(BackendError::not_enough_records());
            }
            delete_member_detail_by_id(conn, member.member_details_id)?;
            delete_member_address_detail_by_id(conn, member.member_address_details_id)?;
            deleted += 1;
        }

        info!("Deleted {deleted} members");
        Ok(())
    })?;
    Ok(())
}

pub fn find_detail_by_detail_id(
    conn: &mut DatabaseConnection,
    detail_id: &i32,
) -> BackendResult<MemberDetail> {
    let id_filter = member_details::id.eq(detail_id);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(id_filter)
        .first::<MemberDetail>(conn)?)
}

pub fn delete_member_address_detail_by_id(
    conn: &mut DatabaseConnection,
    address_details_id: i32,
) -> BackendResult<()> {
    let details = member_address_details::table
        .select(member_address_details::all_columns)
        .filter(member_address_details::id.eq(address_details_id))
        .load::<MemberAddressDetail>(conn)?;
    let maybe_first_error = details
        .iter()
        .map(|detail| {
            diesel::delete(
                member_address_details::table.filter(member_address_details::id.eq(detail.id)),
            )
            .execute(conn)
        })
        .filter(|r| r.is_err())
        .map(|r| r.unwrap_err())
        .nth(0);
    match maybe_first_error {
        Some(first_error) => Err(first_error.into()),
        None => Ok(()),
    }
}

pub fn delete_member_detail_by_id(
    conn: &mut DatabaseConnection,
    details_id: i32,
) -> BackendResult<()> {
    let address_details = member_details::table
        .select(member_details::all_columns)
        .filter(member_details::id.eq(details_id))
        .load::<MemberDetail>(conn)?;
    let maybe_first_error = address_details
        .iter()
        .map(|detail| {
            diesel::delete(member_details::table.filter(member_details::id.eq(detail.id)))
                .execute(conn)
        })
        .filter(|r| r.is_err())
        .map(|r| r.unwrap_err())
        .nth(0);
    match maybe_first_error {
        Some(first_error) => Err(first_error.into()),
        None => Ok(()),
    }
}
