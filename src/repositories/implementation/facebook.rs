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
use diesel::backend::{Backend, SqlDialect};
use diesel::connection::LoadConnection;
use diesel::deserialize::{FromSql, FromStaticSqlRow};
use diesel::dsl::{AsSelect, ILike, InnerJoinQuerySource, Like, Limit, Or};
use diesel::expression::is_aggregate::No;
use diesel::expression::ValidGrouping;
use diesel::internal::derives::as_expression::Bound;
use diesel::internal::derives::multiconnection::sql_dialect::select_statement_syntax::AnsiSqlSelectStatement;
use diesel::internal::derives::multiconnection::{
    DieselReserveSpecialization, LimitClause, LimitOffsetClause, NoLimitClause, NoOffsetClause,
    OffsetClause,
};
use diesel::query_builder::{QueryFragment, QueryId};
use diesel::query_dsl::limit_dsl::LimitDsl;
use diesel::query_dsl::select_dsl::SelectDsl;
use diesel::serialize::ToSql;
use diesel::sql_types::{BigInt, Bool, HasSqlType, Text};
use diesel::{
    AppearsOnTable, BoolExpressionMethods, Connection, Expression, ExpressionMethods, QueryDsl,
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
            .transaction::<(usize, Vec<FacebookMember>), BackendError, _>(|conn| match conn {
                DatabaseConnection::PostgreSQL(conn) => {
                    let filter = FacebookSearchExpressionGenerator::postgresql(&like_search_string);
                    self.search(conn, page_offset, &filter)
                }
                DatabaseConnection::SQLite(conn) => {
                    let filter = FacebookSearchExpressionGenerator::sqlite(&like_search_string);
                    self.search(conn, page_offset, &filter)
                }
            })?;
        Ok((total_count, self.page_size, facebook_members))
    }
}

impl Implementation {
    fn search<DB, TSearchExpression, TConnection>(
        &self,
        conn: &mut TConnection,
        page_offset: usize,
        search_expression: TSearchExpression,
    ) -> Result<(usize, Vec<FacebookMember>), BackendError>
    where
        DB: Backend<SelectStatementSyntax = AnsiSqlSelectStatement>
            + SqlDialect
            + DieselReserveSpecialization
            + HasSqlType<Bool>
            + 'static,
        TConnection: LoadConnection + Connection<Backend = DB> + Send,
        TSearchExpression: QueryFragment<DB>
            + Expression<SqlType = Bool>
            + ValidGrouping<(), IsAggregate = No>
            + AppearsOnTable<InnerJoinQuerySource<members::table, member_details::table>>
            + QueryId,
        Limit<members::table>: LimitDsl,
        Limit<member_details::table>: LimitDsl,
        members::table: SelectDsl<AsSelect<Member, DB>>,
        member_details::table: SelectDsl<AsSelect<MemberDetail, DB>>,
        LimitOffsetClause<LimitClause<Bound<BigInt, i64>>, OffsetClause<Bound<BigInt, i64>>>:
            QueryFragment<DB>,
        LimitOffsetClause<NoLimitClause, NoOffsetClause>: QueryFragment<DB>,
        (Member, MemberDetail):
            FromStaticSqlRow<(AsSelect<Member, DB>, AsSelect<MemberDetail, DB>), DB>,
        bool: ToSql<Bool, DB>,
        i64: FromSql<BigInt, DB>,
    {
        let where_expression = members::activated
            .eq(true)
            .and(members::allow_privacy_info_sharing.eq(true))
            .and(search_expression);

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

pub struct FacebookSearchExpressionGenerator;

impl FacebookSearchExpressionGenerator {
    pub fn postgresql(
        term: &str,
    ) -> Or<
        ILike<member_details::first_name, Bound<Text, &str>>,
        ILike<member_details::last_name, Bound<Text, &str>>,
    >
where {
        use diesel::PgTextExpressionMethods;
        member_details::first_name
            .ilike(term)
            .or(member_details::last_name.ilike(term))
    }

    pub fn sqlite(
        term: &str,
    ) -> Or<
        Like<member_details::first_name, Bound<Text, &str>>,
        Like<member_details::last_name, Bound<Text, &str>>,
    > {
        use diesel::TextExpressionMethods;
        member_details::first_name
            .like(term)
            .or(member_details::last_name.like(term))
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
