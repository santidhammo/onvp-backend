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
use crate::model::storage::extended_entities::FacebookMember;
use crate::repositories::definitions::FacebookRepository;
use crate::schema::{member_details, members};
use actix_web::web::Data;

use crate::model::storage::entities::{Member, MemberDetail};
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use std::sync::Arc;

pub struct Implementation {
    page_size: usize,
}

impl FacebookRepository for Implementation {
    fn search(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
    ) -> BackendResult<(usize, usize, Vec<FacebookMember>)> {
        let like_search_string = search_helpers::create_like_string(term);
        let (total_count, facebook_members) = conn
            .transaction::<(usize, Vec<FacebookMember>), BackendError, _>(|conn| {
                self.search(conn, page_offset, &like_search_string)
            })?;
        Ok((total_count, self.page_size, facebook_members))
    }
}

impl Implementation {
    fn search(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
    ) -> Result<(usize, Vec<FacebookMember>), BackendError> {
        let where_expression = members::activated
            .eq(true)
            .and(members::allow_privacy_info_sharing.eq(true))
            .and(
                member_details::first_name
                    .ilike(term)
                    .or(member_details::last_name.ilike(term)),
            );

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
                .map(|(member, member_detail)| FacebookMember::from((member, member_detail)))
                .collect(),
        ))
    }
}

impl Injectable<(), dyn FacebookRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn FacebookRepository> {
        let arc: Arc<dyn FacebookRepository> = Arc::new(Self {
            page_size: *SEARCH_PAGE_SIZE,
        });
        Data::from(arc)
    }
}
