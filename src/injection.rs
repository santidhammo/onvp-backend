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
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::generic::storage::session::DefaultSessionManagerImplementation;
use crate::generic::Injectable;
use crate::model::interface::client::UserClaims;
use crate::repositories::definitions::{
    AuthorizationRepository, FacebookRepository, ImageRepository, MailTemplateRepository,
    MemberPictureRepository, MemberRepository, MemberRoleRepository, MusicalInstrumentRepository,
    PageRepository, PropertiesRepository, WorkgroupRepository, WorkgroupRoleRepository,
};
use crate::{repositories, services};
use actix_jwt_auth_middleware::TokenSigner;
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::web::Data;
use actix_web::{App, Error};
use jwt_compact::alg::Ed25519;

pub(crate) fn inject<T>(
    pool: &DatabaseConnectionPool,
    token_signer: &Data<TokenSigner<UserClaims, Ed25519>>,
    app: App<T>,
) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    let repositories = ServiceDependencies::dependencies(token_signer);
    let session_manager = DefaultSessionManagerImplementation::make(pool);

    let app = app.app_data(session_manager);
    let app = inject_command_services(app, &repositories);
    inject_request_services(app, &repositories)
}

fn inject_command_services<T>(app: App<T>, service_deps: &ServiceDependencies) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    use services::implementation::command::*;
    app.app_data(setup::Implementation::make(service_deps))
        .app_data(member::Implementation::make(service_deps))
        .app_data(workgroup::Implementation::make(service_deps))
        .app_data(member_picture::Implementation::make(service_deps))
        .app_data(member_activation::Implementation::make(service_deps))
        .app_data(role::Implementation::make(service_deps))
        .app_data(page::Implementation::make(service_deps))
        .app_data(image::Implementation::make(service_deps))
        .app_data(musical_instrument::Implementation::make(service_deps))
        .app_data(mail_template::Implementation::make(service_deps))
        .app_data(mailing::Implementation::make(service_deps))
}

fn inject_request_services<T>(app: App<T>, service_deps: &ServiceDependencies) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    use services::implementation::request::*;
    app.app_data(setup::Implementation::make(service_deps))
        .app_data(authorization::Implementation::make(service_deps))
        .app_data(member::Implementation::make(service_deps))
        .app_data(workgroup::Implementation::make(service_deps))
        .app_data(member_picture::Implementation::make(service_deps))
        .app_data(role::Implementation::make(service_deps))
        .app_data(facebook::Implementation::make(service_deps))
        .app_data(page::Implementation::make(service_deps))
        .app_data(image::Implementation::make(service_deps))
        .app_data(musical_instrument::Implementation::make(service_deps))
        .app_data(mail_template::Implementation::make(service_deps))
}

pub struct ServiceDependencies {
    pub properties_repository: Data<dyn PropertiesRepository>,
    pub member_repository: Data<dyn MemberRepository>,
    pub workgroup_repository: Data<dyn WorkgroupRepository>,
    pub member_role_repository: Data<dyn MemberRoleRepository>,
    pub member_picture_repository: Data<dyn MemberPictureRepository>,
    pub workgroup_role_repository: Data<dyn WorkgroupRoleRepository>,
    pub authorization_repository: Data<dyn AuthorizationRepository>,
    pub facebook_repository: Data<dyn FacebookRepository>,
    pub page_repository: Data<dyn PageRepository>,
    pub image_repository: Data<dyn ImageRepository>,
    pub musical_instrument_repository: Data<dyn MusicalInstrumentRepository>,
    pub mail_template_repository: Data<dyn MailTemplateRepository>,
    pub token_signer: Data<TokenSigner<UserClaims, Ed25519>>,
}

impl ServiceDependencies {
    fn dependencies(token_signer: &Data<TokenSigner<UserClaims, Ed25519>>) -> ServiceDependencies {
        use repositories::implementation::*;
        let repositories = ServiceDependencies {
            properties_repository: properties::Implementation::make(&()),
            member_repository: member::Implementation::make(&()),
            workgroup_repository: workgroup::Implementation::make(&()),
            member_role_repository: member_role::Implementation::make(&()),
            member_picture_repository: member_picture::Implementation::make(&()),
            workgroup_role_repository: workgroup_role::Implementation::make(&()),
            authorization_repository: authorization::Implementation::make(&()),
            facebook_repository: facebook::Implementation::make(&()),
            page_repository: page::Implementation::make(&()),
            image_repository: image::Implementation::make(&()),
            musical_instrument_repository: musical_instrument::Implementation::make(&()),
            mail_template_repository: mail_template::Implementation::make(&()),
            token_signer: token_signer.clone(),
        };
        repositories
    }
}
