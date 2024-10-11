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

//! Exposes the API

use actix_jwt_auth_middleware::use_jwt::UseJWTOnScope;
use actix_jwt_auth_middleware::{Authority, TokenSigner};
use actix_state_guards::UseStateGuardOnScope;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use ed25519_compact::{KeyPair, PublicKey, SecretKey};
use jwt_compact::alg::Ed25519;
use log::info;
use std::env;
use std::fs::File;
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

mod authorization;
pub mod members;
pub mod roles;
pub mod setup;
pub mod workgroups;

use crate::generic::security;
use crate::generic::storage::database;
use crate::model::interface::client::*;
use crate::model::interface::commands::*;
use crate::model::interface::requests::*;
use crate::model::interface::responses::*;
use crate::model::interface::search::*;
use crate::model::primitives::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        setup::should_setup,
        setup::setup_first_operator,
        authorization::login,
        authorization::refresh,
        authorization::logout,
        members::register,
        members::activation_code,
        members::activate,
        members::search,
        members::find,
        members::find_address,
        members::update,
        members::update_address,
        members::upload_picture_asset,
        members::picture_asset,
        members::picture,
        roles::associate,
        roles::dissociate,
        roles::list,
        workgroups::register,
        workgroups::search,
        workgroups::find,
        workgroups::update,
        source_code::details,
    ),
    components(
        schemas(MemberActivationCommand),
        schemas(FirstOperatorRegisterCommand),
        schemas(MemberRegisterCommand),
        schemas(MemberUpdateCommand),
        schemas(MemberUpdateAddressCommand),
        schemas(WorkgroupRegisterCommand),
        schemas(AssociateRoleCommand),
        schemas(DissociateRoleCommand),
        schemas(AuthorizationRequest),
        schemas(SearchParams),
        schemas(SearchResult<MemberResponse>),
        schemas(SearchResult<WorkgroupResponse>),
        schemas(MemberResponse),
        schemas(WorkgroupResponse),
        schemas(MemberAddressResponse),
        schemas(WorkgroupResponse),
        schemas(RoleClass),
        schemas(Role),
    ),
    tags(
        (name = "api::members", description = "Member management endpoints"),
        (name = "api::setup", description = "Application setup endpoints"),
        (name = "api::workgroups", description = "Workgroup management endpoints"),
        (name = "api::source_code", description = "Source Code endpoints")
    ),
)]
pub struct ApiDoc;

pub async fn run_api_server() -> std::io::Result<()> {
    let (secret_key, public_key) = load_key_pair();

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

        let app = crate::injection::inject(&pool, &Data::new(token_signer.clone()), App::new());
        app.wrap(Logger::default())
            .service(
                web::scope(members::CONTEXT)
                    .service(members::activation_code)
                    .service(members::activate)
                    .use_jwt(
                        authority.clone(),
                        web::scope("")
                            .service(members::picture_asset)
                            .service(members::picture)
                            .use_state_guard(
                                |claims: UserClaims| async move {
                                    security::operator_state_guard(&claims)
                                },
                                web::scope("")
                                    .service(members::search)
                                    .service(members::find)
                                    .service(members::find_address)
                                    .service(members::update)
                                    .service(members::update_address)
                                    .service(members::upload_picture_asset)
                                    .service(members::register),
                            ),
                    ),
            )
            .service(
                web::scope(authorization::CONTEXT)
                    .service(authorization::login)
                    .use_jwt(
                        authority.clone(),
                        web::scope("")
                            .service(authorization::refresh)
                            .service(authorization::logout),
                    ),
            )
            .service(web::scope(workgroups::CONTEXT).use_jwt(
                authority.clone(),
                web::scope("")
                    .service(workgroups::search)
                    .service(workgroups::find)
                    .use_state_guard(
                    |claims: UserClaims| async move { security::operator_state_guard(&claims) },
                    web::scope("").service(workgroups::register).service(workgroups::update),
                ),
            ))
            .service(
                web::scope(roles::CONTEXT).use_jwt(
                    authority.clone(),
                    web::scope("").use_state_guard(
                        |claims: UserClaims| async move { security::operator_state_guard(&claims) },
                        web::scope("")
                            .service(roles::associate)
                            .service(roles::dissociate),
                    ),
                ),
            )
            .service(
                web::scope(setup::CONTEXT)
                    .service(setup::should_setup)
                    .service(setup::setup_first_operator),
            )
            .service(Scalar::with_url("/docs", ApiDoc::openapi()))
            .service(source_code::details)
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await?)
}

fn load_key_pair() -> (SecretKey, PublicKey) {
    let keys_location = env::var("JWT_KEYS").expect("JWT_KEYS should be set");
    info!("Loading JWT keys from {}", keys_location);
    let path = Path::new(&keys_location);
    let mut pem = String::new();
    let _ = File::open(path)
        .expect("JWT_KEYS should exist")
        .read_to_string(&mut pem)
        .expect("JWT_KEYS should be readable");
    let KeyPair {
        sk: secret_key,
        pk: public_key,
    } = KeyPair::from_pem(&pem)
        .expect("Key pair should be created with the specified file in JWT_KEYS");
    (secret_key, public_key)
}

mod source_code {
    use crate::{BACKEND_SOURCE_CODE_URL, FRONTEND_SOURCE_CODE_URL};
    use actix_web::get;
    use actix_web::web::Json;
    use serde::Serialize;
    use std::sync::LazyLock;
    use utoipa::ToSchema;

    /// Shows the source code details of the frontend and backend
    #[utoipa::path(
        context_path = "",
        responses(
            (status = 200, description = "Source code details")
        )
    )]
    #[get("/api/source_code_details")]
    pub async fn details() -> Json<SourceCodeDetails> {
        Json(SOURCE_CODE_DETAILS.clone())
    }

    static SOURCE_CODE_DETAILS: LazyLock<SourceCodeDetails> = LazyLock::new(|| SourceCodeDetails {
        frontend_url: FRONTEND_SOURCE_CODE_URL.clone(),
        backend_url: BACKEND_SOURCE_CODE_URL.clone(),
    });

    #[derive(Serialize, ToSchema, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SourceCodeDetails {
        frontend_url: String,
        backend_url: String,
    }
}
