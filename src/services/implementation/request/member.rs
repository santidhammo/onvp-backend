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

use crate::generic::result::{BackendError, BackendResult};
use crate::generic::{search_helpers, Injectable};
use crate::model::interface::search::{SearchParams, SearchResult};
use crate::model::storage::entities::{Member, MemberAddressDetail, MemberDetail};
use crate::model::storage::extended_entities::ExtendedMember;
use crate::repositories::definitions::MemberRepository;

use crate::generic::lazy::SEARCH_PAGE_SIZE;
use crate::generic::search_helpers::create_like_string;
use crate::generic::storage::database::{DatabaseConnection, DatabaseConnectionPool};
use crate::model::interface::responses::MemberResponse;
use crate::schema::{member_address_details, member_details, members};
use crate::services::definitions::request::{MemberRequestService, SearchController};
use actix_web::web::Data;
use diesel::{
    BoolExpressionMethods, Connection, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    SqliteConnection,
};
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    page_size: usize,
    member_repository: Data<dyn MemberRepository>,
}

impl MemberRequestService for Implementation {
    fn find_by_id(&self, member_id: i32) -> BackendResult<MemberResponse> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_id(&mut conn, member_id)?;
        Ok(MemberResponse::from(&extended_member))
    }

    fn find_by_activation_string(&self, activation_string: &str) -> BackendResult<MemberResponse> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_activation_string(&mut conn, activation_string)?;
        Ok(MemberResponse::from(&extended_member))
    }

    fn find_by_email_address(&self, email_address: &str) -> BackendResult<MemberResponse> {
        let mut conn = self.pool.get()?;
        let extended_member = self
            .member_repository
            .find_extended_by_email_address(&mut conn, email_address)?;
        Ok(MemberResponse::from(&extended_member))
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
    ) -> Result<(usize, Vec<(Member, MemberDetail, MemberAddressDetail)>), BackendError> {
        use diesel::PgTextExpressionMethods;
        let filter = member_details::first_name
            .ilike(&like_search_string)
            .or(member_details::last_name.ilike(&like_search_string))
            .or(member_details::email_address.ilike(&like_search_string));

        let total_count: usize = member_details::dsl::member_details
            .filter(filter)
            .count()
            .get_result::<i64>(conn)? as usize;

        let member_details: Vec<(Member, MemberDetail, MemberAddressDetail)> = members::table
            .inner_join(member_details::table)
            .inner_join(member_address_details::table)
            .filter(filter)
            .order_by(member_details::last_name)
            .order_by(member_details::first_name)
            .limit(self.page_size as i64)
            .offset((params.page_offset * self.page_size) as i64)
            .select((
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ))
            .load(conn)?;

        Ok((total_count, member_details))
    }

    fn sqlite_search(
        &self,
        params: &SearchParams,
        like_search_string: &str,
        conn: &mut SqliteConnection,
    ) -> Result<(usize, Vec<(Member, MemberDetail, MemberAddressDetail)>), BackendError> {
        use diesel::TextExpressionMethods;
        let filter = member_details::first_name
            .like(&like_search_string)
            .or(member_details::last_name.like(&like_search_string))
            .or(member_details::email_address.like(&like_search_string));

        let total_count: usize = member_details::dsl::member_details
            .filter(filter)
            .count()
            .get_result::<i64>(conn)? as usize;

        let member_details: Vec<(Member, MemberDetail, MemberAddressDetail)> = members::table
            .inner_join(member_details::table)
            .inner_join(member_address_details::table)
            .filter(filter)
            .order_by(member_details::last_name)
            .limit(self.page_size as i64)
            .offset(params.page_offset as i64)
            .select((
                Member::as_select(),
                MemberDetail::as_select(),
                MemberAddressDetail::as_select(),
            ))
            .load(conn)?;

        Ok((total_count, member_details))
    }

    fn search_internal(
        &self,
        params: &SearchParams,
        connection: &mut DatabaseConnection,
        like_search_string: &str,
    ) -> BackendResult<SearchResult<MemberResponse>> {
        connection.transaction::<SearchResult<MemberResponse>, BackendError, _>(|connection| {
            // ILIKE is only supported on PostgreSQL
            let (total_count, member_details) = match connection {
                DatabaseConnection::PostgreSQL(ref mut conn) => {
                    self.postgresql_search(params, like_search_string, conn)?
                }

                DatabaseConnection::SQLite(ref mut conn) => {
                    self.sqlite_search(params, like_search_string, conn)?
                }
            };
            Ok(SearchResult {
                total_count,
                page_offset: params.page_offset,
                page_count: search_helpers::calculate_page_count(self.page_size, total_count),
                rows: member_details
                    .iter()
                    .map(|(member, member_detail, member_address_detail)| {
                        ExtendedMember::from((member, member_detail, member_address_detail))
                    })
                    .map(|extended_member| MemberResponse::from(&extended_member))
                    .collect(),
            })
        })
    }
}

impl Injectable<(&DatabaseConnectionPool, &Data<dyn MemberRepository>), dyn MemberRequestService>
    for Implementation
{
    fn injectable(
        (pool, member_repository): (&DatabaseConnectionPool, &Data<dyn MemberRepository>),
    ) -> Data<dyn MemberRequestService> {
        let implementation = Self {
            pool: pool.clone(),
            page_size: *SEARCH_PAGE_SIZE,
            member_repository: member_repository.clone(),
        };

        let arc: Arc<dyn MemberRequestService> = Arc::new(implementation);
        Data::from(arc)
    }
}
