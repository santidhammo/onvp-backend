use crate::dal::DbConnection;
use crate::model::generic::SearchResult;
use crate::model::members::{Member, MemberAddressDetail, MemberDetail, MemberWithDetail};
use crate::model::security::Role;
use crate::model::setup::FirstOperator;
use crate::schema::*;
use crate::security::FIRST_OPERATOR_ACTIVATION_MINUTES;
use crate::{dal, Error, Result};
use chrono::TimeDelta;
use diesel::prelude::*;
use diesel::update;
use std::collections::HashSet;

pub fn has_operators(conn: &mut DbConnection) -> Result<bool> {
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
) -> Result<()> {
    conn.transaction::<_, Error, _>(|conn| {
        let member_address_detail = MemberAddressDetail {
            id: 0,
            street: operator.street.clone(),
            house_number: operator.house_number.clone(),
            house_number_postfix: operator.house_number_postfix.clone(),
            postal_code: operator.postal_code.clone(),
            domicile: operator.domicile.clone(),
        };

        let member_detail = MemberDetail {
            id: 0,
            first_name: operator.first_name.clone(),
            last_name: operator.last_name.clone(),
            email_address: operator.email_address.clone(),
            phone_number: operator.phone_number.clone(),
        };

        create_inactive_member(
            conn,
            member_address_detail,
            member_detail,
            activation_string,
            *FIRST_OPERATOR_ACTIVATION_MINUTES,
            Role::Operator,
        )
    })
    .map_err(|e| e.into())
}

pub fn create_inactive_member(
    conn: &mut DbConnection,
    member_address_detail: MemberAddressDetail,
    member_detail: MemberDetail,
    activation_string: &str,
    activation_delta: TimeDelta,
    role: Role,
) -> Result<()> {
    conn.transaction::<_, Error, _>(|conn| {
        let mad_id: i32 = diesel::insert_into(member_address_details::dsl::member_address_details)
            .values(&member_address_detail)
            .returning(member_address_details::dsl::id)
            .get_result(conn)?;

        let md_id: i32 = diesel::insert_into(member_details::dsl::member_details)
            .values(&member_detail)
            .returning(member_details::dsl::id)
            .get_result(conn)?;

        let data =
            internal::create_member_record(activation_string, mad_id, md_id, activation_delta);

        let member_id: i32 = diesel::insert_into(members::dsl::members)
            .values(&data)
            .returning(members::dsl::id)
            .get_result(conn)?;

        diesel::insert_into(member_role_associations::dsl::member_role_associations)
            .values((
                member_role_associations::dsl::member_id.eq(member_id),
                member_role_associations::dsl::system_role.eq(role),
            ))
            .execute(conn)?;
        Ok(())
    })
}

pub fn get_member_by_activation_string(
    conn: &mut DbConnection,
    activation_string: &str,
) -> Result<Member> {
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

pub fn get_member_by_email_address(conn: &mut DbConnection, email_address: &str) -> Result<Member> {
    let member_details = get_member_detail_by_email_address(conn, email_address)?;

    let activated_filter = members::activated.eq(true);
    let details_filter = members::member_details_id.eq(member_details.id);

    Ok(members::table
        .select(members::all_columns)
        .filter(activated_filter.and(details_filter))
        .first::<Member>(conn)?)
}

pub fn get_member_detail_by_id(
    conn: &mut DbConnection,
    member_details_id: &i32,
) -> Result<MemberDetail> {
    let id_filter = member_details::id.eq(member_details_id);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(id_filter)
        .first::<MemberDetail>(conn)?)
}

pub fn get_member_detail_by_email_address(
    conn: &mut DbConnection,
    email_address: &str,
) -> Result<MemberDetail> {
    let email_address_filter = member_details::email_address.eq(email_address);

    Ok(member_details::table
        .select(member_details::all_columns)
        .filter(email_address_filter)
        .first::<MemberDetail>(conn)?)
}

