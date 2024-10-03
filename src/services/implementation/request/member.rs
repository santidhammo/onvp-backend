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

use crate::dal;
use crate::dal::members::with_detail_response_by_id;
use crate::dal::{DbConnection, DbPool};
use crate::generic::result::{BackendError, BackendResult};
use crate::injection::Injectable;
use crate::model::database::entities::{Member, MemberDetail};
use crate::model::interface::prelude::{MemberResponse, SearchResult};
use crate::model::interface::search::{SearchParams, SEARCH_PAGE_SIZE};
use crate::schema::{member_details, members};
use crate::services::traits::request::{MemberRequestService, SearchController};
use actix_web::web::Data;
use dal::create_like_string;
use diesel::{
    BoolExpressionMethods, Connection, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    SqliteConnection,
};
use std::sync::Arc;

pub struct Implementation {
    pool: DbPool,
    page_size: usize,
}

impl MemberRequestService for Implementation {
    fn find(&self, member_id: &i32) -> BackendResult<MemberResponse> {
        let mut connection = self.pool.get()?;
        Ok(MemberResponse::from(&with_detail_response_by_id(
            &mut connection,
            *member_id,
        )?))
    }
}

impl SearchController<MemberResponse> for Implementation {
    fn search(&self, params: &SearchParams) -> BackendResult<SearchResult<MemberResponse>> {
        let like_search_string = create_like_string(params.term.clone().unwrap_or_default());
        let mut connection = self.pool.get()?;
        self.search_internal(params, &mut connection, &like_search_string)
    }
}

impl Implementation {
    fn postgresql_search(
        &self,
        params: &SearchParams,
        like_search_string: &str,
        conn: &mut PgConnection,
    ) -> Result<(usize, Vec<(Member, MemberDetail)>), BackendError> {
        use diesel::PgTextExpressionMethods;
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
            .limit(self.page_size as i64)
            .offset((params.page_offset * self.page_size) as i64)
            .select((Member::as_select(), MemberDetail::as_select()))
            .load(conn)?;

        Ok((total_count, member_details))
    }

    fn sqlite_search(
        &self,
        params: &SearchParams,
        like_search_string: &str,
        conn: &mut SqliteConnection,
    ) -> Result<(usize, Vec<(Member, MemberDetail)>), BackendError> {
        use diesel::TextExpressionMethods;
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
            .limit(self.page_size as i64)
            .offset(params.page_offset as i64)
            .select((Member::as_select(), MemberDetail::as_select()))
            .load(conn)?;

        Ok((total_count, member_details))
    }

    fn search_internal(
        &self,
        params: &SearchParams,
        connection: &mut DbConnection,
        like_search_string: &str,
    ) -> BackendResult<SearchResult<MemberResponse>> {
        connection.transaction::<SearchResult<MemberResponse>, BackendError, _>(|connection| {
            // ILIKE is only supported on PostgreSQL
            let (total_count, member_details) = match connection {
                DbConnection::PostgreSQL(ref mut conn) => {
                    self.postgresql_search(params, like_search_string, conn)?
                }

                DbConnection::SQLite(ref mut conn) => {
                    self.sqlite_search(params, like_search_string, conn)?
                }
            };
            Ok(SearchResult {
                total_count,
                page_offset: params.page_offset,
                page_count: dal::calculate_page_count(self.page_size, total_count),
                rows: member_details.iter().map(MemberResponse::from).collect(),
            })
        })
    }
}

impl Injectable<&DbPool, dyn MemberRequestService> for Implementation {
    fn injectable(pool: &DbPool) -> Data<dyn MemberRequestService> {
        let implementation = Self {
            pool: pool.clone(),
            page_size: *SEARCH_PAGE_SIZE,
        };

        let member_command_controller_arc: Arc<dyn MemberRequestService> = Arc::new(implementation);
        Data::from(member_command_controller_arc)
    }
}
