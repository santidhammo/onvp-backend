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
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Del,
    Options,
    Head,
    Patch,
    Trace,
}
impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Del => write!(f, "DELETE"),
            Method::Options => write!(f, "OPTIONS"),
            Method::Head => write!(f, "HEAD"),
            Method::Patch => write!(f, "PATCH"),
            Method::Trace => write!(f, "TRACE"),
        }
    }
}

impl Debug for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<&actix_web::http::Method> for Method {
    fn from(value: &actix_web::http::Method) -> Self {
        if value == &actix_web::http::Method::GET {
            Method::Get
        } else if value == &actix_web::http::Method::POST {
            Method::Post
        } else if value == &actix_web::http::Method::PUT {
            Method::Put
        } else if value == &actix_web::http::Method::DELETE {
            Method::Del
        } else if value == &actix_web::http::Method::OPTIONS {
            Method::Options
        } else if value == &actix_web::http::Method::HEAD {
            Method::Head
        } else if value == &actix_web::http::Method::PATCH {
            Method::Patch
        } else if value == &actix_web::http::Method::TRACE {
            Method::Trace
        } else {
            // This should not occur, but generally trace is unlikely to be allowed
            Method::Trace
        }
    }
}

pub struct MethodError;

impl Debug for MethodError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Method error while converting method")
    }
}

impl Display for MethodError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Method error while converting method")
    }
}

impl Error for MethodError {}
