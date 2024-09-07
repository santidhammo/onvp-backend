use crate::dal::DbConnection;
use crate::model::members::MemberAddressDetails;
use crate::model::members::{Member, MemberDetails};
use crate::model::security::Role;
use crate::model::setup::FirstOperator;
use crate::schema::member_address_details;
use crate::schema::member_details;
use crate::schema::member_role_associations;
use crate::schema::members;
use crate::security::generate_encoded_nonce;
use crate::{dal, Error};
use chrono::TimeDelta;
use diesel::prelude::*;
use std::ops::Add;

pub fn has_operators(conn: &mut dal::DbConnection) -> Result<bool, Error> {
    let count: i64 = member_role_associations::dsl::member_role_associations
        .filter(member_role_associations::dsl::system_role.eq(Role::Operator))
        .count()
        .get_result(conn)?;

    Ok(count != 0)
}

pub fn create_first_operator(
    conn: &mut DbConnection,
    operator: &FirstOperator,
    activation_string: &str,
) -> Result<(), Error> {
    conn.transaction::<_, Error, _>(|conn| {
        let data = MemberAddressDetails {
            id: 0,
            street: operator.street.clone(),
            house_number: operator.house_number.clone(),
            house_number_postfix: operator.house_number_postfix.clone(),
            postal_code: operator.postal_code.clone(),
            domicile: operator.domicile.clone(),
        };
        let mad_id: i32 = diesel::insert_into(member_address_details::dsl::member_address_details)
            .values(&data)
            .returning(member_address_details::dsl::id)
            .get_result(conn)?;

        let data = MemberDetails {
            id: 0,
            first_name: operator.first_name.clone(),
            last_name: operator.last_name.clone(),
            email_address: operator.email_address.clone(),
            phone_number: operator.phone_number.clone(),
        };

        let md_id: i32 = diesel::insert_into(member_details::dsl::member_details)
            .values(&data)
            .returning(member_details::dsl::id)
            .get_result(conn)?;

        let data = Member {
            id: 0,
            member_address_details_id: mad_id,
            member_details_id: md_id,
            musical_instrument_id: None,
            picture_asset_id: None,
            allow_privacy_info_sharing: false,
            activated: false,
            activation_string: activation_string.to_string(),
            activation_time: chrono::Utc::now().add(TimeDelta::minutes(30)).naive_utc(),
            creation_time: chrono::Utc::now().naive_utc(),
            nonce: generate_encoded_nonce(),
        };

        let member_id: i32 = diesel::insert_into(members::dsl::members)
            .values(&data)
            .returning(members::dsl::id)
            .get_result(conn)?;

        diesel::insert_into(member_role_associations::dsl::member_role_associations)
            .values((
                member_role_associations::dsl::member_id.eq(member_id),
                member_role_associations::dsl::system_role.eq(Role::Operator),
            ))
            .execute(conn)?;
        Ok(())
    })
    .map_err(|e| e.into())
}

pub fn get_member_by_activation_string(
    conn: &mut DbConnection,
    activation_string: &str,
) -> Result<Member, Error> {
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

pub fn get_member_by_email_address(
    conn: &mut DbConnection,
    email_address: &str,
) -> Result<Member, Error> {
    let member_details = get_member_details_by_email_address(conn, email_address)?;

    let activated_filter = members::activated.eq(true);
    let details_filter = members::member_details_id.eq(member_details.id);

    Ok(members::table
        .select(members::all_columns)
        .filter(activated_filter.and(details_filter))
        .first::<Member>(conn)?)
}

pub fn get_member_details_by_id(
    conn: &mut DbConnection,
    member_details_id: &i32,
) -> Result<MemberDetails, Error> {
    let id_filter = member_details::id.eq(member_details_id);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(id_filter)
        .first::<MemberDetails>(conn)?)
}

pub fn get_member_details_by_email_address(
    conn: &mut DbConnection,
    email_address: &str,
) -> Result<MemberDetails, Error> {
    let email_address_filter = member_details::email_address.eq(email_address);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(email_address_filter)
        .first::<MemberDetails>(conn)?)
}

pub fn delete_member_address_details_by_id(
    conn: &mut DbConnection,
    member_address_details_id: i32,
) -> Result<(), Error> {
    let details = member_details::dsl::member_details
        .select(member_details::all_columns)
        .filter(member_details::id.eq(member_address_details_id))
        .load::<MemberDetails>(conn)?;
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

pub fn delete_member_details_by_id(
    conn: &mut DbConnection,
    member_details_id: i32,
) -> Result<(), Error> {
    let address_details = member_address_details::dsl::member_address_details
        .select(member_address_details::all_columns)
        .filter(member_address_details::id.eq(member_details_id))
        .load::<MemberAddressDetails>(conn)?;
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

pub fn activate(conn: &mut DbConnection, member: &Member) -> Result<(), Error> {
    diesel::update(members::dsl::members)
        .filter(members::id.eq(member.id))
        .set(members::dsl::activated.eq(true))
        .execute(conn)?;
    Ok(())
}
