use crate::model::members::{Member, MemberDetails};
use crate::model::FirstOperator;
use crate::schema::member_address_details;
use crate::schema::member_details;
use crate::schema::member_role_associations;
use crate::schema::members;
use crate::security::Role;
use crate::DbPool;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

pub fn has_operators(pool: &DbPool) -> Result<bool, String> {
    let mut conn = get_connection(pool)?;

    let count: i64 = member_role_associations::dsl::member_role_associations
        .filter(member_role_associations::dsl::system_role.eq(Role::Operator))
        .count()
        .get_result(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(count != 0)
}

pub fn create_first_operator(pool: &DbPool, operator: &FirstOperator) -> Result<(), String> {
    use crate::model::members::MemberAddressDetails;
    let mut conn = get_connection(pool)?;
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let data = MemberAddressDetails {
            id: None,
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
            id: None,
            user_name: operator.user_name.clone(),
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
            id: None,
            member_address_details_id: mad_id,
            member_details_id: md_id,
            musical_instrument_id: None,
            picture_asset_id: None,
            allow_privacy_info_sharing: false,
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

fn get_connection(
    pool: &DbPool,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, String> {
    Ok(pool.get().map_err(|e| e.to_string())?)
}
