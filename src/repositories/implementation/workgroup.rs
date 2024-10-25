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
use crate::repositories::implementation::search_expressions::{
    MemberSearchExpressionGenerator, WorkgroupSearchExpressionGenerator,
};
use crate::schema::member_address_details;
use crate::schema::member_details;
use crate::schema::members;
use crate::schema::workgroup_member_relationships;
use crate::schema::workgroup_role_associations;
use crate::schema::workgroups;
use actix_web::web::Data;
use diesel::backend::{Backend, SqlDialect};
use diesel::connection::LoadConnection;
use diesel::deserialize::{FromSql, FromStaticSqlRow};
use diesel::dsl::{exists, not, AsSelect, InnerJoinQuerySource, Limit};
use diesel::expression::is_aggregate::No;
use diesel::expression::ValidGrouping;
use diesel::internal::derives::as_expression::Bound;
use diesel::internal::derives::multiconnection::sql_dialect::exists_syntax::AnsiSqlExistsSyntax;
use diesel::internal::derives::multiconnection::sql_dialect::select_statement_syntax::AnsiSqlSelectStatement;
use diesel::internal::derives::multiconnection::{
    DieselReserveSpecialization, LimitClause, LimitOffsetClause, NoLimitClause, NoOffsetClause,
    OffsetClause,
};
use diesel::query_builder::{QueryFragment, QueryId};
use diesel::query_dsl::limit_dsl::LimitDsl;
use diesel::query_dsl::select_dsl::SelectDsl;
use diesel::serialize::ToSql;
use diesel::sql_types::{BigInt, Bool, HasSqlType, Integer};
use diesel::{
    AppearsOnTable, BoolExpressionMethods, Connection, Expression, ExpressionMethods, QueryDsl,
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
            .transaction::<(usize, Vec<Workgroup>), BackendError, _>(|conn| match conn {
                DatabaseConnection::PostgreSQL(ref mut conn) => {
                    let filter =
                        WorkgroupSearchExpressionGenerator::postgresql(&like_search_string);
                    self.search_workgroups(conn, page_offset, &filter)
                }

                DatabaseConnection::SQLite(ref mut conn) => {
                    let filter = WorkgroupSearchExpressionGenerator::sqlite(&like_search_string);
                    self.search_workgroups(conn, page_offset, &filter)
                }
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
            .transaction::<(usize, Vec<ExtendedMember>), BackendError, _>(|conn| match conn {
                DatabaseConnection::PostgreSQL(conn) => {
                    let filter = MemberSearchExpressionGenerator::postgresql(&like_search_string);
                    self.search_available_members(conn, page_offset, &filter, workgroup_id)
                }
                DatabaseConnection::SQLite(conn) => {
                    let filter = MemberSearchExpressionGenerator::sqlite(&like_search_string);
                    self.search_available_members(conn, page_offset, &filter, workgroup_id)
                }
            })?;
        Ok((total_count, self.page_size, extended_members))
    }
}

impl Implementation {
    fn search_workgroups<DB, TSearchExpression, TConnection>(
        &self,
        conn: &mut TConnection,
        page_offset: usize,
        search_expression: TSearchExpression,
    ) -> Result<(usize, Vec<Workgroup>), BackendError>
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
            + AppearsOnTable<workgroups::table>
            + QueryId,
        Limit<workgroups::table>: LimitDsl,
        workgroups::table: SelectDsl<AsSelect<Workgroup, DB>>,
        LimitOffsetClause<LimitClause<Bound<BigInt, i64>>, OffsetClause<Bound<BigInt, i64>>>:
            QueryFragment<DB>,
        LimitOffsetClause<NoLimitClause, NoOffsetClause>: QueryFragment<DB>,
        (Workgroup,): FromStaticSqlRow<(AsSelect<Workgroup, DB>,), DB>,
        bool: ToSql<Bool, DB>,
        i64: FromSql<BigInt, DB>,
    {
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

    fn search_available_members<DB, TSearchExpression, TConnection>(
        &self,
        conn: &mut TConnection,
        page_offset: usize,
        search_expression: TSearchExpression,
        workgroup_id: i32,
    ) -> Result<(usize, Vec<ExtendedMember>), BackendError>
    where
        DB: Backend<
                SelectStatementSyntax = AnsiSqlSelectStatement,
                ExistsSyntax = AnsiSqlExistsSyntax,
            > + SqlDialect
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
        Limit<workgroup_member_relationships::table>: LimitDsl,
        members::table: SelectDsl<AsSelect<Member, DB>>,
        member_details::table: SelectDsl<AsSelect<MemberDetail, DB>>,
        LimitOffsetClause<LimitClause<Bound<BigInt, i64>>, OffsetClause<Bound<BigInt, i64>>>:
            QueryFragment<DB>,
        LimitOffsetClause<NoLimitClause, NoOffsetClause>: QueryFragment<DB>,
        (Member, MemberDetail):
            FromStaticSqlRow<(AsSelect<Member, DB>, AsSelect<MemberDetail, DB>), DB>,
        bool: ToSql<Bool, DB>,
        i32: ToSql<Integer, DB>,
        i64: FromSql<BigInt, DB>,
    {
        let sub_table = QueryDsl::select(
            workgroup_member_relationships::table,
            workgroup_member_relationships::member_id,
        )
        .filter(
            workgroup_member_relationships::workgroup_id
                .eq(workgroup_id)
                .and(workgroup_member_relationships::member_id.eq(members::id)),
        );

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
