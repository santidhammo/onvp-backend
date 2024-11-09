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
use crate::generic::security::ClaimRoles;
use crate::generic::storage::database::DatabaseConnection;
use crate::generic::{search_helpers, Injectable};
use crate::model::primitives::Role;
use crate::model::storage::entities::Page;
use crate::repositories::definitions::PageRepository;
use crate::repositories::implementation::search_expressions::PageSearchExpressionGenerator;
use crate::schema::*;
use actix_web::web::Data;
use diesel::dsl::exists;
use diesel::prelude::*;
use std::sync::Arc;

pub struct Implementation {
    page_size: usize,
}

impl PageRepository for Implementation {
    fn create(&self, conn: &mut DatabaseConnection, page: Page) -> BackendResult<()> {
        conn.transaction::<_, BackendError, _>(|conn| {
            let page_id: i32 = diesel::insert_into(pages::table)
                .values(page)
                .returning(pages::id)
                .get_result(conn)?;

            self.reset_roles(conn, page_id)?;
            Ok(())
        })
    }

    fn update(&self, conn: &mut DatabaseConnection, page: Page) -> BackendResult<()> {
        diesel::update(pages::table)
            .filter(pages::id.eq(page.id))
            .set(page)
            .execute(conn)?;
        Ok(())
    }

    fn find_by_id(&self, conn: &mut DatabaseConnection, page_id: i32) -> BackendResult<Page> {
        let page = pages::table
            .filter(pages::id.eq(page_id))
            .select(Page::as_select())
            .first::<Page>(conn)?;
        Ok(page)
    }

    fn list_by_parent_id(
        &self,
        conn: &mut DatabaseConnection,
        parent_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<Vec<Page>> {
        let pages = match conn {
            DatabaseConnection::PostgreSQL(conn) => {
                let sub_table = page_access_policies::table
                    .select(page_access_policies::page_id)
                    .distinct()
                    .filter(roles.generate_policy_expression(&page_access_policies::system_role));

                if parent_id == 0 {
                    pages::table
                        .filter(
                            pages::parent_id
                                .eq(parent_id)
                                .or(pages::parent_id.is_null())
                                .and(exists(sub_table)),
                        )
                        .select(Page::as_select())
                        .order_by(pages::title)
                        .load(conn)?
                } else {
                    pages::table
                        .filter(pages::parent_id.eq(parent_id).and(exists(sub_table)))
                        .select(Page::as_select())
                        .order_by(pages::title)
                        .load(conn)?
                }
            }
            DatabaseConnection::SQLite(conn) => {
                let sub_table = page_access_policies::table
                    .select(page_access_policies::page_id)
                    .distinct()
                    .filter(roles.generate_policy_expression(&page_access_policies::system_role));

                if parent_id == 0 {
                    pages::table
                        .filter(
                            pages::parent_id
                                .eq(parent_id)
                                .or(pages::parent_id.is_null())
                                .and(exists(sub_table)),
                        )
                        .select(Page::as_select())
                        .order_by(pages::title)
                        .load(conn)?
                } else {
                    pages::table
                        .filter(pages::parent_id.eq(parent_id).and(exists(sub_table)))
                        .select(Page::as_select())
                        .order_by(pages::title)
                        .load(conn)?
                }
            }
        };

        Ok(pages)
    }

    fn find_associated_roles_by_id(
        &self,
        conn: &mut DatabaseConnection,
        page_id: i32,
    ) -> BackendResult<Vec<Role>> {
        let associated_roles: Vec<Role> = page_access_policies::table
            .filter(page_access_policies::page_id.eq(page_id))
            .select(page_access_policies::system_role)
            .load(conn)?;

        Ok(associated_roles)
    }

    fn delete(&self, conn: &mut DatabaseConnection, page_id: i32) -> BackendResult<()> {
        diesel::delete(pages::table)
            .filter(pages::id.eq(page_id))
            .execute(conn)?;
        Ok(())
    }

    fn reset_roles(&self, conn: &mut DatabaseConnection, page_id: i32) -> BackendResult<()> {
        conn.transaction::<_, BackendError, _>(|conn| {
            diesel::delete(page_access_policies::table)
                .filter(page_access_policies::page_id.eq(page_id))
                .execute(conn)?;

            diesel::insert_into(page_access_policies::table)
                .values((
                    page_access_policies::page_id.eq(page_id),
                    page_access_policies::system_role.eq(Role::Operator),
                ))
                .execute(conn)?;

            Ok(())
        })
    }

    fn assign_roles(
        &self,
        conn: &mut DatabaseConnection,
        page_id: i32,
        roles: &Vec<Role>,
    ) -> BackendResult<()> {
        conn.transaction::<_, BackendError, _>(|conn| {
            for role in roles {
                if role != &Role::Operator {
                    diesel::insert_into(page_access_policies::table)
                        .values((
                            page_access_policies::page_id.eq(page_id),
                            page_access_policies::system_role.eq(role),
                        ))
                        .execute(conn)?;
                }
            }
            Ok(())
        })
    }

    fn search(
        &self,
        conn: &mut DatabaseConnection,
        page_offset: usize,
        term: &str,
        roles: &ClaimRoles,
    ) -> BackendResult<(usize, usize, Vec<Page>)> {
        let like_search_string = search_helpers::create_like_string(term);
        let (total_count, pages) =
            conn.transaction::<(usize, Vec<Page>), BackendError, _>(|conn| match conn {
                DatabaseConnection::PostgreSQL(conn) => {
                    let filter = PageSearchExpressionGenerator::postgresql(&like_search_string);
                    search_impl::search(conn, self.page_size, page_offset, roles, &filter)
                }
                DatabaseConnection::SQLite(conn) => {
                    let filter = PageSearchExpressionGenerator::sqlite(&like_search_string);
                    search_impl::search(conn, self.page_size, page_offset, roles, &filter)
                }
            })?;
        Ok((total_count, self.page_size, pages))
    }
}

mod search_impl {
    use diesel::backend::{Backend, SqlDialect};
    use diesel::connection::LoadConnection;
    use diesel::deserialize::{FromSql, FromStaticSqlRow};
    use diesel::dsl::{exists, AsSelect, Limit};
    use diesel::expression::is_aggregate::No;
    use diesel::expression::ValidGrouping;

