use crate::Error;
use actix_jwt_auth_middleware::FromRequest;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::expression::AsExpression;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Integer;
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
    pub fn new(email_address: &str, roles: &Vec<Role>) -> Self {
        Self {
            email_address: email_address.to_string(),
            roles: roles.clone(),
        }
    }

    /// Checks if the user claims contain the given role
    pub fn has_role(&self, role: Role) -> bool {
        for intern in &self.roles {
            if intern == &role {
                return true;
            }
        }
        false
    }
}

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
            Role::Operator => (Role::Operator as i32).to_sql(out),
        }
    }
}

impl TryFrom<u8> for Role {
    type Error = Error;

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
