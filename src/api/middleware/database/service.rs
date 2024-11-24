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
use crate::generic::storage::session::SessionManager;
use actix_web::body::MessageBody;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

pub struct DatabaseService<S> {
    service: Rc<S>,
}

impl<S> DatabaseService<S> {
    pub fn new(service: S) -> Self {
        let service = Rc::new(service);
        Self {
            service: service.clone(),
        }
    }
}

impl<S, B> Service<ServiceRequest> for DatabaseService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + MessageBody,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let manager = req.app_data::<Data<dyn SessionManager>>();
            if let Some(manager) = manager {
                let mut session = manager.prepare()?;
                req.extensions_mut().insert(session.clone());
                let response = service.call(req).await;
                match response {
                    Ok(response) => {
                        session.commit()?;
                        Ok(response)
                    }
                    Err(e) => {
                        session.rollback()?;
                        Err(e)
                    }
                }
            } else {
                Err(actix_web::error::ErrorInternalServerError(
                    "No Session Manager Present",
                ))?
            }

            // Set up transaction
        })
    }
}
