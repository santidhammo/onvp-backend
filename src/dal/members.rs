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

use crate::generic::result::{BackendError, BackendResult};
use crate::model::interface::prelude::*;
use crate::model::prelude::*;
use crate::model::storage::prelude::*;
use crate::schema::*;
use diesel::prelude::*;
use std::collections::HashSet;

pub fn find_by_activation_string(2
    conn: &mut DbConnection,
    activation_string: &str,
) -> BackendResult<Member> {
    let activated_filter = members::activated.eq(false);
    let activation_time_filter = members::activation_time.gt(chrono::Utc::now().naive_utc());
    let activation_string_filter = members::activation_string.eq(activation_string);

    Ok(members::table
        .select(members::all_columns)
        .filter(
            activated_filter
                .and(activation_time_filter)
                .and(activation_string_filter),
        )
        .first::<Member>(conn)?)
}

pub fn find_by_id(conn: &mut DbConnection, id: &i32) -> BackendResult<Member> {
    Ok(members::table
        .select(members::all_columns)
        .filter(members::id.eq(id))
        .first::<Member>(conn)?)
}

pub fn find_by_email_address(
    conn: &mut DbConnection,
    email_address: &str,
) -> BackendResult<Member> {
    let member_details = get_member_detail_by_email_address(conn, email_address)?;

    let activated_filter = members::activated.eq(true);
    let details_filter = members::member_details_id.eq(member_details.id);

    Ok(members::table
        .select(members::all_columns)
        .filter(activated_filter.and(details_filter))
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

pub fn get_member_detail_by_email_address(
    conn: &mut DbConnection,
    email_address: &str,
) -> BackendResult<MemberDetail> {
    let email_address_filter = member_details::email_address.eq(email_address);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(email_address_filter)
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

pub(crate) fn get_member_roles_by_member_id(
    conn: &mut DbConnection,
    member_id: &i32,
) -> BackendResult<Vec<Role>> {
    let mut roles = member_role_associations::dsl::member_role_associations
        .select(member_role_associations::system_role)
        .filter(member_role_associations::member_id.eq(member_id))
        .load::<Role>(conn)?;

    let workgroup_ids = workgroup_member_relationships::dsl::workgroup_member_relationships
        .select(workgroup_member_relationships::workgroup_id)
        .filter(workgroup_member_relationships::member_id.eq(member_id))
        .load::<i32>(conn)?;

    for workgroup_id in workgroup_ids {
        let workgroup_roles = workgroup_role_associations::dsl::workgroup_role_associations
            .select(workgroup_role_associations::system_role)
            .filter(workgroup_role_associations::workgroup_id.eq(workgroup_id))
            .load::<Role>(conn)?;

        roles.extend(workgroup_roles);
    }
    roles.push(Role::Public);
    roles.push(Role::Member);

    let result: HashSet<Role> = HashSet::from_iter(roles);

    Ok(result.iter().map(|v| *v).collect())
}

pub fn activate(conn: &mut DbConnection, member_id: &i32) -> BackendResult<()> {
    diesel::update(members::table)
        .filter(members::id.eq(member_id))
        .set(members::activated.eq(true))
        .execute(conn)?;
    Ok(())
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

pub(crate) fn update(
    conn: &mut DbConnection,
    member_id: &i32,
    command: &MemberUpdateCommand,
) -> BackendResult<()> {
    conn.transaction::<_, BackendError, _>(|conn| {
        let filter = members::id.eq(member_id);

        diesel::update(members::table)
            .filter(filter)
            .set(members::musical_instrument_id.eq(command.musical_instrument_id.clone()))
            .execute(conn)?;

        let result: Member = members::table
            .inner_join(member_details::table)
            .filter(filter)
            .select(Member::as_select())
            .first(conn)?;

        diesel::update(member_details::table)
            .filter(member_details::id.eq(result.member_details_id))
            .set((
                member_details::first_name.eq(command.first_name.clone()),
                member_details::last_name.eq(command.last_name.clone()),
                member_details::phone_number.eq(command.phone_number.clone()),
                member_details::email_address.eq(command.email_address.clone()),
            ))
            .execute(conn)?;

        Ok(())
    })
}

pub(crate) fn update_address(
    conn: &mut DbConnection,
    member_id: &i32,
    command: &MemberUpdateAddressCommand,
) -> BackendResult<()> {
    conn.transaction::<_, BackendError, _>(|conn| {
        let member_address_details_id: i32 = members::table
            .filter(members::id.eq(member_id))
            .select(members::member_address_details_id)
            .first(conn)?;

        diesel::update(member_address_details::table)
            .filter(member_address_details::id.eq(member_address_details_id))
            .set((
                member_address_details::street.eq(command.street.to_string()),
                member_address_details::house_number.eq(command.house_number.clone()),
                member_address_details::house_number_postfix
                    .eq(command.house_number_postfix.clone()),
                member_address_details::postal_code.eq(command.postal_code.clone()),
                member_address_details::domicile.eq(command.domicile.clone()),
            ))
            .execute(conn)?;

        Ok(())
    })
}

pub(crate) fn store_member_picture_asset_id(
    conn: &mut DbConnection,
    member_id: &i32,
    picture_asset_id: &str,
) -> BackendResult<()> {
    diesel::update(members::table)
        .filter(members::id.eq(member_id))
        .set(members::picture_asset_id.eq(picture_asset_id))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn role_list(conn: &mut DbConnection, member_id: &i32) -> BackendResult<Vec<Role>> {
    let filter = member_role_associations::member_id.eq(member_id);
    let role_associations: Vec<MemberRoleAssociation> = member_role_associations::table
        .filter(filter)
        .select(MemberRoleAssociation::as_select())
        .load(conn)?;
    Ok(role_associations.iter().map(|ra| ra.system_role).collect())
}
