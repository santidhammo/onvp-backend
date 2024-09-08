use actix_jwt_auth_middleware::{Authority, TokenSigner};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use ed25519_compact::{KeyPair, PublicKey, SecretKey};
use jwt_compact::alg::Ed25519;
use log::info;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::Read;
use std::net::Ipv4Addr;
use std::path::Path;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

pub mod members;
pub mod setup;

use crate::model::security::{Role, UserClaims};
use crate::{dal, model};

#[derive(OpenApi)]
#[openapi(
    paths(
        setup::should_setup,
        setup::setup_first_operator,
        members::list,
        members::activation_code,
        members::activate,
        members::login,
    ),
    components(
        schemas(model::members::Member),
        schemas(model::setup::FirstOperator),
        schemas(model::security::TokenData),
        schemas(model::security::LoginData),
    ),
    tags(
        (name = "api::members", description = "Member management endpoints"),
        (name = "api::setup", description = "Application setup endpoints")
    ),
)]
pub struct ApiDoc;

pub async fn run_api_server() -> std::io::Result<()> {
    let (secret_key, public_key) = load_key_pair();

    let pool = dal::initialize_db_pool();

    let token_signer = TokenSigner::new()
        .signing_key(secret_key.clone())
        .algorithm(Ed25519)
        .build()
        .expect("Token Signer should be initialized");

    Ok(HttpServer::new(move || {
        let authority = Authority::<UserClaims, Ed25519, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .token_signer(Some(token_signer.clone()))
            .verifying_key(public_key)
            .build()
            .expect("Token Verifier should be initialized");

        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(token_signer.clone()))
            .service(
                web::scope(members::CONTEXT)
                    .service(members::list)
                    .service(members::activation_code)
                    .service(members::activate)
                    .service(members::login),
            )
            .service(
                web::scope(setup::CONTEXT)
                    .service(setup::should_setup)
                    .service(setup::setup_first_operator),
            )
            .service(RapiDoc::with_openapi("/docs/openapi.json", ApiDoc::openapi()).path("/docs"))
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
    _ = File::open(path)
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
