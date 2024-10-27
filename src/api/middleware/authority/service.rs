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
use crate::api::middleware::authority::Allowance;
use crate::generic::http::Method;
use crate::model::interface::client::UserClaims;
use crate::services::definitions::request::traits::RoleContainer;
use actix_jwt_auth_middleware::Authority;
use actix_web::body::MessageBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse};
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, FromRequest, Handler, HttpMessage};
use jwt_compact::Algorithm;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

pub struct AuthorityService<S, Claims, Algo, ReAuth, Args>
where
    Algo: Algorithm + Clone + 'static,
    Algo::SigningKey: Clone,
{
    service: Rc<S>,
    authority: Arc<Authority<Claims, Algo, ReAuth, Args>>,
    config: Arc<AuthorityConfig>,
    cache: Arc<Cache>,
}

impl<S, Claims, Algo, ReAuth, Args> AuthorityService<S, Claims, Algo, ReAuth, Args>
where
    Algo: Algorithm + Clone + 'static,
    Algo::SigningKey: Clone,
{
    pub fn new(
        service: S,
        authority: Authority<Claims, Algo, ReAuth, Args>,
        config: AuthorityConfig,
    ) -> Self {
        let service = Rc::new(service);
        let authority = Arc::new(authority);
        Self {
            service: service.clone(),
            authority: authority.clone(),
            config: Arc::new(config),
            cache: Arc::new(Cache::new()),
        }
    }
}

impl<S, B, Claims, Algo, ReAuth, Args> Service<ServiceRequest>
    for AuthorityService<S, Claims, Algo, ReAuth, Args>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + MessageBody,
    Claims: Serialize + DeserializeOwned + 'static,
    Algo: Algorithm + Clone + 'static + 'static,
    Algo::SigningKey: Clone,
    ReAuth: Handler<Args, Output = Result<(), Error>>,
    Args: FromRequest + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let authority = self.authority.clone();
        let service = self.service.clone();
        let config = self.config.clone();
        let cache = self.cache.clone();

        Box::pin(async move {
            let _ = authority.verify_service_request(&mut req).await;
            let user_claims = {
                let extensions = req.extensions();
                extensions.get::<UserClaims>().cloned()
            };

            let method = Method::from(req.method());
            let allowance = cache.lookup(
                config.deref(),
                method,
                &req.match_pattern().unwrap_or("".to_owned()),
            );

            if match_allowance(&user_claims, allowance) {
                let response = service.call(req).await?;
                Ok(response)
            } else {
                Err(ErrorUnauthorized("Unauthorized"))
            }
        })
    }
}

fn match_allowance(claims: &Option<UserClaims>, allowance: Allowance) -> bool {
    if allowance == Allowance::Any {
        true
    } else {
        match claims {
            Some(claims) => {
                if allowance == Allowance::LoggedInMember {
                    true
                } else {
                    for role in &claims.roles {
                        if claims.has_role(*role) {
                            return true;
                        }
                    }
                    return false;
                }
            }
            _ => false,
        }
    }
}

struct Cache {
    map: moka::sync::Cache<(Method, String), Allowance>,
}

impl Cache {
    fn new() -> Self {
        Self {
            map: moka::sync::Cache::new(10_000),
        }
    }
    fn lookup(&self, config: &AuthorityConfig, method: Method, path: &str) -> Allowance {
        match self.map.get(&(method.clone(), path.to_string())) {
            None => {
                let authorize = config.find(method.clone(), path);
                self.map
                    .insert((method, path.to_string()), authorize.clone());
                authorize
            }
            Some(result) => result.clone(),
        }
    }
}
