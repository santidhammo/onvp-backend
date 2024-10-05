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

use crate::generic::result::BackendResult;

use crate::model::storage::entities::{Member, MemberAddressDetail, MemberDetail};
use crate::schema::*;
use diesel::prelude::*;

pub fn find_by_id(conn: &mut DbConnection, id: &i32) -> BackendResult<Member> {
    Ok(members::table
        .select(members::all_columns)
        .filter(members::id.eq(id))
        .first::<Member>(conn)?)
}

pub fn find_detail_by_detail_id(
    conn: &mut DbConnection,
    detail_id: &i32,
) -> BackendResult<MemberDetail> {
    let id_filter = member_details::id.eq(detail_id);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(id_filter)
        .first::<MemberDetail>(conn)?)
}

pub fn delete_member_address_detail_by_id(
    conn: &mut DbConnection,
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

pub fn delete_member_detail_by_id(conn: &mut DbConnection, details_id: i32) -> BackendResult<()> {
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

pub fn with_detail_response_by_id(
    conn: &mut DbConnection,
    id: i32,
) -> BackendResult<(Member, MemberDetail)> {
    let filter = members::id.eq(id);
    Ok(members::table
        .inner_join(member_details::table)
        .filter(filter)
        .select((Member::as_select(), MemberDetail::as_select()))
        .first(conn)?)
}
