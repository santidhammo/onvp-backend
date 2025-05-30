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

use actix_jwt_auth_middleware::AuthError;
use actix_web::body::BoxBody;
use actix_web::http::{header, StatusCode};
use actix_web::web::BytesMut;
use actix_web::{HttpResponse, ResponseError};
use image::ImageError;
use jwt_compact::{ParseError, ValidationError};
use lettre::address::AddressError;
use r2d2;
use serde::Serialize;
use std::env::VarError;
use std::fmt::{Debug, Display, Formatter, Write};
use std::time::SystemTimeError;
use totp_rs::TotpUrlError;

pub type BackendResult<T> = Result<T, BackendError>;

#[derive(Debug, Clone)]
pub struct BackendError {
    pub kind: ErrorKind,
}

impl BackendError {
    pub(crate) fn not_enough_records() -> Self {
        Self {
            kind: ErrorKind::Database("Not enough records found".to_owned()),
        }
    }
    pub(crate) fn bad() -> Self {
        Self {
            kind: ErrorKind::BadRequest,
        }
    }
    pub(crate) fn insufficient_bytes(reason: &str) -> Self {
        Self {
            kind: ErrorKind::InsufficientBytes(reason.to_string()),
        }
    }
    pub(crate) fn forbidden() -> Self {
        Self {
            kind: ErrorKind::Forbidden,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    BadRequest,
    Database(String),
    ByteConversion(String),
    QrCodeGeneration(String),
    SystemTime(String),
    InsufficientBytes(String),
    Aes(String),
    Base64Decode(String),
    Base64Encode(String),
    TOTP(String),
    VarError(String),
    ConfigError(String),
    EmailError(String),
    TemplateError(String),
    Forbidden,
}

impl ErrorKind {
    pub fn simplified_string(&self) -> &'static str {
        match self {
            ErrorKind::BadRequest => "BAD_REQUEST",
            ErrorKind::Database(_) => "DATABASE",
            ErrorKind::ByteConversion(_) => "BYTE_CONVERSION",
            ErrorKind::QrCodeGeneration(_) => "QR_CODE_GENERATION",
            ErrorKind::SystemTime(_) => "SYSTEM_TIME",
            ErrorKind::InsufficientBytes(_) => "INSUFFICIENT_BYTES",
            ErrorKind::Aes(_) => "AES",
            ErrorKind::Base64Decode(_) => "BASE_64_DECODE",
            ErrorKind::Base64Encode(_) => "BASE_64_ENCODE",
            ErrorKind::TOTP(_) => "TOTP",
            ErrorKind::VarError(_) => "VAR_ERROR",
            ErrorKind::ConfigError(_) => "CONFIG_ERROR",
            ErrorKind::EmailError(_) => "EMAIL_ERROR",
            ErrorKind::TemplateError(_) => "TEMPLATE_ERROR",
            ErrorKind::Forbidden => "FORBIDDEN",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match &self {
            ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
            ErrorKind::Forbidden => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match &self {
            ErrorKind::Database(s) => s.to_string(),
            ErrorKind::ByteConversion(s) => s.to_string(),
            ErrorKind::QrCodeGeneration(s) => s.to_string(),
            ErrorKind::SystemTime(s) => s.to_string(),
            ErrorKind::BadRequest => "Bad Request".to_string(),
            ErrorKind::InsufficientBytes(s) => s.to_string(),
            ErrorKind::Aes(s) => s.to_string(),
            ErrorKind::Base64Decode(s) => s.to_string(),
            ErrorKind::Base64Encode(s) => s.to_string(),
            ErrorKind::TOTP(s) => s.to_string(),
            ErrorKind::VarError(s) => s.to_string(),
            ErrorKind::ConfigError(s) => s.to_string(),
            ErrorKind::EmailError(s) => s.to_string(),
            ErrorKind::TemplateError(s) => s.to_string(),
            ErrorKind::Forbidden => "Access Denied".to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
struct PreparedError {
    kind: String,
    message: String,
}

impl BackendError {
    pub fn byte_conversion<T: ToString>(s: T) -> BackendError {
        Self {
            kind: ErrorKind::ByteConversion(s.to_string()),
        }
    }

    pub fn qr_code_generation(reason: String) -> BackendError {
        Self {
            kind: ErrorKind::QrCodeGeneration(reason),
        }
    }

    pub fn as_json(&self) -> String {
        let pre = PreparedError {
            kind: self.kind.simplified_string().to_string(),
            message: self.kind.message(),
        };
        serde_json::to_string_pretty(&pre).unwrap_or_default()
    }
}

impl Display for BackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kind = self.kind.simplified_string();
        let explanation = self.kind.message();
        if explanation.len() > 0 {
            write!(f, "{}: {}", kind, explanation)
        } else {
            write!(f, "{}", kind)
        }
    }
}

impl ResponseError for BackendError {
    fn status_code(&self) -> StatusCode {
        self.kind.status_code()
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut res = HttpResponse::new(self.status_code());

        res.headers_mut()
            .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

        let mut buf = BytesMut::new();
        let json = self.as_json();
        let _ = buf.write_str(&json);
        res.set_body(BoxBody::new(buf))
    }
}

impl std::error::Error for BackendError {}

impl From<r2d2::Error> for BackendError {
    fn from(value: r2d2::Error) -> Self {
        Self {
            kind: ErrorKind::Database(value.to_string()),
        }
    }
}

impl From<SystemTimeError> for BackendError {
    fn from(value: SystemTimeError) -> Self {
        Self {
            kind: ErrorKind::SystemTime(value.to_string()),
        }
    }
}

impl From<aes_gcm::Error> for BackendError {
    fn from(value: aes_gcm::Error) -> Self {
        Self {
            kind: ErrorKind::Aes(value.to_string()),
        }
    }
}

impl From<base64::DecodeError> for BackendError {
    fn from(value: base64::DecodeError) -> Self {
        Self {
            kind: ErrorKind::Base64Decode(value.to_string()),
        }
    }
}

impl From<base64::EncodeSliceError> for BackendError {
    fn from(value: base64::EncodeSliceError) -> Self {
        Self {
            kind: ErrorKind::Base64Encode(value.to_string()),
        }
    }
}

impl From<TotpUrlError> for BackendError {
    fn from(value: TotpUrlError) -> Self {
        Self {
            kind: ErrorKind::TOTP(value.to_string()),
        }
    }
}

impl From<AuthError> for BackendError {
    fn from(_: AuthError) -> Self {
        Self {
            kind: ErrorKind::BadRequest,
        }
    }
}

impl From<ParseError> for BackendError {
    fn from(_: ParseError) -> Self {
        Self {
            kind: ErrorKind::BadRequest,
        }
    }
}

impl From<ValidationError> for BackendError {
    fn from(_: ValidationError) -> Self {
        Self {
            kind: ErrorKind::BadRequest,
        }
    }
}

impl From<std::io::Error> for BackendError {
    fn from(_: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::BadRequest,
        }
    }
}

impl From<ImageError> for BackendError {
    fn from(_: ImageError) -> Self {
        Self {
            kind: ErrorKind::BadRequest,
        }
    }
}

impl From<VarError> for BackendError {
    fn from(value: VarError) -> Self {
        Self {
            kind: ErrorKind::VarError(value.to_string()),
        }
    }
}

impl From<AddressError> for BackendError {
    fn from(value: AddressError) -> Self {
        Self {
            kind: ErrorKind::ConfigError(format!("Email configuration error: {value}")),
        }
    }
}

impl From<lettre::error::Error> for BackendError {
    fn from(value: lettre::error::Error) -> Self {
        Self {
            kind: ErrorKind::EmailError(value.to_string()),
        }
    }
}

impl From<lettre::transport::smtp::Error> for BackendError {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        Self {
            kind: ErrorKind::EmailError(value.to_string()),
        }
    }
}

impl From<handlebars::TemplateError> for BackendError {
    fn from(value: handlebars::TemplateError) -> Self {
        Self {
            kind: ErrorKind::TemplateError(value.to_string()),
        }
    }
}

impl From<handlebars::RenderError> for BackendError {
    fn from(value: handlebars::RenderError) -> Self {
        Self {
            kind: ErrorKind::TemplateError(value.to_string()),
        }
    }
}

impl From<diesel::result::Error> for BackendError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::InvalidCString(_) => Self {
                kind: ErrorKind::Database("".to_string()),
            },
            diesel::result::Error::NotFound => Self {
                // Pretend to be a bad request, the resource was not found
                kind: ErrorKind::BadRequest,
            },
            diesel::result::Error::DatabaseError(_, info) => Self {
                kind: ErrorKind::Database(info.message().to_string()),
            },
            diesel::result::Error::QueryBuilderError(e) => Self {
                kind: ErrorKind::Database(e.to_string()),
            },
            diesel::result::Error::DeserializationError(e) => Self {
                kind: ErrorKind::Database(e.to_string()),
            },
            diesel::result::Error::SerializationError(e) => Self {
                kind: ErrorKind::Database(e.to_string()),
            },
            diesel::result::Error::RollbackErrorOnCommit {
                rollback_error,
                commit_error,
            } => Self {
                kind: ErrorKind::Database(format!(
                    "Rollback Error: {} on Commit Error: {}",
                    rollback_error.to_string(),
                    commit_error.to_string()
                )),
            },
            diesel::result::Error::RollbackTransaction => Self {
                kind: ErrorKind::Database("Rollback Transaction".to_string()),
            },
            diesel::result::Error::AlreadyInTransaction => Self {
                kind: ErrorKind::Database("Already In Transaction".to_string()),
            },
            diesel::result::Error::NotInTransaction => Self {
                kind: ErrorKind::Database("Not In Transaction".to_string()),
            },
            diesel::result::Error::BrokenTransactionManager => Self {
                kind: ErrorKind::Database("Broken Transaction Manager".to_string()),
            },
            _ => Self {
                kind: ErrorKind::Database("Unspecified error".to_string()),
            },
        }
    }
}
