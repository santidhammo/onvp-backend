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
use crate::generic::lazy::SEARCH_PAGE_SIZE;
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::storage::database::DatabaseConnection;
use crate::generic::{search_helpers, Injectable};
use crate::model::primitives::Role;
use crate::model::storage::entities::{Member, MemberAddressDetail, MemberDetail, Workgroup};
use crate::model::storage::extended_entities::ExtendedMember;
use crate::repositories::definitions::MemberRepository;
use crate::schema::{
    member_address_details, member_details, member_role_associations, members,
    workgroup_member_relationships, workgroups,
};
use actix_web::web::Data;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper, SqliteConnection,
};
use std::sync::Arc;

pub struct Implementation {
    page_size: usize,
}

impl MemberRepository for Implementation {
    fn create_inactive(
        &self,
        conn: &mut DatabaseConnection,
        extended_member: &ExtendedMember,
    ) -> BackendResult<i32> {
        conn.transaction::<i32, BackendError, _>(|conn| {
            let member_detail_id: i32 = diesel::insert_into(member_details::table)
                .values(&extended_member.member_detail)
                .returning(member_details::id)
                .get_result(conn)?;

            let member_address_detail_id: i32 = diesel::insert_into(member_address_details::table)
                .values(&extended_member.member_address_detail)
                .returning(member_address_details::id)
                .get_result(conn)?;

            let mut member = Member::from(extended_member);
            member.member_details_id = member_detail_id;
            member.member_address_details_id = member_address_detail_id;
            let member_id = diesel::insert_into(members::table)
                .values(member)
                .returning(members::id)
                .get_result(conn)?;

            Ok(member_id)
        })
    }

    fn find_extended_by_id(
        &self,
        conn: &mut DatabaseConnection,
        id: i32,
    ) -> BackendResult<ExtendedMember> {
        let (member, member_detail, member_address_detail): (
            Member,
            MemberDetail,
            MemberAddressDetail,
        ) = members::table
            .inner_join(member_details::table)
            .inner_join(member_address_details::table)
            .filter(members::id.eq(id))
            .select((
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ))
            .first(conn)?;
        Ok(ExtendedMember::from((
            &member,
            &member_detail,
            &member_address_detail,
        )))
    }

    fn find_extended_by_activation_string(
        &self,
        conn: &mut DatabaseConnection,
        activation_string: &str,
    ) -> BackendResult<ExtendedMember> {
        let activated_filter = members::activated.eq(false);
        let activation_time_filter = members::activation_time.gt(chrono::Utc::now().naive_utc());
        let activation_string_filter = members::activation_string.eq(activation_string);

        let (member, member_detail, member_address_detail): (
            Member,
            MemberDetail,
            MemberAddressDetail,
        ) = members::table
            .inner_join(member_details::table)
            .inner_join(member_address_details::table)
            .select((
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ))
            .filter(
                activated_filter
                    .and(activation_time_filter)
                    .and(activation_string_filter),
            )
            .first(conn)?;
        Ok(ExtendedMember::from((
            &member,
            &member_detail,
            &member_address_detail,
        )))
    }

    fn find_extended_by_email_address(
        &self,
        conn: &mut DatabaseConnection,
        email_address: &str,
    ) -> BackendResult<ExtendedMember> {
        let activated_filter = members::activated.eq(true);
        let email_address_filter = member_details::email_address.eq(email_address);
        let filter = activated_filter.and(email_address_filter);

        let (member, member_detail, member_address_detail): (
            Member,
            MemberDetail,
            MemberAddressDetail,
        ) = members::table
            .inner_join(member_details::table)
            .inner_join(member_address_details::table)
            .select((
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ))
            .filter(filter)
            .first(conn)?;
        Ok(ExtendedMember::from((
            &member,
            &member_detail,
            &member_address_detail,
        )))
    }

    fn find_workgroups(
        &self,
        conn: &mut DatabaseConnection,
        id: i32,
    ) -> BackendResult<Vec<Workgroup>> {
        let result: Vec<Workgroup> = workgroup_member_relationships::table
            .inner_join(workgroups::table)
            .filter(workgroup_member_relationships::member_id.eq(id))
            .select(Workgroup::as_select())
            .load(conn)?;
        Ok(result)
    }

    fn save(&self, conn: &mut DatabaseConnection, member: ExtendedMember) -> BackendResult<()> {
        conn.transaction::<_, BackendError, _>(|conn| {
            let filter = members::id.eq(member.id);

            diesel::update(members::table)
                .filter(filter)
                .set((
                    members::musical_instrument_id.eq(member.musical_instrument_id.clone()),
                    members::description.eq(member.description.clone()),
                    members::allow_privacy_info_sharing.eq(member.allow_privacy_info_sharing),
                ))
                .execute(conn)?;

            let member_detail = member.member_detail;

            diesel::update(member_details::table)
                .filter(member_details::id.eq(member_detail.id))
                .set(member_detail)
                .execute(conn)?;

            let member_address_detail = member.member_address_detail;

            diesel::update(member_address_details::table)
                .filter(member_address_details::id.eq(member_address_detail.id))
                .set(member_address_detail)
                .execute(conn)?;

            Ok(())
        })
    }

