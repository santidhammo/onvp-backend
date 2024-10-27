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

use crate::api;
use crate::api::config;
use crate::api::endpoints::*;
use crate::api::middleware::authority::AuthorityMiddleware;
use crate::generic::storage::database;
use crate::model::interface::client::UserClaims;
use actix_jwt_auth_middleware::{Authority, TokenSigner};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use jwt_compact::alg::Ed25519;
use std::net::Ipv4Addr;
use std::time::Duration;
use utoipa_actix_web::{scope, AppExt};
use utoipa_scalar::{Scalar, Servable};

pub async fn launch() -> std::io::Result<()> {
    let (secret_key, public_key) = api::load_key_pair();

    let pool = database::initialize_database_connection_pool();

    let token_signer = TokenSigner::new()
        .signing_key(secret_key.clone())
        .algorithm(Ed25519)
        .access_token_lifetime(Duration::from_secs(3 * 60))
        .refresh_token_lifetime(Duration::from_secs(10 * 60))
        .build()
        .expect("Token Signer should be initialized");

    Ok(HttpServer::new(move || {
        let authority = Authority::<UserClaims, Ed25519, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(token_signer.clone()))
            .verifying_key(public_key)
            .build()
            .expect("Token Verifier should be initialized");

        let authority_middleware =
            AuthorityMiddleware::new(authority, config::configure_authority());

        let app = crate::injection::inject(&pool, &Data::new(token_signer.clone()), App::new());
        let (app, api) = app
            .into_utoipa_app()
            .map(|app| app.wrap(Logger::default()).wrap(authority_middleware))
            .service(
                scope("/api/authorization")
                    .service(authorization::login)
                    .service(authorization::logout)
                    .service(authorization::refresh),
            )
            .service(
                scope("api/members")
                    .service(members::activation_code)
                    .service(members::activate)
                    .service(members::picture_asset)
                    .service(members::picture)
                    .service(members::search)
                    .service(members::find)
                    .service(members::find_address)
                    .service(members::find_workgroups)
                    .service(members::find_privacy_info_sharing)
                    .service(members::update)
                    .service(members::update_address)
                    .service(members::update_privacy_info_sharing)
                    .service(members::upload_picture_asset)
                    .service(members::register)
                    .service(members::unregister),
            )
            .service(
                scope("/api/facebook")
                    .service(facebook::search)
                    .service(facebook::picture_asset),
            )
            .service(
                scope("/api/roles")
                    .service(roles::associate)
                    .service(roles::dissociate)
                    .service(roles::list),
            )
            .service(
                scope("/api/setup")
                    .service(setup::should_setup)
                    .service(setup::setup_first_operator),
            )
            .service(
                scope("/api/workgroups")
                    .service(workgroups::search)
                    .service(workgroups::find)
                    .service(workgroups::register)
                    .service(workgroups::associate)
                    .service(workgroups::dissociate)
                    .service(workgroups::update)
                    .service(workgroups::find_members)
                    .service(workgroups::available_members_search)
                    .service(workgroups::unregister),
            )
            .service(scope("/api/source_code_details").service(source_code::details))
            .split_for_parts();

        app.service(Scalar::with_url("/docs", api))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await?)
}
