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
use crate::generic::Injectable;
use crate::model::interface::client::UserClaims;
use crate::repositories::traits::{
    AuthorizationRepository, MemberPictureRepository, MemberRepository, MemberRoleRepository,
    WorkgroupRoleRepository,
};
use crate::{repositories, services};
use actix_jwt_auth_middleware::TokenSigner;
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::web::Data;
use actix_web::{App, Error};
use jwt_compact::alg::Ed25519;

pub(crate) fn inject<T>(
    pool: &crate::generic::storage::database::DatabaseConnectionPool,
    token_signer: &Data<TokenSigner<UserClaims, Ed25519>>,
    app: App<T>,
) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let repositories = DependantRepositories::dependencies();

    app.app_data(
        services::implementation::command::setup::Implementation::injectable((
            pool,
            &repositories.member_repository,
            &repositories.member_role_repository,
        )),
    )
    .app_data(
        services::implementation::command::member::Implementation::injectable((
            pool,
            &repositories.member_repository,
            &repositories.member_role_repository,
        )),
    )
    .app_data(
        services::implementation::command::member_picture::Implementation::injectable((
            pool,
            &repositories.member_repository,
            &repositories.member_picture_repository,
        )),
    )
    .app_data(
        services::implementation::command::member_activation::Implementation::injectable((
            pool,
            &repositories.member_repository,
        )),
    )
    .app_data(
        services::implementation::command::role::Implementation::injectable((
            pool,
            &repositories.member_role_repository,
            &repositories.workgroup_role_repository,
        )),
    )
    .app_data(
        services::implementation::request::setup::Implementation::injectable((
            pool,
            &repositories.member_repository,
        )),
    )
    .app_data(
        services::implementation::request::authorization::Implementation::injectable((
            pool,
            &repositories.member_repository,
            &repositories.authorization_repository,
            &token_signer,
        )),
    )
    .app_data(
        services::implementation::request::member::Implementation::injectable((
            pool,
            &repositories.member_repository,
        )),
    )
    .app_data(
        services::implementation::request::member_picture::Implementation::injectable((
            pool,
            &repositories.member_repository,
        )),
    )
}

struct DependantRepositories {
    member_repository: Data<dyn MemberRepository>,
    member_role_repository: Data<dyn MemberRoleRepository>,
    member_picture_repository: Data<dyn MemberPictureRepository>,
    workgroup_role_repository: Data<dyn WorkgroupRoleRepository>,
    authorization_repository: Data<dyn AuthorizationRepository>,
}

impl DependantRepositories {
    fn dependencies() -> DependantRepositories {
        let repositories = DependantRepositories {
            member_repository: repositories::implementation::member::Implementation::injectable(()),
            member_role_repository:
                repositories::implementation::member_role::Implementation::injectable(()),
            member_picture_repository:
                repositories::implementation::member_picture::Implementation::injectable(()),
            workgroup_role_repository:
                repositories::implementation::workgroup_role::Implementation::injectable(()),
            authorization_repository:
                repositories::implementation::authorization::Implementation::injectable(()),
        };
        repositories
    }
}
