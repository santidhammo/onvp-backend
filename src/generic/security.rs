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
use crate::model::interface::client::UserClaims;
use crate::model::primitives::Role;
use crate::services::definitions::request::traits::RoleContainer;
use actix_jwt_auth_middleware::FromRequest;
use diesel::backend::{Backend, SqlDialect};
use diesel::expression::is_aggregate::No;
use diesel::expression::{AsExpression, ValidGrouping};
use diesel::internal::derives::multiconnection::DieselReserveSpecialization;
use diesel::query_builder::{QueryFragment, QueryId};
use diesel::serialize::ToSql;
use diesel::sql_types::{Bool, HasSqlType, Integer};
use diesel::{
    AppearsOnTable, BoolExpressionMethods, BoxableExpression, Expression, ExpressionMethods,
    SelectableExpression,
};
use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use std::collections::HashSet;
pub use totp_rs::TOTP;

pub fn operator_state_guard(claims: &UserClaims) -> BackendResult<()> {
    if claims.has_role(Role::Operator) {
        Ok(())
    } else {
        Err(BackendError::bad())
    }
}

pub fn generate_activation_string() -> String {
    let validation_string = Alphanumeric.sample_string(&mut thread_rng(), 32);
    validation_string
}

#[non_exhaustive]
#[derive(Clone, FromRequest)]
pub struct ClaimRoles(HashSet<Role>);

impl ClaimRoles {
    pub fn set(&self) -> &HashSet<Role> {
        &self.0
    }

    pub fn generate_policy_expression<DB, QS, Exp>(
        &self,
        exp: &'static Exp,
    ) -> Box<dyn BoxableExpression<QS, DB, SqlType = Bool>>
    where
        QS: 'static,
        DB: Backend + SqlDialect + DieselReserveSpecialization + HasSqlType<Bool> + 'static,
        Exp: QueryFragment<DB>
            + Expression<SqlType = Integer>
            + ValidGrouping<(), IsAggregate = No>
            + AppearsOnTable<QS>
            + QueryId
            + SelectableExpression<QS>
            + Send,
        &'static Exp: Send,
        i32: AsExpression<Integer> + ToSql<Integer, DB>,
    {
        let mut result: Box<dyn BoxableExpression<QS, DB, SqlType = Bool>> = Box::new(exp.eq(exp));

        for role in self.set() {
            let r = *role;
            let lhs: Box<dyn BoxableExpression<QS, DB, SqlType = Bool>> =
                Box::new(ExpressionMethods::eq(exp, r));

            result = Box::new(lhs.or(result));
        }

        result
    }
}

impl RoleContainer for ClaimRoles {
    fn has_role(&self, role: Role) -> bool {
        self.set().contains(&role)
    }
}

impl From<&Option<UserClaims>> for ClaimRoles {
    fn from(value: &Option<UserClaims>) -> Self {
        let mut result = HashSet::new();
        result.insert(Role::Public);
        if let Some(claims) = value {
            for role in &claims.roles {
                let _ = result.insert(*role);
            }
        }
        Self(result)
    }
}