pub fn delete_member_address_detail_by_id(
    conn: &mut DbConnection,
    member_address_details_id: i32,
) -> Result<()> {
    let details = member_details::dsl::member_details
        .select(member_details::all_columns)
        .filter(member_details::id.eq(member_address_details_id))
        .load::<MemberDetail>(conn)?;
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
    let address_details = member_address_details::dsl::member_address_details
        .select(member_address_details::all_columns)
        .filter(member_address_details::id.eq(member_details_id))
        .load::<MemberAddressDetail>(conn)?;
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

pub fn activate(conn: &mut DbConnection, member: &Member) -> Result<()> {
    update(members::dsl::members)
        .filter(members::id.eq(member.id))
        .set(members::dsl::activated.eq(true))
        .execute(conn)?;
    Ok(())
}

pub fn get_member_with_detail_by_id(conn: &mut DbConnection, id: i32) -> Result<MemberWithDetail> {
    let filter = members::id.eq(id);
    let result: (Member, MemberDetail) = members::table
        .inner_join(member_details::table)
        .filter(filter)
        .select((Member::as_select(), MemberDetail::as_select()))
        .first(conn)?;
    Ok(MemberWithDetail::from(&result))
}

pub fn find_members_with_details_by_search_string(
    conn: &mut DbConnection,
    search_string: &String,
    page_size: usize,
    page_offset: usize,
) -> Result<SearchResult<MemberWithDetail>> {
    let like_search_string = dal::create_like_string(search_string);

    conn.transaction::<SearchResult<MemberWithDetail>, Error, _>(|conn| {
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

                let member_details: Vec<(Member, MemberDetail)> = members::table
                    .inner_join(member_details::table)
                    .filter(filter)
                    .order_by(member_details::last_name)
                    .order_by(member_details::first_name)
                    .limit(page_size as i64)
                    .offset((page_offset * page_size) as i64)
                    .select((Member::as_select(), MemberDetail::as_select()))
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

                let member_details: Vec<(Member, MemberDetail)> = members::table
                    .inner_join(member_details::table)
                    .filter(filter)
                    .order_by(member_details::last_name)
                    .limit(page_size as i64)
                    .offset(page_offset as i64)
                    .select((Member::as_select(), MemberDetail::as_select()))
                    .load(conn)?;

                (total_count, member_details)
            }
        };
        Ok(SearchResult {
            total_count,
            page_offset,
            page_count: dal::calculate_page_count(page_size, total_count),
            rows: member_details.iter().map(MemberWithDetail::from).collect(),
        })
    })
}

mod internal {
    use crate::model::members::Member;
    use crate::security::generate_encoded_nonce;
    use chrono::TimeDelta;
    use std::ops::Add;
    pub(super) fn create_member_record(
        activation_string: &str,
        mad_id: i32,
        md_id: i32,
        activation_delta: TimeDelta,
    ) -> Member {
        let data = Member {
            id: 0,
            member_address_details_id: mad_id,
            member_details_id: md_id,
            musical_instrument_id: None,
            picture_asset_id: None,
            allow_privacy_info_sharing: false,
            activated: false,
            activation_string: activation_string.to_string(),
            activation_time: chrono::Utc::now().add(activation_delta).naive_utc(),
            creation_time: chrono::Utc::now().naive_utc(),
            nonce: generate_encoded_nonce(),
        };
        data
    }
}

pub(crate) fn update_member_with_detail(
    conn: &mut DbConnection,
    member_with_detail: &MemberWithDetail,
) -> Result<()> {
    conn.transaction::<_, Error, _>(|conn| {
        let filter = members::id.eq(member_with_detail.id);

        update(members::table)
            .filter(filter)
            .set((
                members::musical_instrument_id.eq(member_with_detail.musical_instrument_id.clone()),
                members::picture_asset_id.eq(member_with_detail.picture_asset_id.clone()),
            ))
            .execute(conn)?;

        let result: Member = members::table
            .inner_join(member_details::table)
            .filter(filter)
            .select(Member::as_select())
            .first(conn)?;

        update(member_details::table)
            .filter(member_details::id.eq(result.member_details_id))
            .set((
                member_details::first_name.eq(member_with_detail.first_name.clone()),
                member_details::last_name.eq(member_with_detail.last_name.clone()),
                member_details::phone_number.eq(member_with_detail.phone_number.clone()),
                member_details::email_address.eq(member_with_detail.email_address.clone()),
            ))
            .execute(conn)?;

        Ok(())
    })
}
