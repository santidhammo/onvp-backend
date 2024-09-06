use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::net::Ipv4Addr;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

pub mod members;
pub mod setup;

use crate::model;

#[derive(OpenApi)]
#[openapi(
    paths(
        setup::should_setup,
        setup::setup_first_operator,
        members::list,
        members::activation_code,
        members::activate,
    ),
    components(
        schemas(model::members::Member),
        schemas(model::setup::FirstOperator),
        schemas(model::security::TokenData)
    ),
    tags(
        (name = "api::members", description = "Member management endpoints"),
        (name = "api::setup", description = "Application setup endpoints")
    ),
)]
pub struct ApiDoc;

pub async fn run_api_server() -> std::io::Result<()> {
    let pool = crate::initialize_db_pool();

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope(members::CONTEXT)
                    .service(members::list)
                    .service(members::activation_code)
                    .service(members::activate),
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