    use crate::generic::result::BackendError;
    use crate::generic::security::ClaimRoles;
    use crate::model::primitives::Role;
    use crate::model::storage::entities::Page;
    use crate::schema::*;
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
        AppearsOnTable, BoolExpressionMethods, Connection, Expression, QueryDsl, RunQueryDsl,
        SelectableHelper,
    };

    pub(super) fn search<DB, TConnection, TSearchExpression>(
        conn: &mut TConnection,
        page_size: usize,
        page_offset: usize,
        roles: &ClaimRoles,
        search_expression: TSearchExpression,
    ) -> Result<(usize, Vec<Page>), BackendError>
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
            + AppearsOnTable<pages::table>
            + QueryId,

        Limit<pages::table>: LimitDsl,
        Limit<page_access_policies::table>: LimitDsl,
        pages::table: SelectDsl<AsSelect<Page, DB>>,
        LimitOffsetClause<LimitClause<Bound<BigInt, i64>>, OffsetClause<Bound<BigInt, i64>>>:
            QueryFragment<DB>,
        LimitOffsetClause<NoLimitClause, NoOffsetClause>: QueryFragment<DB>,
        (Page,): FromStaticSqlRow<(AsSelect<Page, DB>,), DB>,
        bool: ToSql<Bool, DB>,
        Role: ToSql<Integer, DB>,
        i64: FromSql<BigInt, DB>,
        i32: ToSql<Integer, DB>,
    {
        let sub_table =
            QueryDsl::select(page_access_policies::table, page_access_policies::page_id)
                .distinct()
                .filter(roles.generate_policy_expression(&page_access_policies::system_role));

        let where_expression = search_expression.and(exists(sub_table));

        let total_count: usize = pages::table
            .filter(&where_expression)
            .count()
            .get_result::<i64>(conn)? as usize;

        let result: Vec<(Page,)> = QueryDsl::select(
            QueryDsl::limit(
                pages::table
                    .filter(&where_expression)
                    .order_by(pages::title),
                page_size as i64,
            )
            .offset((page_offset * page_size) as i64),
            (Page::as_select(),),
        )
        .load(conn)?;

        Ok((total_count, result.iter().map(|(p,)| p.clone()).collect()))
    }
}

impl Injectable<(), dyn PageRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn PageRepository> {
        let arc: Arc<dyn PageRepository> = Arc::new(Self {
            page_size: *SEARCH_PAGE_SIZE,
        });
        Data::from(arc)
    }
}