    fn count_members_with_role(
        &self,
        conn: &mut DatabaseConnection,
        role: Role,
    ) -> BackendResult<usize> {
        let filter = member_role_associations::system_role.eq(role);
        let count: i64 = member_role_associations::table
            .filter(filter)
            .count()
            .get_result(conn)?;

        Ok(count as usize)
    }

    fn activate_by_id(&self, conn: &mut DatabaseConnection, member_id: i32) -> BackendResult<()> {
        let result_id: i32 = diesel::update(members::table)
            .filter(members::id.eq(member_id))
            .set(members::activated.eq(true))
            .returning(members::id)
            .get_result(conn)?;

        if result_id == member_id {
            Ok(())
        } else {
            Err(BackendError::bad())
        }
    }

    fn search(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<ExtendedMember>)> {
        let like_search_string = search_helpers::create_like_string(term);
        conn.transaction::<(usize, usize, Vec<ExtendedMember>), BackendError, _>(|conn| {
            // ILIKE is only supported on PostgreSQL
            let (total_count, extended_members) = match conn {
                DatabaseConnection::PostgreSQL(ref mut conn) => {
                    self.postgresql_search(conn, page_offset, &like_search_string)
                }

                DatabaseConnection::SQLite(ref mut conn) => {
                    self.sqlite_search(conn, page_offset, &like_search_string)
                }
            }?;
            Ok((total_count, self.page_size, extended_members))
        })
    }

    fn unregister(&self, conn: &mut DatabaseConnection, member_id: i32) -> BackendResult<()> {
        conn.transaction::<_, BackendError, _>(|conn| {
            let extended_member = self.find_extended_by_id(conn, member_id)?;
            let member_detail_id = extended_member.member_detail.id;
            let member_address_detail_id = extended_member.member_address_detail.id;

            diesel::delete(
                member_role_associations::table
                    .filter(member_role_associations::member_id.eq(member_id)),
            )
            .execute(conn)?;

            let deleted_rows = diesel::delete(members::table)
                .filter(members::id.eq(member_id))
                .execute(conn)?;

            diesel::delete(member_address_details::table)
                .filter(member_address_details::id.eq(member_address_detail_id))
                .execute(conn)?;
            diesel::delete(member_details::table)
                .filter(member_details::id.eq(member_detail_id))
                .execute(conn)?;

            if deleted_rows == 0 {
                Err(BackendError::not_enough_records())
            } else {
                Ok(())
            }
        })
    }
}

impl Implementation {
    fn postgresql_search(
        &self,
        conn: &mut PgConnection,
        page_offset: usize,
        like_search_string: &str,
    ) -> Result<(usize, Vec<ExtendedMember>), BackendError> {
        use diesel::PgTextExpressionMethods;
        let filter = member_details::first_name
            .ilike(&like_search_string)
            .or(member_details::last_name.ilike(&like_search_string))
            .or(member_details::email_address.ilike(&like_search_string));

        let total_count: usize = member_details::dsl::member_details
            .filter(filter)
            .count()
            .get_result::<i64>(conn)? as usize;

        let result: Vec<(Member, MemberDetail, MemberAddressDetail)> = members::table
            .inner_join(member_details::table)
            .inner_join(member_address_details::table)
            .filter(filter)
            .order_by(member_details::last_name)
            .order_by(member_details::first_name)
            .limit(self.page_size as i64)
            .offset((page_offset * self.page_size) as i64)
            .select((
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ))
            .load(conn)?;

        Ok((
            total_count,
            result
                .iter()
                .map(|(member, member_detail, member_address_details)| {
                    ExtendedMember::from((member, member_detail, member_address_details))
                })
                .collect(),
        ))
    }

    fn sqlite_search(
        &self,
        conn: &mut SqliteConnection,
        page_offset: usize,
        like_search_string: &str,
    ) -> Result<(usize, Vec<ExtendedMember>), BackendError> {
        use diesel::TextExpressionMethods;
        let filter = member_details::first_name
            .like(&like_search_string)
            .or(member_details::last_name.like(&like_search_string))
            .or(member_details::email_address.like(&like_search_string));

        let total_count: usize = member_details::dsl::member_details
            .filter(filter)
            .count()
            .get_result::<i64>(conn)? as usize;

        let result: Vec<(Member, MemberDetail, MemberAddressDetail)> = members::table
            .inner_join(member_details::table)
            .inner_join(member_address_details::table)
            .filter(filter)
            .order_by(member_details::last_name)
            .order_by(member_details::first_name)
            .limit(self.page_size as i64)
            .offset((page_offset * self.page_size) as i64)
            .select((
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ))
            .load(conn)?;

        Ok((
            total_count,
            result
                .iter()
                .map(|(member, member_detail, member_address_details)| {
                    ExtendedMember::from((member, member_detail, member_address_details))
                })
                .collect(),
        ))
    }
}

impl Injectable<(), dyn MemberRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn MemberRepository> {
        let arc: Arc<dyn MemberRepository> = Arc::new(Self {
            page_size: *SEARCH_PAGE_SIZE,
        });
        Data::from(arc)
    }
}
