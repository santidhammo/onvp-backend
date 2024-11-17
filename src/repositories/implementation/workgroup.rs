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
use crate::model::storage::entities::{Member, MemberAddressDetail, MemberDetail, Workgroup};
use crate::model::storage::extended_entities::ExtendedMember;
use crate::repositories::definitions::WorkgroupRepository;
use crate::schema::member_address_details;
use crate::schema::member_details;
use crate::schema::members;
use crate::schema::workgroup_member_relationships;
use crate::schema::workgroup_role_associations;
use crate::schema::workgroups;
use actix_web::web::Data;
use diesel::dsl::{exists, not};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use std::sync::Arc;

pub struct Implementation {
    page_size: usize,
}

impl WorkgroupRepository for Implementation {
    fn register(&self, conn: &mut DatabaseConnection, workgroup: Workgroup) -> BackendResult<i32> {
        let result: usize = diesel::insert_into(workgroups::table)
            .values(workgroup)
            .returning(workgroups::id)
            .execute(conn)?;
        Ok(result as i32)
    }

    fn find_by_id(&self, conn: &mut DatabaseConnection, id: i32) -> BackendResult<Workgroup> {
        let workgroup: Workgroup = QueryDsl::select(
            workgroups::table.filter(workgroups::id.eq(id)),
            Workgroup::as_select(),
        )
        .first(conn)?;
        Ok(workgroup)
    }

    fn save(&self, conn: &mut DatabaseConnection, workgroup: Workgroup) -> BackendResult<()> {
        diesel::update(workgroups::table)
            .filter(workgroups::id.eq(workgroup.id))
            .set(workgroup)
            .execute(conn)?;
        Ok(())
    }

    fn search(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<Workgroup>)> {
        let like_search_string = search_helpers::create_like_string(term);
        let (total_count, workgroups) = conn
            .transaction::<(usize, Vec<Workgroup>), BackendError, _>(|conn| {
                self.search_workgroups(conn, page_offset, &like_search_string)
            })?;
        Ok((total_count, self.page_size, workgroups))
    }

    fn unregister(&self, conn: &mut DatabaseConnection, workgroup_id: i32) -> BackendResult<()> {
        conn.transaction::<_, BackendError, _>(|conn| {
            diesel::delete(workgroup_role_associations::table)
                .filter(workgroup_role_associations::workgroup_id.eq(workgroup_id))
                .execute(conn)?;

            let deleted_rows = diesel::delete(workgroups::table)
                .filter(workgroups::id.eq(workgroup_id))
                .execute(conn)?;

            if deleted_rows == 0 {
                Err(BackendError::not_enough_records())
            } else {
                Ok(())
            }
        })
    }

    fn find_members_by_id(
        &self,
        conn: &mut DatabaseConnection,
        workgroup_id: i32,
    ) -> BackendResult<Vec<ExtendedMember>> {
        let result: Vec<(Member, MemberDetail, MemberAddressDetail)> = QueryDsl::select(
            workgroup_member_relationships::table
                .inner_join(
                    members::table
                        .inner_join(member_details::table)
                        .inner_join(member_address_details::table),
                )
                .filter(workgroup_member_relationships::workgroup_id.eq(workgroup_id)),
            (
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ),
        )
        .load(conn)?;
        Ok(result
            .iter()
            .map(|(member, member_detail, member_address_detail)| {
                ExtendedMember::from((member, member_detail, member_address_detail))
            })
            .collect())
    }

    fn associate_member_to_workgroup(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
        workgroup_id: i32,
    ) -> BackendResult<()> {
        diesel::insert_into(workgroup_member_relationships::table)
            .values((
                workgroup_member_relationships::member_id.eq(member_id),
                workgroup_member_relationships::workgroup_id.eq(workgroup_id),
            ))
            .execute(conn)?;
        Ok(())
    }

    fn dissociate_member_from_workgroup(
        &self,
        conn: &mut DatabaseConnection,
        member_id: i32,
        workgroup_id: i32,
    ) -> BackendResult<()> {
        diesel::delete(workgroup_member_relationships::table)
            .filter(
                workgroup_member_relationships::member_id
                    .eq(member_id)
                    .and(workgroup_member_relationships::workgroup_id.eq(workgroup_id)),
            )
            .execute(conn)?;
        Ok(())
    }

    fn available_members_search(
        &self,
        conn: &mut DatabaseConnection,
        workgroup_id: i32,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<ExtendedMember>)> {
        let like_search_string = search_helpers::create_like_string(term);
        let (total_count, extended_members) = conn
            .transaction::<(usize, Vec<ExtendedMember>), BackendError, _>(|conn| {
                self.search_available_members(conn, page_offset, &like_search_string, workgroup_id)
            })?;
        Ok((total_count, self.page_size, extended_members))
    }
}

impl Implementation {
    fn search_workgroups(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
    ) -> Result<(usize, Vec<Workgroup>), BackendError> {
        let search_expression = workgroups::name.ilike(term);

        let total_count: usize = workgroups::table
            .filter(&search_expression)
            .count()
            .get_result::<i64>(conn)? as usize;

        let workgroups: Vec<(Workgroup,)> = QueryDsl::select(
            QueryDsl::limit(
                workgroups::table
                    .filter(&search_expression)
                    .order_by(workgroups::name),
                self.page_size as i64,
            )
            .offset((page_offset * self.page_size) as i64),
            (Workgroup::as_select(),),
        )
        .load(conn)?;

        Ok((
            total_count,
            workgroups
                .iter()
                .map(|(workgroup,)| workgroup.clone())
                .collect(),
        ))
    }

    fn search_available_members(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
        workgroup_id: i32,
    ) -> Result<(usize, Vec<ExtendedMember>), BackendError> {
        let sub_table = workgroup_member_relationships::table
            .select(workgroup_member_relationships::member_id)
            .filter(
                workgroup_member_relationships::workgroup_id
                    .eq(workgroup_id)
                    .and(workgroup_member_relationships::member_id.eq(members::id)),
            );

        let search_expression = member_details::first_name
            .ilike(term)
            .or(member_details::last_name.ilike(term))
            .or(member_details::email_address.ilike(term));

        let where_expression = members::activated
            .eq(true)
            .and(search_expression)
            .and(not(exists(sub_table)));

        let total_count: usize = members::table
            .inner_join(member_details::table)
            .filter(&where_expression)
            .count()
            .get_result::<i64>(conn)? as usize;

        let result: Vec<(Member, MemberDetail)> = QueryDsl::select(
            QueryDsl::limit(
                members::table
                    .inner_join(member_details::table)
                    .filter(&where_expression)
                    .order_by(member_details::last_name)
                    .order_by(member_details::first_name),
                self.page_size as i64,
            )
            .offset((page_offset * self.page_size) as i64),
            (Member::as_select(), MemberDetail::as_select()),
        )
        .load(conn)?;

        Ok((
            total_count,
            result
                .iter()
                .map(|(member, member_detail)| ExtendedMember::from((member, member_detail)))
                .collect(),
        ))
    }
}

impl Injectable<(), dyn WorkgroupRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn WorkgroupRepository> {
        let arc: Arc<dyn WorkgroupRepository> = Arc::new(Self {
            page_size: *SEARCH_PAGE_SIZE,
        });
        Data::from(arc)
    }
}
