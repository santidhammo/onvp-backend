use crate::result;
use actix_jwt_auth_middleware::FromRequest;
use diesel::expression::AsExpression;
use diesel::internal::derives::as_expression::Bound;
use diesel::sql_types::Int4;
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
struct UserClaims {
    id: u32,
    role: Role,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, FromSqlRow, Eq, PartialEq, ToSchema)]
#[diesel(sql_type = Int4)]
#[repr(u8)]
pub enum Role {
    Public = 0x0,
    Member = 0x1,
    OrchestraCommittee = 0x2,
    Operator = 0xFF,
}

impl TryFrom<u8> for Role {
    type Error = result::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Public),
            1 => Ok(Self::Member),
            2 => Ok(Self::OrchestraCommittee),
            3..=0xFE => Err(result::Error::byte_conversion(value)),
            _ => Ok(Self::Operator),
        }
    }
}

impl AsExpression<Int4> for Role {
    type Expression = Bound<Int4, i32>;

    fn as_expression(self) -> Self::Expression {
        <i32 as AsExpression<Int4>>::as_expression(self as i32)
    }
}
