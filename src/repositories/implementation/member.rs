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
use crate::injection::Injectable;
use crate::model::prelude::Role;
use crate::model::storage::entities::{Member, MemberDetail};
use crate::model::storage::extended_entities::ExtendedMember;
use crate::model::storage::prelude::MemberAddressDetail;
use crate::repositories::traits::MemberRepository;
use crate::schema::{member_address_details, member_details, member_role_associations, members};
use actix_web::web::Data;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

pub struct Implementation;

impl MemberRepository for Implementation {
    fn create_inactive(
        &self,
        conn: &mut DbConnection,
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
        conn: &mut DbConnection,
        id: i32,
    ) -> BackendResult<ExtendedMember> {
        let data: (Member, MemberDetail, MemberAddressDetail) = members::table
            .inner_join(member_details::table)
            .inner_join(member_address_details::table)
            .filter(members::id.eq(id))
            .select((
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ))
            .first(conn)?;
        Ok(ExtendedMember::from((&data.0, &data.1, &data.2)))
    }

    fn count_members_with_role(&self, conn: &mut DbConnection, role: Role) -> BackendResult<usize> {
        let filter = member_role_associations::system_role.eq(role);
        let count: i64 = member_role_associations::table
            .filter(filter)
            .count()
            .get_result(conn)?;

        Ok(count as usize)
    }
}

impl Injectable<(), dyn MemberRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn MemberRepository> {
        let arc: Arc<dyn MemberRepository> = Arc::new(Self);
        Data::from(arc)
    }
}
