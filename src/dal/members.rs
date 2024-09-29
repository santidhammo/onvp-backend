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
use crate::generic::security::{
    generate_encoded_nonce, FIRST_OPERATOR_ACTIVATION_MINUTES, MEMBER_ACTIVATION_MINUTES,
};

use crate::model::prelude::*;
use crate::schema::*;
use crate::{dal, Error, Result};
use chrono::TimeDelta;
use diesel::prelude::*;
use std::collections::HashSet;
use std::ops::Add;

pub fn has_operators(conn: &mut DbConnection) -> Result<bool> {
    let filter = member_role_associations::system_role.eq(Role::Operator);
    let count: i64 = member_role_associations::table
        .filter(filter)
        .count()
        .get_result(conn)?;

    Ok(count != 0)
}

pub fn register_first_operator(
    conn: &mut DbConnection,
    register_command: &FirstOperatorRegisterCommand,
    activation_string: &str,
) -> Result<()> {
    conn.transaction::<_, Error, _>(|conn| {
        let member_id = create_inactive_member(
            conn,
            register_command,
            activation_string,
            *FIRST_OPERATOR_ACTIVATION_MINUTES,
            Role::Member,
        )?;

        associate_role(conn, &member_id, &Role::Operator)?;
        Ok(())
    })
    .map_err(|e| e.into())
}

pub fn register(
    conn: &mut DbConnection,
    register_command: &MemberRegisterCommand,
    activation_string: &str,
) -> Result<()> {
    conn.transaction::<_, Error, _>(|conn| {
        let _ = create_inactive_member(
            conn,
            register_command,
            activation_string,
            *MEMBER_ACTIVATION_MINUTES,
            Role::Member,
        )?;
        Ok(())
    })
    .map_err(|e| e.into())
}

pub fn get_member_by_activation_string(
    conn: &mut DbConnection,
    activation_string: &str,
) -> Result<MemberEntity> {
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
        .first::<MemberEntity>(conn)?)
}

pub fn get_member_by_id(conn: &mut DbConnection, id: &i32) -> Result<MemberEntity> {
    Ok(members::table
        .select(members::all_columns)
        .filter(members::id.eq(id))
        .first::<MemberEntity>(conn)?)
}

pub fn get_member_by_email_address(
    conn: &mut DbConnection,
    email_address: &str,
) -> Result<MemberEntity> {
    let member_details = get_member_detail_by_email_address(conn, email_address)?;

    let activated_filter = members::activated.eq(true);
    let details_filter = members::member_details_id.eq(member_details.id);

    Ok(members::table
        .select(members::all_columns)
        .filter(activated_filter.and(details_filter))
        .first::<MemberEntity>(conn)?)
}

pub fn get_member_detail_by_id(
    conn: &mut DbConnection,
    member_details_id: &i32,
) -> Result<MemberDetailEntity> {
    let id_filter = member_details::id.eq(member_details_id);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(id_filter)
        .first::<MemberDetailEntity>(conn)?)
}

pub fn get_member_detail_by_email_address(
    conn: &mut DbConnection,
    email_address: &str,
) -> Result<MemberDetailEntity> {
    let email_address_filter = member_details::email_address.eq(email_address);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(email_address_filter)
        .first::<MemberDetailEntity>(conn)?)
}

pub fn delete_member_address_detail_by_id(
    conn: &mut DbConnection,
    member_address_details_id: i32,
) -> Result<()> {
    let details = member_address_details::table
        .select(member_address_details::all_columns)
        .filter(member_address_details::id.eq(member_address_details_id))
        .load::<MemberAddressDetailEntity>(conn)?;
    let maybe_first_error = details
        .iter()
        .map(|detail| diesel::delete(detail).execute(conn))
        .filter(|r| r.is_err())
        .map(|r| r.unwrap_err())
        .nth(0);
    match maybe_first_error {
        Some(first_error) => Err(first_error.into()),
        None => Ok(()),
    }
}

