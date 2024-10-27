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
use crate::api::middleware::authority::config::AuthorityConfig;
use crate::api::middleware::authority::service::AuthorityService;
use actix_jwt_auth_middleware::Authority;
use actix_web::body::MessageBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, FromRequest, Handler};
use jwt_compact::Algorithm;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::{ready, Ready};

pub struct AuthorityMiddleware<Claims, Algo, ReAuth, Args>
where
    Claims: Clone,
    Algo: Algorithm + Clone + 'static,
    Algo::SigningKey: Clone,
    Algo::VerifyingKey: Clone,
    Args: Clone + FromRequest + 'static,
{
    authority: Authority<Claims, Algo, ReAuth, Args>,
    config: AuthorityConfig,
}

impl<Claims, Algo, ReAuth, Args> AuthorityMiddleware<Claims, Algo, ReAuth, Args>
where
    Claims: Clone,
    Algo: Algorithm + Clone + 'static,
    Algo::SigningKey: Clone,
    Algo::VerifyingKey: Clone,
    Args: Clone + FromRequest + 'static,
{
    pub fn new(authority: Authority<Claims, Algo, ReAuth, Args>, config: AuthorityConfig) -> Self {
        Self { authority, config }
    }
}
impl<S, B, Claims, Algo, ReAuth, Args> Transform<S, ServiceRequest>
    for AuthorityMiddleware<Claims, Algo, ReAuth, Args>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    Claims: Serialize + DeserializeOwned + Clone + 'static + 'static,
    Algo: Algorithm + Clone + 'static + 'static,
    Algo::SigningKey: Clone,
    Algo::VerifyingKey: Clone,
    B: 'static + MessageBody,
    ReAuth: Handler<Args, Output = Result<(), Error>>,
    Args: FromRequest + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthorityService<S, Claims, Algo, ReAuth, Args>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorityService::new(
            service,
            self.authority.clone(),
            self.config.clone(),
        )))
    }
}
