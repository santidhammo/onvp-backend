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
use chrono::{Datelike, NaiveDate};
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::expression::AsExpression;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Integer;
use diesel::FromSqlRow;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    FromSqlRow,
    Eq,
    PartialEq,
    ToSchema,
    Hash,
    AsExpression,
)]
#[diesel(sql_type = Integer)]
#[repr(u8)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    Public = 0x0,
    Member = 0x1,
    OrchestraCommittee = 0x2,
    Director = 0x3,
    Operator = 0xFF,
}

impl<DB> FromSql<Integer, DB> for Role
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let v = i32::from_sql(bytes)?;
        if v > u8::MAX as i32 || v < 0 {
            Err(format!("Could not expand variant into role: {}", v).into())
        } else {
            Role::try_from(v as u8).map_err(|e| e.into())
        }
    }
}

impl<DB> ToSql<Integer, DB> for Role
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> diesel::serialize::Result {
        match self {
            Role::Public => (Role::Public as i32).to_sql(out),
            Role::Member => (Role::Member as i32).to_sql(out),
            Role::OrchestraCommittee => (Role::OrchestraCommittee as i32).to_sql(out),
            Role::Director => (Role::Director as i32).to_sql(out),
            Role::Operator => (Role::Operator as i32).to_sql(out),
        }
    }
}

impl TryFrom<u8> for Role {
    type Error = BackendError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Public),
            1 => Ok(Self::Member),
            2 => Ok(Self::OrchestraCommittee),
            3 => Ok(Self::Director),
            0xFF => Ok(Self::Operator),
            x => Err(BackendError::byte_conversion(format!(
                "Could not expand variant into role: {}",
                x
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoleClass {
    Member,
    Workgroup,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleComposition {
    pub(crate) roles: Vec<Role>,
}

impl From<Role> for RoleComposition {
    fn from(role: Role) -> Self {
        Self { roles: vec![role] }
    }
}

impl From<HashSet<Role>> for RoleComposition {
    fn from(roles: HashSet<Role>) -> Self {
        Self {
            roles: roles.into_iter().map(Role::from).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventDate {
    #[schema(example = 1)]
    pub day: u32,
    #[schema(example = 1)]
    pub month: u32,
    #[schema(example = 2123)]
    pub year: i32,
}

impl EventDate {
    pub fn as_validated(&self) -> BackendResult<NaiveDate> {
        let maybe_naive = NaiveDate::from_ymd_opt(self.year, self.month, self.day);
        match maybe_naive {
            Some(naive) => Ok(naive),
            None => Err(BackendError::bad()),
        }
    }

    pub fn validate(&self) -> BackendResult<()> {
        let _ = self.as_validated()?;
        Ok(())
    }
}

impl From<&NaiveDate> for EventDate {
    fn from(value: &NaiveDate) -> Self {
        Self {
            day: value.day(),
            month: value.month(),
            year: value.year(),
        }
    }
}

pub type Identity = i32;
