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
use crate::model::storage::entities::Image;
use crate::repositories::definitions::ImageRepository;
use crate::repositories::implementation::search_expressions::ImageSearchExpressionGenerator;
use crate::schema::*;
use actix_web::web::Data;
use diesel::prelude::*;
use std::sync::Arc;

pub struct Implementation {
    page_size: usize,
}

impl ImageRepository for Implementation {
    fn create(&self, conn: &mut DatabaseConnection, image: Image) -> BackendResult<()> {
        conn.transaction::<_, BackendError, _>(|conn| {
            let image_id: i32 = diesel::insert_into(images::table)
                .values(image)
                .returning(images::id)
                .get_result(conn)?;

            self.reset_roles(conn, image_id)?;
            Ok(())
        })
    }

    fn find_by_id(&self, conn: &mut DatabaseConnection, image_id: i32) -> BackendResult<Image> {
        let image = images::table
            .filter(images::id.eq(image_id))
            .select(Image::as_select())
            .first::<Image>(conn)?;
        Ok(image)
    }

    fn find_associated_roles_by_id(
        &self,
        conn: &mut DatabaseConnection,
        image_id: i32,
    ) -> BackendResult<Vec<Role>> {
        let associated_roles: Vec<Role> = image_access_policies::table
            .filter(image_access_policies::image_id.eq(image_id))
            .select(image_access_policies::system_role)
            .load(conn)?;

        Ok(associated_roles)
    }

    fn delete(&self, conn: &mut DatabaseConnection, image_id: i32) -> BackendResult<()> {
        diesel::delete(images::table)
            .filter(images::id.eq(image_id))
            .execute(conn)?;
        Ok(())
    }

    fn reset_roles(&self, conn: &mut DatabaseConnection, image_id: i32) -> BackendResult<()> {
        diesel::delete(image_access_policies::table)
            .filter(image_access_policies::image_id.eq(image_id))
            .execute(conn)?;

        diesel::insert_into(image_access_policies::table)
            .values((
                image_access_policies::image_id.eq(image_id),
                image_access_policies::system_role.eq(Role::Operator),
            ))
            .execute(conn)?;

        Ok(())
    }

    fn assign_roles(
        &self,
        conn: &mut DatabaseConnection,
        image_id: i32,
        roles: &Vec<Role>,
    ) -> BackendResult<()> {
        conn.transaction::<_, BackendError, _>(|conn| {
            for role in roles {
                if role != &Role::Operator {
                    diesel::insert_into(image_access_policies::table)
                        .values((
                            image_access_policies::image_id.eq(image_id),
                            image_access_policies::system_role.eq(role),
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
    ) -> BackendResult<(usize, usize, Vec<Image>)> {
        let like_search_string = search_helpers::create_like_string(term);
        let (total_count, pages) =
            conn.transaction::<(usize, Vec<Image>), BackendError, _>(|conn| match conn {
                DatabaseConnection::PostgreSQL(conn) => {
                    let filter = ImageSearchExpressionGenerator::postgresql(&like_search_string);
                    search_impl::search(conn, self.page_size, page_offset, &filter)
                }
                DatabaseConnection::SQLite(conn) => {
                    let filter = ImageSearchExpressionGenerator::sqlite(&like_search_string);
                    search_impl::search(conn, self.page_size, page_offset, &filter)
                }
            })?;
        Ok((total_count, self.page_size, pages))
    }
}

mod search_impl {
    use diesel::backend::{Backend, SqlDialect};
    use diesel::connection::LoadConnection;
    use diesel::deserialize::{FromSql, FromStaticSqlRow};
    use diesel::dsl::{AsSelect, Limit};
    use diesel::expression::is_aggregate::No;
    use diesel::expression::ValidGrouping;

    use crate::generic::result::BackendError;
    use crate::model::storage::entities::Image;
    use crate::schema::*;
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
    use diesel::sql_types::{BigInt, Bool, HasSqlType, Integer};
    use diesel::{AppearsOnTable, Connection, Expression, QueryDsl, RunQueryDsl, SelectableHelper};

    pub(super) fn search<DB, TConnection, TSearchExpression>(
        conn: &mut TConnection,
        page_size: usize,
        page_offset: usize,
        search_expression: TSearchExpression,
    ) -> Result<(usize, Vec<Image>), BackendError>
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
            + AppearsOnTable<images::table>
            + QueryId,

        Limit<images::table>: LimitDsl,
        images::table: SelectDsl<AsSelect<Image, DB>>,
        LimitOffsetClause<LimitClause<Bound<BigInt, i64>>, OffsetClause<Bound<BigInt, i64>>>:
            QueryFragment<DB>,
        LimitOffsetClause<NoLimitClause, NoOffsetClause>: QueryFragment<DB>,
        (Image,): FromStaticSqlRow<(AsSelect<Image, DB>,), DB>,
        bool: ToSql<Bool, DB>,
        i64: FromSql<BigInt, DB>,
        i32: ToSql<Integer, DB>,
    {
        let total_count: usize = images::table
            .filter(&search_expression)
            .count()
            .get_result::<i64>(conn)? as usize;

        let result: Vec<(Image,)> = QueryDsl::select(
            QueryDsl::limit(
                images::table
                    .filter(&search_expression)
                    .order_by(images::title),
                page_size as i64,
            )
            .offset((page_offset * page_size) as i64),
            (Image::as_select(),),
        )
        .load(conn)?;

        Ok((total_count, result.iter().map(|(i,)| i.clone()).collect()))
    }
}

impl Injectable<(), dyn ImageRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn ImageRepository> {
        let arc: Arc<dyn ImageRepository> = Arc::new(Self {
            page_size: *SEARCH_PAGE_SIZE,
        });
        Data::from(arc)
    }
}
