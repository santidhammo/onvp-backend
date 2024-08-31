use diesel::expression::AsExpression;
use diesel::internal::derives::as_expression::Bound;
use diesel::sql_types::Int4;
use diesel::FromSqlRow;

#[derive(Debug, Clone, Copy, FromSqlRow, Eq, PartialEq)]
#[diesel(sql_type = Int4)]
#[repr(u8)]
pub enum Role {
    Public = 0x0,
    Member = 0x1,
    OrchestraCommittee = 0x2,
    Operator = 0xFF,
}

impl TryFrom<u8> for Role {
    type Error = role::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Public),
            1 => Ok(Self::Member),
            2 => Ok(Self::OrchestraCommittee),
            3..=0xFE => Err(role::Error::new(role::ErrorKind::ByteConversion)),
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

// impl<DB: Backend> ToSql<Int4, DB> for Role {
//     fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> diesel::serialize::Result {
//         ToSql::to_sql(&(*self as i32), out)
//     }
// }

// impl<DB: Backend> FromSql<Int4, DB> for Role {
//     fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
//         let step1 = i32::from_sql(bytes.into())?;
//         if step1 < 0 || step1 > u8::MAX as i32 {
//             Err(role::Error::new(role::ErrorKind::ByteConversion)).into()
//         } else {
//             match Self::try_from(step1 as u8) {
//                 Ok(step2) => Ok(step2),
//                 Err(e) => Err(e.into()),
//             }
//         }
//     }
// }

pub mod role {
    use std::fmt::{Debug, Display, Formatter};

    #[derive(Debug, Clone, Copy)]
    pub struct Error {
        kind: ErrorKind,
    }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum ErrorKind {
        ByteConversion = 0,
    }

    impl Error {
        pub fn new(kind: ErrorKind) -> Self {
            Self { kind }
        }
    }

    impl Display for Error {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self.kind {
                ErrorKind::ByteConversion => write!(
                    f,
                    "Illegal conversion of enumeration byte value to enumeration type"
                ),
            }
        }
    }

    impl std::error::Error for Error {}
}
