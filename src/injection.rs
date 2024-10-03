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

use crate::{repositories, services};
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::web::Data;
use actix_web::{App, Error};

pub(crate) fn inject<T>(pool: &crate::dal::DbPool, app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let member_role_repository =
        repositories::implementation::member_role::Implementation::injectable(());
    let workgroup_role_repository =
        repositories::implementation::workgroup_role::Implementation::injectable(());

    app.app_data(services::implementation::command::member::Implementation::injectable(pool))
        .app_data(services::implementation::request::member::Implementation::injectable(pool))
        .app_data(
            services::implementation::command::role::Implementation::injectable((
                &member_role_repository,
                &workgroup_role_repository,
                pool,
            )),
        )
        .app_data(member_role_repository)
}

/// This trait is implemented by all injectables with the need of a data object itself
pub trait Injectable<U, T: ?Sized> {
    fn injectable(dependencies: U) -> Data<T>;
}
