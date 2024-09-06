use crate::model::members::MemberAddressDetails;
use crate::model::members::{Member, MemberDetails};
use crate::model::setup::FirstOperator;
use crate::schema::member_address_details;
use crate::schema::member_details;
use crate::schema::member_role_associations;
use crate::schema::members;
use crate::security::Role;
use crate::{dal, DbPool};
use chrono::TimeDelta;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use std::ops::Add;

pub fn has_operators(pool: &DbPool) -> Result<bool, String> {
    let mut conn = dal::get_connection(pool)?;

    let count: i64 = member_role_associations::dsl::member_role_associations
        .filter(member_role_associations::dsl::system_role.eq(Role::Operator))
        .count()
        .get_result(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(count != 0)
}

pub fn create_first_operator(
    pool: &DbPool,
    operator: &FirstOperator,
    activation_string: &str,
) -> Result<(), String> {
    let mut conn = dal::get_connection(pool)?;
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
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
        };

        let member_id: i32 = diesel::insert_into(members::dsl::members)
            .values(&data)
            .returning(members::dsl::id)
            .get_result(conn)?;

        diesel::insert_into(member_role_associations::dsl::member_role_associations)
            .values([(
                member_role_associations::dsl::member_id.eq(member_id),
                member_role_associations::dsl::system_role.eq(Role::Operator),
            )])
            .execute(conn)?;
        Ok(())
    })
    .map_err(|e| format!("Error running transaction: {e}"))
}

pub fn delete_member_address_details_by_id(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    member_address_details_id: i32,
) -> Result<(), diesel::result::Error> {
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
        Some(first_error) => Err(first_error),
        None => Ok(()),
    }
}

pub fn delete_member_details_by_id(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    member_details_id: i32,
) -> Result<(), diesel::result::Error> {
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
        Some(first_error) => Err(first_error),
        None => Ok(()),
    }
}