pub fn delete_member_detail_by_id(conn: &mut DbConnection, member_details_id: i32) -> Result<()> {
    let address_details = member_details::table
        .select(member_details::all_columns)
        .filter(member_details::id.eq(member_details_id))
        .load::<MemberDetailEntity>(conn)?;
    let maybe_first_error = address_details
        .iter()
        .map(|detail| diesel::delete(detail).execute(conn))
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
) -> Result<Vec<Role>> {
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

pub fn activate(conn: &mut DbConnection, member: &MemberEntity) -> Result<()> {
    diesel::update(members::table)
        .filter(members::id.eq(member.id))
        .set(members::activated.eq(true))
        .execute(conn)?;
    Ok(())
}

pub fn get_member_with_detail_by_id(
    conn: &mut DbConnection,
    id: i32,
) -> Result<MemberWithDetailLogicalEntity> {
    let filter = members::id.eq(id);
    let result: (MemberEntity, MemberDetailEntity) = members::table
        .inner_join(member_details::table)
        .filter(filter)
        .select((MemberEntity::as_select(), MemberDetailEntity::as_select()))
        .first(conn)?;
    Ok(MemberWithDetailLogicalEntity::from(&result))
}

pub fn find_with_details_by_search_string(
    conn: &mut DbConnection,
    search_string: &String,
    page_size: usize,
    page_offset: usize,
) -> Result<SearchResult<MemberWithDetailLogicalEntity>> {
    let like_search_string = dal::create_like_string(search_string);

    conn.transaction::<SearchResult<MemberWithDetailLogicalEntity>, Error, _>(|conn| {
        // ILIKE is only supported on PostgreSQL
        let (total_count, member_details) = match conn {
            DbConnection::PostgreSQL(ref mut conn) => {
                let filter = member_details::first_name
                    .ilike(&like_search_string)
                    .or(member_details::last_name.ilike(&like_search_string))
                    .or(member_details::email_address.ilike(&like_search_string));

                let total_count: usize = member_details::dsl::member_details
                    .filter(filter)
                    .count()
                    .get_result::<i64>(conn)? as usize;

                let member_details: Vec<(MemberEntity, MemberDetailEntity)> = members::table
                    .inner_join(member_details::table)
                    .filter(filter)
                    .order_by(member_details::last_name)
                    .order_by(member_details::first_name)
                    .limit(page_size as i64)
                    .offset((page_offset * page_size) as i64)
                    .select((MemberEntity::as_select(), MemberDetailEntity::as_select()))
                    .load(conn)?;

                (total_count, member_details)
            }

            DbConnection::SQLite(ref mut conn) => {
                let filter = member_details::first_name
                    .like(&like_search_string)
                    .or(member_details::last_name.like(&like_search_string))
                    .or(member_details::email_address.like(&like_search_string));

                let total_count: usize = member_details::dsl::member_details
                    .filter(filter)
                    .count()
                    .get_result::<i64>(conn)? as usize;

                let member_details: Vec<(MemberEntity, MemberDetailEntity)> = members::table
                    .inner_join(member_details::table)
                    .filter(filter)
                    .order_by(member_details::last_name)
                    .limit(page_size as i64)
                    .offset(page_offset as i64)
                    .select((MemberEntity::as_select(), MemberDetailEntity::as_select()))
                    .load(conn)?;

                (total_count, member_details)
            }
        };
        Ok(SearchResult {
            total_count,
            page_offset,
            page_count: dal::calculate_page_count(page_size, total_count),
            rows: member_details
                .iter()
                .map(MemberWithDetailLogicalEntity::from)
                .collect(),
        })
    })
}

pub(crate) fn update(
    conn: &mut DbConnection,
    member_id: &i32,
    command: &MemberUpdateCommand,
) -> Result<()> {
    conn.transaction::<_, Error, _>(|conn| {
        let filter = members::id.eq(member_id);

        diesel::update(members::table)
            .filter(filter)
            .set(members::musical_instrument_id.eq(command.musical_instrument_id.clone()))
            .execute(conn)?;

        let result: MemberEntity = members::table
            .inner_join(member_details::table)
            .filter(filter)
            .select(MemberEntity::as_select())
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
) -> Result<()> {
    conn.transaction::<_, Error, _>(|conn| {
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
) -> Result<()> {
    diesel::update(members::table)
        .filter(members::id.eq(member_id))
        .set(members::picture_asset_id.eq(picture_asset_id))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn associate_role(conn: &mut DbConnection, member_id: &i32, role: &Role) -> Result<()> {
    // It is not possible to associate the public role
    if role == &Role::Public {
        return Err(Error::bad_request());
    }

    diesel::insert_into(member_role_associations::table)
        .values((
            member_role_associations::member_id.eq(member_id),
            member_role_associations::system_role.eq(role),
        ))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn dissociate_role(conn: &mut DbConnection, member_id: &i32, role: &Role) -> Result<()> {
    // It is not possible to dissociate the member role
    if role == &Role::Member {
        return Err(Error::bad_request());
    }

    let deleted_rows = diesel::delete(member_role_associations::table)
        .filter(
            member_role_associations::member_id
                .eq(member_id)
                .and(member_role_associations::system_role.eq(role)),
        )
        .execute(conn)?;
    if deleted_rows > 0 {
        Ok(())
    } else {
        Err(Error::not_enough_records())
    }
}

pub(crate) fn role_list(conn: &mut DbConnection, member_id: &i32) -> Result<Vec<Role>> {
    let filter = member_role_associations::member_id.eq(member_id);
    let role_associations: Vec<MemberRoleAssociation> = member_role_associations::table
        .filter(filter)
        .select(MemberRoleAssociation::as_select())
        .load(conn)?;
    Ok(role_associations.iter().map(|ra| ra.system_role).collect())
}

pub fn create_inactive_member<S: AsAllMemberRegisterSubCommands>(
    conn: &mut DbConnection,
    all_member_register_subcommands: &S,
    activation_string: &str,
    activation_delta: TimeDelta,
    role: Role,
) -> crate::Result<i32> {
    conn.transaction::<i32, Error, _>(|conn| {
        let member_address_detail_id: i32 = diesel::insert_into(member_address_details::table)
            .values(all_member_register_subcommands.member_address_detail())
            .returning(member_address_details::id)
            .get_result(conn)?;

        let member_detail_id: i32 = diesel::insert_into(member_details::table)
            .values(all_member_register_subcommands.member_detail())
            .returning(member_details::id)
            .get_result(conn)?;

        let now = chrono::Utc::now();

        let member_id: i32 = diesel::insert_into(members::table)
            .values((
                members::member_address_details_id.eq(member_address_detail_id),
                members::member_details_id.eq(member_detail_id),
                members::activation_string.eq(activation_string.to_string()),
                members::activation_time.eq(now.add(activation_delta).naive_utc()),
                members::creation_time.eq(now.naive_utc()),
                members::nonce.eq(generate_encoded_nonce()),
            ))
            .returning(members::id)
            .get_result(conn)?;

        diesel::insert_into(member_role_associations::table)
            .values((
                member_role_associations::member_id.eq(member_id),
                member_role_associations::system_role.eq(role),
            ))
            .execute(conn)?;
        Ok(member_id)
    })
}
