use aes_gcm::aead::OsRng;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit};
use base64::{engine::general_purpose, Engine as _};
use diesel::expression::AsExpression;
use diesel::internal::derives::as_expression::Bound;
use diesel::sql_types::Int4;
use diesel::FromSqlRow;
use std::env;
use std::sync::LazyLock;
use totp_rs::{Algorithm, Secret, TOTP};

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

pub static OTP_CIPHER: LazyLock<Aes256Gcm> = LazyLock::new(|| {
    let key = env::var("OTP_KEY").expect("OTP_KEY must be set");
    let buffer = general_purpose::STANDARD
        .decode(&key)
        .expect("invalid OTP key, not properly encoded");
    let key = Key::<Aes256Gcm>::from_slice(&buffer);
    Aes256Gcm::new(&key)
});

pub fn generate_encoded_nonce() -> String {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    general_purpose::STANDARD.encode(&nonce)
}

pub fn generate_totp(
    cipher_text: Vec<u8>,
    email_address: String,
) -> Result<TOTP, totp_rs::TotpUrlError> {
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Raw(cipher_text).to_bytes().unwrap(),
        Some("ONVP".to_owned()),
        email_address,
    )
}
