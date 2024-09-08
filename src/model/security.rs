use crate::{result, Error};
use actix_jwt_auth_middleware::FromRequest;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::expression::AsExpression;
use diesel::internal::derives::as_expression::Bound;
use diesel::sql_types::{Int4, Integer};
use diesel::FromSqlRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberRoleAssociation {
    #[schema(example = 1)]
    member_id: i32,
    #[schema(example = 1)]
    system_role: i32,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenData {
    #[schema(example = "abc")]
    pub activation_string: String,
    #[schema(example = "123456")]
    pub token: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginData {
    #[schema(example = "john@doe.void")]
    pub email_address: String,
    #[schema(example = "123456")]
    pub token: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, FromRequest)]
pub struct UserClaims {
    pub email_address: String,
    pub roles: Vec<Role>,
}

impl UserClaims {
    pub(crate) fn new(email_address: &str, roles: &Vec<Role>) -> Self {
        Self {
            email_address: email_address.to_string(),
            roles: roles.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, FromSqlRow, Eq, PartialEq, ToSchema, Hash)]
#[diesel(sql_type = Int4)]
#[repr(u8)]
pub enum Role {
    Public = 0x0,
    Member = 0x1,
    OrchestraCommittee = 0x2,
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

impl TryFrom<u8> for Role {
    type Error = result::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Public),
            1 => Ok(Self::Member),
            2 => Ok(Self::OrchestraCommittee),
            0xFF => Ok(Self::Operator),
            x => Err(Error::byte_conversion(format!(
                "Could not expand variant into role: {}",
                x
            ))),
        }
    }
}

impl AsExpression<Int4> for Role {
    type Expression = Bound<Int4, i32>;

    fn as_expression(self) -> Self::Expression {
        <i32 as AsExpression<Int4>>::as_expression(self as i32)
    }
}
